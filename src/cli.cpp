// Command-line commands, after Hype's src/cli.cpp (MIT, David Heinemeier Hansson).
#include "cli.h"

#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QSaveFile>
#include <QTextStream>

#include "backend.h"
#include "engine.h"

static const char *skill = R"(---
name: omatype
description: Read the user's typing practice history from omatype (speed, accuracy, consistency, personal bests, daily progression, weak keys). Use when asked how their typing is going, for typing stats, or to summarize practice.
---

# omatype

omatype is a typing trainer. Every finished test and every keystroke is kept
in a local SQLite history. Read it through the CLI, never the database:

- `omatype stats --json`: totals, last-10 averages, personal bests per test
  type, the last 30 days, and the keys missed most often.
- `omatype history --json [--limit N]`: recent tests, newest first.
- `omatype test <id> --json`: one test with its words, what was typed, and a
  per-second series of net speed (wpm), raw speed, and errors.

Speeds are words per minute (5 characters a word). `wpm` counts only correct
words; `raw` counts every keystroke. Accuracy is the share of correct
keystrokes, consistency is 0-100 from the variation of per-second raw speed.
Each command exits 0, or 1 with the error on stderr.
)";

bool isCommand(const QStringList &arguments) {
    static const QStringList commands{"stats", "history", "test", "skill", "help", "--help", "-h"};
    return arguments.size() > 1 && commands.contains(arguments.at(1));
}

static void out(const QString &text) {
    QTextStream(stdout) << text;
}

static int fail(const QString &message) {
    QTextStream(stderr) << "omatype: " << message << "\n";
    return 1;
}

static QString modeLabel(const QJsonObject &t) {
    const QString mode = t.value("mode").toString();
    QString label = mode;
    const int value = t.value("value").toInt();
    if (mode == "time") label += QString(" %1s").arg(value);
    else if (mode == "words") label += QString(" %1").arg(value);
    else if (mode == "quote") label += value == 1 ? " short" : value == 2 ? " medium" : value == 3 ? " long" : "";
    if (t.value("punctuation").toBool()) label += " +punct";
    if (t.value("numbers").toBool()) label += " +num";
    if (t.value("language").toString() != "en") label += " " + t.value("language").toString();
    return label;
}

static QString row(const QJsonObject &t) {
    return QString("%1  %2  %3 wpm  %4 raw  %5% acc  %6% cons")
        .arg(t.value("id").toInt(), 5)
        .arg(modeLabel(t), -22)
        .arg(t.value("wpm").toDouble(), 6, 'f', 1)
        .arg(t.value("raw").toDouble(), 6, 'f', 1)
        .arg(t.value("accuracy").toDouble(), 5, 'f', 1)
        .arg(t.value("consistency").toDouble(), 5, 'f', 1);
}

int runCommand(const QStringList &arguments) {
    if (!isCommand(arguments))
        return -1;
    const QString command = arguments.at(1);
    const bool json = arguments.contains("--json");

    if (command == "help" || command == "--help" || command == "-h") {
        out("Usage: omatype [open]\n"
            "       omatype stats [--json]\n"
            "       omatype history [--limit N] [--json]\n"
            "       omatype test <id> [--json]\n"
            "       omatype skill install\n");
        return 0;
    }

    if (command == "skill") {
        if (arguments.value(2) != "install")
            return fail("usage: omatype skill install");
        const QString dir = QDir::homePath() + "/.claude/skills/omatype";
        if (!QDir().mkpath(dir))
            return fail("cannot create " + dir);
        QSaveFile file(dir + "/SKILL.md");
        if (!file.open(QIODevice::WriteOnly) || file.write(skill) < 0 || !file.commit())
            return fail("cannot write " + file.fileName());
        out(json ? QString("{\"installed\":\"%1\"}\n").arg(file.fileName()) : "Installed " + file.fileName() + "\n");
        return 0;
    }

    if (!QFile::exists(Backend::defaultDatabasePath()))
        return fail("no history yet at " + Backend::defaultDatabasePath());
    EngineHandle engine(Backend::defaultDatabasePath());
    const QString storeError = engine.call("status").data.toObject().value("storeError").toString();
    if (!storeError.isEmpty())
        return fail(storeError);
    auto query = [&](const QString &method, const QJsonObject &args = {}) {
        const auto reply = engine.call(method, args);
        if (!reply.ok)
            throw reply.error;
        return reply.data;
    };

    try {
        if (command == "stats") {
            const QJsonObject totals = query("totals").toObject();
            const QJsonArray bests = query("bests").toArray();
            const QJsonArray daily = query("daily", {{"days", 30}}).toArray();
            const QJsonArray keys = query("weakKeys", {{"limit", 10}, {"minSeen", 20}}).toArray();
            if (json) {
                out(QJsonDocument(QJsonObject{{"totals", totals}, {"bests", bests}, {"daily", daily}, {"weakKeys", keys}})
                        .toJson(QJsonDocument::Indented));
                return 0;
            }
            out(QString("%1 tests, %2 min typed, %3 keystrokes\n")
                    .arg(totals.value("tests").toInt())
                    .arg(totals.value("seconds").toDouble() / 60, 0, 'f', 1)
                    .arg(totals.value("keystrokes").toInt()));
            out(QString("average %1 wpm (last 10: %2), best %3 wpm, accuracy %4%\n\nPersonal bests\n")
                    .arg(totals.value("wpm").toDouble())
                    .arg(totals.value("last10Wpm").toDouble())
                    .arg(totals.value("best").toDouble())
                    .arg(totals.value("accuracy").toDouble()));
            for (const auto &b : bests)
                out(row(b.toObject()) + "\n");
            return 0;
        }
        if (command == "history") {
            const int index = int(arguments.indexOf("--limit"));
            const int limit = index > 0 ? arguments.value(index + 1).toInt() : 20;
            const QJsonArray tests = query("recent", {{"limit", limit > 0 ? limit : 20}}).toArray();
            if (json) {
                out(QJsonDocument(tests).toJson(QJsonDocument::Indented));
                return 0;
            }
            for (const auto &t : tests)
                out(row(t.toObject()) + "\n");
            return 0;
        }
        if (command == "test") {
            bool ok = false;
            const qint64 id = arguments.value(2).toLongLong(&ok);
            if (!ok)
                return fail("usage: omatype test <id> [--json]");
            const QJsonValue t = query("test", {{"id", id}});
            if (!t.isObject())
                return fail(QString("no test %1").arg(id));
            out(json ? QString(QJsonDocument(t.toObject()).toJson(QJsonDocument::Indented)) : row(t.toObject()) + "\n");
            return 0;
        }
    } catch (const QString &error) {
        return fail(error);
    }
    return fail("unknown command " + command);
}
