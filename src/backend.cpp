#include "backend.h"

#include <QDir>
#include <QJsonArray>
#include <QJsonDocument>
#include <QStandardPaths>

static QStringList strings(const QJsonValue &value) {
    QStringList list;
    for (const auto &v : value.toArray())
        list.append(v.toString());
    return list;
}

QString Backend::defaultDatabasePath() {
    const QString overridden = qEnvironmentVariable("OMATYPE_DB");
    if (!overridden.isEmpty())
        return overridden;
    return QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation) + "/omatype/history.db";
}

QString Backend::defaultSettingsPath() {
    const QString overridden = qEnvironmentVariable("OMATYPE_SETTINGS");
    if (!overridden.isEmpty())
        return overridden;
    return QStandardPaths::writableLocation(QStandardPaths::GenericConfigLocation) + "/omatype/settings.ini";
}

Backend::Backend(const QString &databasePath, const QString &settingsPath, QObject *parent)
    : QObject(parent),
      m_engine(std::make_unique<EngineHandle>(databasePath)),
      m_settings(settingsPath, QSettings::IniFormat) {
    m_elapsed.start();
    m_clock = [this] { return m_elapsed.nsecsElapsed() / 1e6; };

    m_mode = m_settings.value("mode", "time").toString();
    m_value = m_settings.value("value", defaultValue(m_mode)).toInt();
    m_punctuation = m_settings.value("punctuation", false).toBool();
    m_numbers = m_settings.value("numbers", false).toBool();
    m_stopOnError = m_settings.value("stopOnError", false).toBool();
    m_confidence = m_settings.value("confidence", false).toBool();
    m_language = m_settings.value("language", "en").toString();

    // A history that can't be opened is shown in the window: tests still
    // run, but nothing would be kept, and the user should know.
    const auto status = m_engine->call("status");
    m_storeError = status.data.toObject().value("storeError").toString();
    m_error = m_storeError.isEmpty() ? QString() : tr("History unavailable: %1").arg(m_storeError);

    // Time tests end on their own; the ticker also refreshes the live speed.
    m_ticker.setInterval(100);
    connect(&m_ticker, &QTimer::timeout, this, &Backend::tick);
    restart();
}

Backend::~Backend() = default;

int Backend::defaultValue(const QString &mode) {
    if (mode == "words") return 25;
    if (mode == "quote") return 0;
    if (mode == "zen") return 0;
    return 30;
}

void Backend::setMode(const QString &mode) {
    static const QStringList modes{"time", "words", "quote", "zen"};
    if (!modes.contains(mode) || mode == m_mode)
        return;
    m_mode = mode;
    m_value = defaultValue(mode);
    saveConfig();
}

void Backend::setValue(int value) {
    if (value == m_value) return;
    m_value = value;
    saveConfig();
}

void Backend::setPunctuation(bool on) {
    if (on == m_punctuation) return;
    m_punctuation = on;
    saveConfig();
}

void Backend::setNumbers(bool on) {
    if (on == m_numbers) return;
    m_numbers = on;
    saveConfig();
}

void Backend::setStopOnError(bool on) {
    if (on == m_stopOnError) return;
    m_stopOnError = on;
    saveConfig();
}

void Backend::setConfidence(bool on) {
    if (on == m_confidence) return;
    m_confidence = on;
    saveConfig();
}

void Backend::setLanguage(const QString &language) {
    if (language == m_language || (language != "en" && language != "fr")) return;
    m_language = language;
    saveConfig();
}

// Every change of settings starts a fresh test with them.
void Backend::saveConfig() {
    m_settings.setValue("mode", m_mode);
    m_settings.setValue("value", m_value);
    m_settings.setValue("punctuation", m_punctuation);
    m_settings.setValue("numbers", m_numbers);
    m_settings.setValue("stopOnError", m_stopOnError);
    m_settings.setValue("confidence", m_confidence);
    m_settings.setValue("language", m_language);
    m_settings.sync();
    emit configChanged();
    restart();
}

