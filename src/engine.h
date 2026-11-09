#pragma once

#include <QJsonObject>
#include <QJsonValue>
#include <QString>

struct Engine;

// Owns the Rust engine and turns its JSON calls into Qt values.
class EngineHandle {
public:
    explicit EngineHandle(const QString &databasePath);
    ~EngineHandle();
    EngineHandle(const EngineHandle &) = delete;
    EngineHandle &operator=(const EngineHandle &) = delete;

    struct Reply {
        bool ok = false;
        QJsonValue data;
        QString error;
    };

    Reply call(const QString &method, const QJsonObject &args = {}) const;

private:
    Engine *m_engine;
};
