import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts

// Every setting and view behind Escape: type a few letters of what you want,
// Up/Down (or Ctrl+J/K) to pick, Enter to run it.
Popup {
    id: commandPalette
    property var commands: []   // [{section, label, active, run}]
    readonly property var matches: {
        const q = field.text.toLowerCase().trim()
        if (!q) return commands
        // Labels containing the words typed come first, then commands whose
        // section and label hold the letters in order, like fuzzy finders.
        const compact = q.replace(/\s+/g, "")
        const scored = []
        commands.forEach((c, i) => {
            const label = c.label.toLowerCase(), hay = (c.section + " " + c.label).toLowerCase()
            let score = -1
            if (label.startsWith(q)) score = 0
            else if (label.includes(q)) score = 1
            else if (hay.includes(q)) score = 2
            else {
                let k = 0
                for (const ch of hay) if (ch === compact[k]) ++k
                if (k === compact.length) score = 3
            }
            if (score >= 0) scored.push({ c, score, i })
        })
        scored.sort((a, b) => a.score - b.score || a.i - b.i)
        return scored.map(s => s.c)
    }
    property int current: 0

    Material.roundedScale: theme.radius
    modal: true
    focus: true
    dim: true
    Overlay.modal: Rectangle { color: Qt.alpha("black", 0.55) }
    x: Math.round((parent.width - width) / 2)
    y: Math.round(parent.height * 0.14)
    width: Math.min(560, parent.width - 48)
    height: Math.min(parent.height * 0.7, 64 + list.contentHeight + 16)
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    onAboutToShow: { field.text = ""; current = 0; field.forceActiveFocus() }
    onMatchesChanged: current = Math.min(current, Math.max(0, matches.length - 1))

    function run(i) {
        const c = matches[i]
        if (!c) return
        close()
        c.run()
    }

    background: Rectangle {
        radius: theme.radius
        color: theme.background
        border.color: Qt.alpha(theme.accent, 0.5)
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // A plain line rather than a Material field, whose floating label
        // fights with a popup this small.
        Rectangle {
            Layout.fillWidth: true
            Layout.margins: 12
            implicitHeight: 44
            radius: Math.min(6, theme.radius)
            color: theme.surface

            Text {
                anchors.fill: field
                verticalAlignment: Text.AlignVCenter
                visible: field.text === ""
                text: qsTr("Type a command")
                color: theme.muted
                font: field.font
            }
            TextInput {
                id: field
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 14
                verticalAlignment: TextInput.AlignVCenter
                color: theme.foreground
                selectionColor: Qt.alpha(theme.accent, 0.4)
                font.pixelSize: 16
                clip: true
                Keys.onPressed: (event) => {
                    const ctrl = event.modifiers & Qt.ControlModifier
                    if (event.key === Qt.Key_Down || event.key === Qt.Key_Tab || (ctrl && event.key === Qt.Key_J)) {
                        commandPalette.current = Math.min(commandPalette.matches.length - 1, commandPalette.current + 1)
                    } else if (event.key === Qt.Key_Up || event.key === Qt.Key_Backtab || (ctrl && event.key === Qt.Key_K)) {
                        commandPalette.current = Math.max(0, commandPalette.current - 1)
                    } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                        commandPalette.run(commandPalette.current)
                    } else {
                        return
                    }
                    list.positionViewAtIndex(commandPalette.current, ListView.Contain)
                    event.accepted = true
                }
            }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.bottomMargin: 8
            clip: true
            model: commandPalette.matches
            currentIndex: commandPalette.current
            boundsBehavior: Flickable.StopAtBounds
            delegate: Item {
                required property var modelData
                required property int index
                // Each section is titled above its first command.
                readonly property bool first: index === 0 || commandPalette.matches[index - 1].section !== modelData.section
                width: list.width
                height: 34 + (first ? 26 : 0)

                Text {
                    visible: parent.first
                    x: 18
                    y: 8
                    text: modelData.section
                    color: theme.muted
                    font.pixelSize: 12
                }
                Item {
                    id: rowArea
                    width: parent.width
                    height: 34
                    anchors.bottom: parent.bottom
                    Rectangle {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 8
                    radius: Math.min(4, theme.radius)
                    color: index === commandPalette.current ? Qt.alpha(theme.accent, 0.18) : "transparent"
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    x: 22
                    text: modelData.label
                    color: modelData.active ? theme.accent : theme.foreground
                    font.pixelSize: 15
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.right: parent.right
                    anchors.rightMargin: 22
                    text: modelData.active ? "●" : (modelData.keys || "")
                    color: modelData.active ? theme.accent : theme.muted
                    font.pixelSize: modelData.active ? 10 : 13
                }
                }
                MouseArea {
                    anchors.fill: rowArea
                    hoverEnabled: true
                    onEntered: commandPalette.current = index
                    onClicked: commandPalette.run(index)
                }
            }
        }
    }
}
