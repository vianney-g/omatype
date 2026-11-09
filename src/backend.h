#pragma once

#include <QElapsedTimer>
#include <QObject>
#include <QSettings>
#include <QTimer>
#include <QVariantMap>
#include <functional>
#include <memory>

#include "engine.h"
#include "wordmodel.h"

// The running test and the history, for QML. The rules and the numbers live
// in the Rust engine; this keeps the window's copy of the words in step and
// remembers the test settings between launches.
class Backend : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString mode READ mode WRITE setMode NOTIFY configChanged)
    Q_PROPERTY(int value READ value WRITE setValue NOTIFY configChanged)
    Q_PROPERTY(bool punctuation READ punctuation WRITE setPunctuation NOTIFY configChanged)
    Q_PROPERTY(bool numbers READ numbers WRITE setNumbers NOTIFY configChanged)
    Q_PROPERTY(bool stopOnError READ stopOnError WRITE setStopOnError NOTIFY configChanged)
    Q_PROPERTY(bool confidence READ confidence WRITE setConfidence NOTIFY configChanged)
    Q_PROPERTY(QString language READ language WRITE setLanguage NOTIFY configChanged)
    Q_PROPERTY(QObject *words READ words CONSTANT)
    Q_PROPERTY(int current READ current NOTIFY progressed)
    // Letters typed into the current word, which places the caret.
    Q_PROPERTY(int caret READ caret NOTIFY progressed)
    Q_PROPERTY(bool started READ started NOTIFY stateChanged)
    Q_PROPERTY(bool running READ running NOTIFY stateChanged)
    Q_PROPERTY(QVariantMap live READ live NOTIFY liveChanged)
    Q_PROPERTY(QString quoteSource READ quoteSource NOTIFY stateChanged)
    Q_PROPERTY(QVariantMap result READ result NOTIFY resultChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    Q_PROPERTY(int revision READ revision NOTIFY historyChanged)

public:
    explicit Backend(const QString &databasePath, const QString &settingsPath, QObject *parent = nullptr);
    ~Backend() override;

    static QString defaultDatabasePath();
    static QString defaultSettingsPath();

    QString mode() const { return m_mode; }
    int value() const { return m_value; }
    bool punctuation() const { return m_punctuation; }
    bool numbers() const { return m_numbers; }
    bool stopOnError() const { return m_stopOnError; }
    bool confidence() const { return m_confidence; }
    QString language() const { return m_language; }
    QObject *words() { return &m_words; }
    WordModel &wordModel() { return m_words; }
    int current() const { return m_current; }
    int caret() const { return m_caret; }
    bool started() const { return m_started; }
    bool running() const { return m_started && !m_finished; }
    QVariantMap live() const { return m_live; }
    QString quoteSource() const { return m_quoteSource; }
    QVariantMap result() const { return m_result; }
    QString error() const { return m_error; }
    int revision() const { return m_revision; }

    void setMode(const QString &mode);
    void setValue(int value);
    void setPunctuation(bool on);
    void setNumbers(bool on);
    void setStopOnError(bool on);
    void setConfidence(bool on);
    void setLanguage(const QString &language);

    // The usual value for each mode, so switching modes lands somewhere sensible.
    Q_INVOKABLE static int defaultValue(const QString &mode);
    Q_INVOKABLE void restart();
    Q_INVOKABLE void type(const QString &text);
    Q_INVOKABLE void backspace(bool wholeWord);
    Q_INVOKABLE void finishTest();
    Q_INVOKABLE void tick();
    // Reads the history: test, recent, daily, bests, totals, weakKeys.
    Q_INVOKABLE QVariant history(const QString &query, const QVariantMap &args = {});
    Q_INVOKABLE bool deleteTest(qint64 id);
    Q_INVOKABLE void showResult(qint64 id);

    // Tests drive time by hand; the app uses a monotonic clock.
    void setClock(std::function<double()> clock) { m_clock = std::move(clock); }

signals:
    void configChanged();
    void progressed();
    void stateChanged();
    void liveChanged();
    void resultChanged();
    void errorChanged();
    void historyChanged();
    void finished();

private:
    double now() const { return m_clock(); }
    QJsonObject config() const;
    void saveConfig();
    void apply(const QJsonObject &update);
    void setError(const QString &error);
    void conclude(const QJsonObject &result);

    std::unique_ptr<EngineHandle> m_engine;
    QSettings m_settings;
    WordModel m_words;
    QElapsedTimer m_elapsed;
    std::function<double()> m_clock;
    QTimer m_ticker;

    QString m_mode = "time";
    int m_value = 30;
    bool m_punctuation = false;
    bool m_numbers = false;
    bool m_stopOnError = false;
    bool m_confidence = false;
    QString m_language = "en";

    int m_current = 0;
    int m_caret = 0;
    bool m_started = false;
    bool m_finished = false;
    QVariantMap m_live;
    QString m_quoteSource;
    QVariantMap m_result;
    QString m_error;
    QString m_storeError;
    int m_revision = 0;
};
