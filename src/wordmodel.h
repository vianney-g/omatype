#pragma once

#include <QAbstractListModel>
#include <QStringList>

// The words of the current test: what each asks for and what was typed into
// it. QML renders one delegate per word and colors it letter by letter.
class WordModel : public QAbstractListModel {
    Q_OBJECT

public:
    enum Role { TargetRole = Qt::UserRole + 1, TypedRole };

    using QAbstractListModel::QAbstractListModel;

    int rowCount(const QModelIndex &parent = {}) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

    void reset(const QStringList &targets);
    void append(const QStringList &targets);
    void setTyped(int index, const QString &typed);
    // Drops words past `count`; zen words exist only once typed.
    void truncate(int count);

    QString target(int index) const { return m_targets.value(index); }
    QString typed(int index) const { return m_typed.value(index); }

private:
    QStringList m_targets;
    QStringList m_typed;
};
