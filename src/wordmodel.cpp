#include "wordmodel.h"

int WordModel::rowCount(const QModelIndex &parent) const {
    return parent.isValid() ? 0 : int(m_targets.size());
}

QVariant WordModel::data(const QModelIndex &index, int role) const {
    if (!index.isValid() || index.row() >= m_targets.size())
        return {};
    switch (role) {
    case TargetRole: return m_targets.at(index.row());
    case TypedRole: return m_typed.at(index.row());
    }
    return {};
}

QHash<int, QByteArray> WordModel::roleNames() const {
    return {{TargetRole, "target"}, {TypedRole, "typed"}};
}

void WordModel::reset(const QStringList &targets) {
    beginResetModel();
    m_targets = targets;
    m_typed = QStringList();
    for (qsizetype i = 0; i < targets.size(); ++i)
        m_typed.append(QString());
    endResetModel();
}

void WordModel::append(const QStringList &targets) {
    if (targets.isEmpty())
        return;
    beginInsertRows({}, int(m_targets.size()), int(m_targets.size() + targets.size() - 1));
    for (const auto &t : targets) {
        m_targets.append(t);
        m_typed.append(QString());
    }
    endInsertRows();
}

void WordModel::setTyped(int index, const QString &typed) {
    if (index < 0)
        return;
    if (index >= m_targets.size()) {
        QStringList blanks;
        for (qsizetype i = m_targets.size(); i <= index; ++i)
            blanks.append(QString());
        append(blanks);
    }
    if (m_typed.at(index) == typed)
        return;
    m_typed[index] = typed;
    const auto i = createIndex(index, 0);
    emit dataChanged(i, i, {TypedRole});
}

void WordModel::truncate(int count) {
    if (count >= m_targets.size())
        return;
    beginRemoveRows({}, count, int(m_targets.size() - 1));
    m_targets.resize(count);
    m_typed.resize(count);
    endRemoveRows();
}
