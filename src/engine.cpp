#include "engine.h"

#include <QJsonDocument>

#include "../engine/include/omatype_engine.h"

EngineHandle::EngineHandle(const QString &databasePath)
    : m_engine(omt_open(databasePath.toUtf8().constData())) {}

EngineHandle::~EngineHandle() { omt_close(m_engine); }

EngineHandle::Reply EngineHandle::call(const QString &method, const QJsonObject &args) const {
    char *raw = omt_call(m_engine, method.toUtf8().constData(),
                         QJsonDocument(args).toJson(QJsonDocument::Compact).constData());
    Reply reply;
    if (!raw) {
        reply.error = QStringLiteral("engine returned nothing");
        return reply;
    }
    const QJsonObject object = QJsonDocument::fromJson(QByteArray(raw)).object();
    omt_free(raw);
    reply.ok = object.value("ok").toBool();
    reply.data = object.value("data");
    reply.error = object.value("error").toString();
    return reply;
}
