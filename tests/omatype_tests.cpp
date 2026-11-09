#include <QDir>
#include <QFile>
#include <QSignalSpy>
#include <QTemporaryDir>
#include <QtTest>

#include "backend.h"
#include "theme.h"
#include "translator.h"

class OmatypeTests : public QObject {
    Q_OBJECT

private slots:
    void wordsTestFinishesAndIsSaved()
    {
        QTemporaryDir dir;
        Backend backend(dir.filePath("h.db"), dir.filePath("s.ini"));
        double now = 0;
        backend.setClock([&] { return now; });
        backend.setMode("words");
        backend.setValue(10);
        QSignalSpy finished(&backend, &Backend::finished);

        typeAll(backend, now);
        QCOMPARE(finished.count(), 1);
        QCOMPARE(backend.result().value("accuracy").toDouble(), 100.0);
        QVERIFY(backend.result().value("wpm").toDouble() > 0);
        QCOMPARE(backend.history("totals").toMap().value("tests").toInt(), 1);
        const auto recent = backend.history("recent", {{"limit", 5}}).toList();
        QCOMPARE(recent.size(), 1);
        QVERIFY(backend.deleteTest(recent.first().toMap().value("id").toLongLong()));
        QCOMPARE(backend.history("totals").toMap().value("tests").toInt(), 0);
    }

    void timeTestEndsOnTheClock()
    {
        QTemporaryDir dir;
        Backend backend(dir.filePath("h.db"), dir.filePath("s.ini"));
        double now = 1000;
        backend.setClock([&] { return now; });
        backend.setMode("time");
        backend.setValue(15);
        backend.type(backend.wordModel().target(0).left(1));
        QVERIFY(backend.running());
        now += 16000;
        backend.tick();
        QVERIFY(!backend.running());
        QCOMPARE(backend.result().value("duration").toDouble(), 15.0);
    }

    void caretFollowsTheCurrentWord()
    {
        QTemporaryDir dir;
        Backend backend(dir.filePath("h.db"), dir.filePath("s.ini"));
        backend.setMode("zen");
        backend.type("hello wo");
        QCOMPARE(backend.current(), 1);
        QCOMPARE(backend.caret(), 2);
        backend.backspace(false);
        backend.backspace(false);
        backend.backspace(false);
        QCOMPARE(backend.current(), 0);
        QCOMPARE(backend.caret(), 5);
    }

    void settingsPersist()
    {
        QTemporaryDir dir;
        {
            Backend backend(dir.filePath("h.db"), dir.filePath("s.ini"));
            backend.setMode("quote");
            backend.setPunctuation(true);
            backend.setLanguage("fr");
        }
        Backend backend(dir.filePath("h.db"), dir.filePath("s.ini"));
        QCOMPARE(backend.mode(), QString("quote"));
        QVERIFY(backend.punctuation());
        QCOMPARE(backend.language(), QString("fr"));
        QVERIFY(!backend.quoteSource().isEmpty());
    }

    void unwritableHistoryIsReported()
    {
        QTemporaryDir dir;
        write(dir.filePath("file"), "not a directory");
        Backend backend(dir.filePath("file/h.db"), dir.filePath("s.ini"));
        QVERIFY(!backend.error().isEmpty());
        QVERIFY(backend.wordModel().rowCount() > 0);
    }

    void themeFallsBack()
    {
        QTemporaryDir dir;
        Theme theme(dir.path());
        QCOMPARE(theme.accent(), QString("#FFD60A"));
        QCOMPARE(theme.accentForeground(), QString("black"));
        QVERIFY(theme.dark());
    }

    void themeReadsPalette()
    {
        QTemporaryDir dir;
        QDir(dir.path()).mkpath("theme");
        write(dir.path() + "/theme/colors.toml",
              "accent = \"#112233\"\nbackground = \"#f0f0f0\"\nforeground = \"#202020\"\nmuted = \"#808080\"\n"
              "red = \"#aa0000\"\nmode = \"light\"\n");
        Theme theme(dir.path());
        QCOMPARE(theme.accent(), QString("#112233"));
        QCOMPARE(theme.accentForeground(), QString("white"));
        QCOMPARE(theme.background(), QString("#f0f0f0"));
        QCOMPARE(theme.error(), QString("#aa0000"));
        QVERIFY(!theme.dark());
    }

    void themeFollowsHyprlandRounding()
    {
        QTemporaryDir dir;
        const QString response = dir.filePath("response.json"), command = dir.filePath("hyprctl");
        write(command, "#!/bin/sh\ncat '" + response.toUtf8() + "'\n");
        QVERIFY(QFile::setPermissions(command, QFile::ReadOwner | QFile::WriteOwner | QFile::ExeOwner));
        write(response, "{\"int\":0}");

        Theme theme(dir.path(), nullptr, command);
        QCOMPARE(theme.radius(), 0);
        write(response, "{\"int\":6}");
        QTRY_COMPARE(theme.radius(), 6);
    }

    void translatorFallsBackToEnglish()
    {
        QTemporaryDir dir;
        Translator translator(dir.path() + "/language", dir.path());
        QCOMPARE(translator.language(), QString("en"));
        QVERIFY(translator.translate("Main", "Save").isNull());
    }

    void frenchCatalogCoversTheWindow()
    {
        QTemporaryDir dir;
        write(dir.path() + "/language", "fr\n");
        Translator translator(dir.path() + "/language");
        QCOMPARE(translator.language(), QString("fr"));
        QCOMPARE(translator.translate("Main", "history"), QString("historique"));
        QCOMPARE(translator.translate("Main", "%n tests", nullptr, 2), QString("%n tests"));
    }

    void translatorPicksPluralForms()
    {
        QTemporaryDir dir;
        write(dir.path() + "/language", "ru\n");
        write(dir.path() + "/en.json", R"({"%n clips": {"one": "%n clip", "other": "%n clips"}})");
        write(dir.path() + "/ru.json", R"({"%n clips": {"one": "%n клип", "few": "%n клипа", "many": "%n клипов", "other": "%n клипа"}})");

        Translator translator(dir.path() + "/language", dir.path());
        QCOMPARE(translator.translate("Main", "%n clips", nullptr, 21), QString("%n клип"));
        QCOMPARE(translator.translate("Main", "%n clips", nullptr, 25), QString("%n клипов"));
    }

private:
    static void typeAll(Backend &backend, double &now)
    {
        while (!backend.started() || backend.running()) {
            const QString word = backend.wordModel().target(backend.current());
            for (const QChar c : word + " ") {
                now += 120;
                backend.type(QString(c));
                if (!backend.running())
                    return;
            }
        }
    }

    static void write(const QString &path, const QByteArray &contents)
    {
        QFile file(path);
        QVERIFY(file.open(QIODevice::WriteOnly));
        file.write(contents);
    }
};

QTEST_MAIN(OmatypeTests)
#include "omatype_tests.moc"