QJsonObject Backend::config() const {
    return {{"mode", m_mode},          {"value", m_value},           {"punctuation", m_punctuation},
            {"numbers", m_numbers},    {"stopOnError", m_stopOnError}, {"confidence", m_confidence},
            {"language", m_language}};
}

void Backend::setError(const QString &error) {
    if (error == m_error) return;
    m_error = error;
    emit errorChanged();
}

void Backend::restart() {
    m_ticker.stop();
    const auto reply = m_engine->call("start", {{"config", config()}, {"now", now()}});
    if (!reply.ok) {
        setError(reply.error);
        return;
    }
    const QJsonObject data = reply.data.toObject();
    // Zen has no text; its first word exists empty so the caret has a place.
    const QStringList words = strings(data.value("words"));
    m_words.reset(m_mode == "zen" ? QStringList{QString()} : words);
    m_quoteSource = data.value("quote").toString();
    m_current = 0;
    m_caret = 0;
    m_started = false;
    m_finished = false;
    m_live = data.value("live").toObject().toVariantMap();
    emit progressed();
    emit stateChanged();
    emit liveChanged();
}

void Backend::apply(const QJsonObject &update) {
    m_words.append(strings(update.value("append")));
    for (const auto &c : update.value("changed").toArray()) {
        const QJsonObject change = c.toObject();
        m_words.setTyped(change.value("index").toInt(), change.value("typed").toString());
    }
    const int current = update.value("current").toInt();
    if (m_mode == "zen")
        m_words.truncate(current + 1);
    const int caret = int(m_words.typed(current).toUcs4().size());
    if (current != m_current || caret != m_caret) {
        m_current = current;
        m_caret = caret;
        emit progressed();
    }
    const bool started = update.value("started").toBool();
    if (started && !m_started) {
        m_started = true;
        m_ticker.start();
        emit stateChanged();
        tick();
    }
    if (update.contains("result"))
        conclude(update.value("result").toObject());
}

void Backend::conclude(const QJsonObject &result) {
    m_ticker.stop();
    m_finished = true;
    m_result = result.toVariantMap();
    ++m_revision;
    emit stateChanged();
    emit resultChanged();
    emit historyChanged();
    emit finished();
}

void Backend::type(const QString &text) {
    if (m_finished || text.isEmpty()) return;
    const auto reply = m_engine->call("input", {{"text", text}, {"now", now()}});
    if (reply.ok) apply(reply.data.toObject());
    else setError(reply.error);
}

void Backend::backspace(bool wholeWord) {
    if (m_finished) return;
    const auto reply = m_engine->call("backspace", {{"word", wholeWord}, {"now", now()}});
    if (reply.ok) apply(reply.data.toObject());
}

void Backend::finishTest() {
    if (!running()) return;
    const auto reply = m_engine->call("finish", {{"now", now()}});
    if (reply.ok) apply(reply.data.toObject());
    else setError(reply.error);
}

void Backend::tick() {
    if (!running()) return;
    const auto reply = m_engine->call("tick", {{"now", now()}});
    if (!reply.ok) {
        setError(reply.error);
        return;
    }
    const QJsonObject data = reply.data.toObject();
    m_live = data.value("live").toObject().toVariantMap();
    emit liveChanged();
    if (data.contains("result"))
        conclude(data.value("result").toObject());
}

QVariant Backend::history(const QString &query, const QVariantMap &args) {
    if (!m_storeError.isEmpty())
        return {};
    const auto reply = m_engine->call(query, QJsonObject::fromVariantMap(args));
    if (!reply.ok) {
        setError(reply.error);
        return {};
    }
    return reply.data.toVariant();
}

bool Backend::deleteTest(qint64 id) {
    const auto reply = m_engine->call("delete", {{"id", id}});
    if (!reply.ok || !reply.data.toBool())
        return false;
    ++m_revision;
    emit historyChanged();
    return true;
}

void Backend::showResult(qint64 id) {
    const auto reply = m_engine->call("test", {{"id", id}});
    if (!reply.ok || !reply.data.isObject())
        return;
    m_result = reply.data.toObject().toVariantMap();
    emit resultChanged();
}
