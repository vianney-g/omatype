// Theme watching follows Omacut (MIT, David Heinemeier Hansson).
#include "theme.h"

#include <QColor>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QRegularExpression>
#include <QStandardPaths>
#include <cmath>

// Used when Omarchy isn't there: a dark palette with Omarchy's default accent.
static const QHash<QString, QString> fallback{
    {"accent", "#FFD60A"}, {"background", "#0e0e10"}, {"foreground", "#e6e6e6"},
    {"muted", "#6c6c74"},  {"red", "#e5534b"},
};

Theme::Theme(const QString &directory, QObject *parent, const QString &hyprctl)
    : QObject(parent),
      m_directory(directory.isEmpty() ? QDir::homePath() + "/.local/state/omarchy/current" : directory) {
    m_debounce.setSingleShot(true);
    m_debounce.setInterval(80);
    connect(&m_watcher, &QFileSystemWatcher::directoryChanged, this, [this] { m_debounce.start(); });
    connect(&m_watcher, &QFileSystemWatcher::fileChanged, this, [this] { m_debounce.start(); });
    connect(&m_debounce, &QTimer::timeout, this, &Theme::reload);

    // Corners follow Hyprland's effective rounding, which themes and personal
    // overrides can both set, so ask hyprctl rather than reading its config.
    // Without Hyprland the corners stay sharp, like Omarchy's default.
    m_hyprctl = !hyprctl.isEmpty() ? hyprctl
              : directory.isEmpty() && !qEnvironmentVariableIsEmpty("HYPRLAND_INSTANCE_SIGNATURE")
                  ? QStandardPaths::findExecutable("hyprctl") : QString();
    m_roundingTimeout.setSingleShot(true);
    m_roundingTimeout.setInterval(500);
    connect(&m_roundingTimeout, &QTimer::timeout, &m_roundingProcess, &QProcess::kill);
    connect(&m_roundingProcess, &QProcess::errorOccurred, this, [this] { m_roundingTimeout.stop(); });
    connect(&m_roundingProcess, &QProcess::finished, this, [this](int code, QProcess::ExitStatus status) {
        m_roundingTimeout.stop();
        const auto value = QJsonDocument::fromJson(m_roundingProcess.readAllStandardOutput()).object().value("int");
        if (code != 0 || status != QProcess::NormalExit || !value.isDouble() || value.toInt(-1) < 0)
            return;
        if (value.toInt() != m_radius) {
            m_radius = value.toInt();
            emit changed();
        }
    });
    m_roundingPoll.setInterval(1000);
    connect(&m_roundingPoll, &QTimer::timeout, this, &Theme::refreshRounding);
    if (!m_hyprctl.isEmpty())
        m_roundingPoll.start();
    reload();
}

Theme::~Theme() {
    if (m_roundingProcess.state() != QProcess::NotRunning) {
        m_roundingProcess.kill();
        m_roundingProcess.waitForFinished(1000);
    }
}

void Theme::refreshRounding() {
    if (m_hyprctl.isEmpty() || m_roundingProcess.state() != QProcess::NotRunning)
        return;
    m_roundingProcess.start(m_hyprctl, {"-j", "getoption", "decoration:rounding"});
    m_roundingTimeout.start();
}

// Every `key = "value"` line of colors.toml; hex colors and the mode.
QHash<QString, QString> Theme::readColors(const QString &path) {
    QHash<QString, QString> colors;
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly))
        return colors;
    static const QRegularExpression line(R"re(^\s*([a-z0-9_]+)\s*=\s*["']([^"']*)["'])re",
                                         QRegularExpression::MultilineOption);
    auto it = line.globalMatch(QString::fromUtf8(file.readAll()));
    while (it.hasNext()) {
        const auto m = it.next();
        colors.insert(m.captured(1), m.captured(2));
    }
    return colors;
}

QString Theme::readAccent(const QString &path) {
    const QString accent = readColors(path).value("accent");
    static const QRegularExpression hex("^#[0-9a-fA-F]{6}$");
    return hex.match(accent).hasMatch() ? accent : fallback.value("accent");
}

QString Theme::color(const QString &key) const {
    static const QRegularExpression hex("^#[0-9a-fA-F]{6}$");
    const QString value = m_colors.value(key);
    return hex.match(value).hasMatch() ? value : fallback.value(key);
}

// A panel color a step from the background toward the text, so cards read on
// light and dark themes alike.
QString Theme::surface() const {
    const QColor bg(background()), fg(foreground());
    const double t = dark() ? 0.06 : 0.07;
    return QColor::fromRgbF(bg.redF() + (fg.redF() - bg.redF()) * t, bg.greenF() + (fg.greenF() - bg.greenF()) * t,
                            bg.blueF() + (fg.blueF() - bg.blueF()) * t)
        .name();
}

// Black or white, whichever reads on the accent (WCAG relative luminance).
QString Theme::accentForeground() const {
    const QColor c(accent());
    auto linear = [](double v) { return v <= .04045 ? v / 12.92 : std::pow((v + .055) / 1.055, 2.4); };
    const double luminance = .2126 * linear(c.redF()) + .7152 * linear(c.greenF()) + .0722 * linear(c.blueF());
    return luminance > .179 ? "black" : "white";
}

void Theme::reload() {
    refreshRounding();
    const auto paths = m_watcher.files() + m_watcher.directories();
    if (!paths.isEmpty())
        m_watcher.removePaths(paths);

    // A theme switch replaces symlinks and files, so watch the parents too and
    // re-arm after every change. Watching the nearest existing ancestor means
    // installing Omarchy later works without restarting the app.
    QString ancestor = m_directory;
    while (!QFileInfo::exists(ancestor) && ancestor != "/")
        ancestor = QFileInfo(ancestor).absolutePath();
    QStringList candidates{ancestor, QFileInfo(ancestor).absolutePath(), m_directory,
                           m_directory + "/theme", m_directory + "/theme/colors.toml"};
    candidates.removeDuplicates();
    for (const auto &path : candidates)
        if (QFileInfo::exists(path))
            m_watcher.addPath(path);

    const auto colors = readColors(m_directory + "/theme/colors.toml");
    // A theme without a background is treated as missing, so the fallback
    // palette stays consistent rather than mixing light text on light.
    const QString mode = colors.contains("background") ? colors.value("mode", "dark") : "dark";
    if (colors != m_colors || mode != m_mode) {
        m_colors = colors.contains("background") ? colors : QHash<QString, QString>{{"accent", colors.value("accent")}};
        m_mode = mode;
        emit changed();
    }
}
