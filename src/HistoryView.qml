import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import "format.js" as Fmt

// Everything the history knows: totals, progression by day and by test,
// personal bests per kind of test, weak keys, and the recent tests.
// Up/Down (or j/k) pick a test, Enter opens it, Delete removes it.
FocusScope {
    id: view
    signal open(var id)
    signal remove(var test)

    property var totals: ({})
    property var daily: []
    property var runs: []
    property var bests: []
    property var weakKeys: []
    property var recent: []
    property int selected: 0

    function reload() {
        totals = backend.history("totals") || {}
        daily = backend.history("daily", { days: 90 }) || []
        runs = (backend.history("recent", { limit: 100 }) || []).slice().reverse()
        bests = backend.history("bests") || []
        weakKeys = backend.history("weakKeys", { limit: 8, minSeen: 20 }) || []
        recent = backend.history("recent", { limit: 50 }) || []
        selected = Math.max(0, Math.min(selected, recent.length - 1))
    }

    function select(i) {
        if (!recent.length) return
        selected = Math.max(0, Math.min(recent.length - 1, i))
        // Keep the picked row in view.
        const row = recentRepeater.itemAt(selected)
        if (!row) return
        const y = row.mapToItem(page, 0, 0).y
        if (y < flick.contentY + 40) flick.contentY = Math.max(0, y - 80)
        else if (y + row.height > flick.contentY + flick.height - 40)
            flick.contentY = Math.min(flick.contentHeight - flick.height, y + row.height - flick.height + 80)
    }

    onVisibleChanged: if (visible) reload()
    Connections {
        target: backend
        function onHistoryChanged() { if (view.visible) view.reload() }
    }

    Keys.onPressed: (event) => {
        switch (event.key) {
        case Qt.Key_Down: case Qt.Key_J: select(selected + 1); break
        case Qt.Key_Up: case Qt.Key_K: select(selected - 1); break
        case Qt.Key_Home: flick.contentY = 0; selected = 0; break
        case Qt.Key_End: select(recent.length - 1); break
        case Qt.Key_PageDown: flick.contentY = Math.min(Math.max(0, flick.contentHeight - flick.height), flick.contentY + flick.height * 0.8); break
        case Qt.Key_PageUp: flick.contentY = Math.max(0, flick.contentY - flick.height * 0.8); break
        case Qt.Key_Return: case Qt.Key_Enter: if (recent.length) view.open(recent[selected].id); break
        case Qt.Key_Delete: if (recent.length) view.remove(recent[selected]); break
        default: return
        }
        event.accepted = true
    }

    component Heading: Text {
        color: theme.foreground
        font.pixelSize: 17
        font.weight: Font.DemiBold
        Layout.topMargin: 12
    }

    component Card: Rectangle {
        radius: theme.radius
        color: theme.surface
    }

    Text {
        anchors.centerIn: parent
        visible: !(view.totals.tests > 0)
        horizontalAlignment: Text.AlignHCenter
        color: theme.muted
        font.pixelSize: 18
        text: backend.error !== "" ? backend.error : qsTr("No tests yet.\nPress Tab to start one.")
    }

    Flickable {
        id: flick
        anchors.fill: parent
        visible: view.totals.tests > 0
        contentWidth: width
        contentHeight: page.implicitHeight + 64
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar {}
        Behavior on contentY { NumberAnimation { duration: 120 } }

        ColumnLayout {
            id: page
            x: 40
            y: 24
            width: flick.width - 80
            spacing: 14

            Flow {
                Layout.fillWidth: true
                spacing: 44
                StatBlock { label: qsTr("tests"); value: Fmt.num(view.totals.tests) }
                StatBlock { label: qsTr("time typing"); value: Fmt.duration(view.totals.seconds) }
                StatBlock { label: qsTr("keystrokes"); value: Fmt.num(view.totals.keystrokes) }
                StatBlock {
                    label: qsTr("average")
                    value: qsTr("%1 wpm").arg(Fmt.num(view.totals.wpm))
                    detail: qsTr("last 10: %1").arg(Fmt.num(view.totals.last10Wpm))
                }
                StatBlock { label: qsTr("best"); value: qsTr("%1 wpm").arg(Fmt.num(view.totals.best)); valueColor: theme.accent }
                StatBlock {
                    label: qsTr("accuracy")
                    value: Fmt.num(view.totals.accuracy, 1) + "%"
                    detail: qsTr("last 10: %1%").arg(Fmt.num(view.totals.last10Accuracy, 1))
                }
                StatBlock { label: qsTr("consistency"); value: Fmt.num(view.totals.consistency) + "%" }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 20

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 1
                    Heading { text: qsTr("Progress by day") }
                    Text {
                        color: theme.muted
                        font.pixelSize: 12
                        text: qsTr("Average speed per day, best of the day as a dash, tests per day as bars")
                    }
                    Card {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 230
                        ProgressChart {
                            anchors.fill: parent
                            anchors.margins: 10
                            points: view.daily.map(d => ({
                                label: Fmt.day(d.day), value: d.wpm, best: d.best, count: d.tests,
                                tip: [Fmt.day(d.day), qsTr("%1 wpm average").arg(Fmt.num(d.wpm)),
                                      qsTr("%1 wpm best").arg(Fmt.num(d.best)),
                                      qsTr("%1% accuracy").arg(Fmt.num(d.accuracy, 1)),
                                      qsTr("%n tests", "", d.tests) + " · " + Fmt.duration(d.seconds)]
                            }))
                        }
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 1
                    Heading { text: qsTr("Last %n tests", "", view.runs.length) }
                    Text {
                        color: theme.muted
                        font.pixelSize: 12
                        text: qsTr("Each test as a dot, the line is the average of the last ten")
                    }
                    Card {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 230
                        ProgressChart {
                            anchors.fill: parent
                            anchors.margins: 10
                            runningAverage: true
                            points: view.runs.map(t => ({
                                label: new Date(t.startedAt).toLocaleDateString(Qt.locale(), "d MMM"), value: t.wpm,
                                tip: [Fmt.dateTime(t.startedAt), Fmt.modeLabel(t.mode, t.value),
                                      qsTr("%1 wpm").arg(Fmt.num(t.wpm)), qsTr("%1% accuracy").arg(Fmt.num(t.accuracy, 1))]
                            }))
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 20

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 3
                    Layout.alignment: Qt.AlignTop
                    Heading { text: qsTr("Personal bests") }
                    Card {
                        Layout.fillWidth: true
                        Layout.preferredHeight: bestsColumn.implicitHeight + 16
                        Column {
                            id: bestsColumn
                            x: 16
                            y: 8
                            width: parent.width - 32
                            Repeater {
                                model: view.bests
                                delegate: Item {
                                    required property var modelData
                                    width: bestsColumn.width
                                    height: 40
                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: parent.width * 0.42
                                        elide: Text.ElideRight
                                        color: theme.foreground
                                        font.pixelSize: 14
                                        text: Fmt.modeLabel(modelData.mode, modelData.value)
                                              + "  <font color=\"" + theme.muted + "\">" + Fmt.extras(modelData) + "</font>"
                                        textFormat: Text.StyledText
                                    }
                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        x: parent.width * 0.44
                                        color: theme.accent
                                        font.family: "monospace"
                                        font.pixelSize: 18
                                        text: Fmt.num(modelData.wpm, 1)
                                    }
                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        x: parent.width * 0.58
                                        color: theme.muted
                                        font.pixelSize: 13
                                        text: qsTr("%1% acc · %2 raw").arg(Fmt.num(modelData.accuracy, 1)).arg(Fmt.num(modelData.raw))
                                    }
                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.right: parent.right
                                        color: theme.muted
                                        font.pixelSize: 13
                                        text: qsTr("%n runs", "", modelData.runs)
                                    }
                                }
                            }
                        }
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 2
                    Layout.alignment: Qt.AlignTop
                    Heading { text: qsTr("Weak keys") }
                    Card {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.max(56, keysColumn.implicitHeight + 16)
                        Text {
                            anchors.centerIn: parent
                            visible: view.weakKeys.length === 0
                            color: theme.muted
                            font.pixelSize: 13
                            text: qsTr("Not enough keystrokes yet")
                        }
                        Column {
                            id: keysColumn
                            x: 16
                            y: 8
                            width: parent.width - 32
                            Repeater {
                                model: view.weakKeys
                                delegate: Item {
                                    required property var modelData
                                    width: keysColumn.width
                                    height: 32
                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: 36
                                        color: theme.foreground
                                        font.family: "monospace"
                                        font.pixelSize: 18
                                        text: modelData.key === " " ? "␣" : modelData.key
                                    }
                                    Rectangle {
                                        anchors.verticalCenter: parent.verticalCenter
                                        x: 40
                                        width: (parent.width - 150) * Math.min(1, modelData.rate / Math.max(1, view.weakKeys[0].rate))
                                        height: 8
                                        radius: Math.min(4, theme.radius)
                                        color: Qt.alpha(theme.error, 0.75)
                                    }
                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.right: parent.right
                                        color: theme.muted
                                        font.pixelSize: 13
                                        text: qsTr("%1% of %2").arg(Fmt.num(modelData.rate, 1)).arg(Fmt.num(modelData.seen))
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Heading { text: qsTr("Recent tests") }
            Card {
                Layout.fillWidth: true
                Layout.preferredHeight: recentColumn.implicitHeight + 12
                Column {
                    id: recentColumn
                    y: 6
                    width: parent.width
                    Repeater {
                        id: recentRepeater
                        model: view.recent
                        delegate: Rectangle {
                            required property var modelData
                            required property int index
                            width: recentColumn.width
                            height: 38
                            color: index === view.selected && view.activeFocus ? Qt.alpha(theme.accent, 0.16)
                                 : rowMouse.containsMouse ? Qt.alpha(theme.foreground, 0.05) : "transparent"
                            radius: Math.min(4, theme.radius)

                            Row {
                                anchors.verticalCenter: parent.verticalCenter
                                x: 16
                                spacing: 0
                                Text { width: 170; color: theme.muted; font.pixelSize: 13; text: Fmt.dateTime(modelData.startedAt) }
                                Text {
                                    width: 220; color: theme.foreground; font.pixelSize: 14; elide: Text.ElideRight
                                    text: Fmt.modeLabel(modelData.mode, modelData.value)
                                }
                                Text { width: 110; color: theme.accent; font.family: "monospace"; font.pixelSize: 15; text: qsTr("%1 wpm").arg(Fmt.num(modelData.wpm, 1)) }
                                Text { width: 100; color: theme.foreground; font.family: "monospace"; font.pixelSize: 14; text: qsTr("%1 raw").arg(Fmt.num(modelData.raw)) }
                                Text { width: 100; color: theme.foreground; font.family: "monospace"; font.pixelSize: 14; text: Fmt.num(modelData.accuracy, 1) + "%" }
                                Text { width: 100; color: theme.muted; font.family: "monospace"; font.pixelSize: 14; text: Fmt.num(modelData.consistency) + "%" }
                                Text { color: theme.muted; font.pixelSize: 13; text: Fmt.extras(modelData) }
                            }
                            MouseArea {
                                id: rowMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: { view.selected = index; view.forceActiveFocus() }
                                onDoubleClicked: view.open(modelData.id)
                            }
                        }
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                Layout.topMargin: 8
                horizontalAlignment: Text.AlignHCenter
                color: theme.muted
                font.pixelSize: 13
                text: qsTr("↑↓ pick · Enter open · Delete remove · Tab new test · Esc commands")
            }
        }
    }
}
