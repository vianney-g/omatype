import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts

// Every shortcut, on F1 or Ctrl+/ (? is a letter you might be typing).
Popup {
    id: help
    Material.roundedScale: theme.radius
    modal: true
    dim: true
    Overlay.modal: Rectangle { color: Qt.alpha("black", 0.55) }
    focus: true
    x: Math.round((parent.width - width) / 2)
    y: Math.round((parent.height - height) / 2)
    padding: 28
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    background: Rectangle {
        radius: theme.radius
        color: theme.background
        border.color: Qt.alpha(theme.muted, 0.4)
    }

    readonly property var rows: [
        ["Tab", qsTr("New test (from anywhere)")],
        ["Esc", qsTr("Commands: modes, options, history")],
        ["↑  /  Shift+Tab", qsTr("Settings bar (before typing)")],
        ["←  →  Enter", qsTr("Move and pick in the settings bar")],
        ["Backspace", qsTr("Delete a letter (a wrong word can be reopened)")],
        ["Ctrl+Backspace", qsTr("Delete the word")],
        ["Shift+Enter", qsTr("End a zen test")],
        ["Ctrl+H", qsTr("History and stats")],
        ["↑  ↓  Enter  Delete", qsTr("Pick, open, remove a test in history")],
        ["F1  /  Ctrl+/", qsTr("This help")],
        ["Ctrl+Q", qsTr("Quit")]
    ]

    contentItem: ColumnLayout {
        spacing: 14
        Text {
            text: qsTr("Keyboard shortcuts")
            color: theme.foreground
            font.pixelSize: 20
            font.weight: Font.DemiBold
        }
        GridLayout {
            columns: 2
            columnSpacing: 28
            rowSpacing: 10
            Repeater {
                model: help.rows.length * 2
                delegate: Text {
                    required property int index
                    readonly property var row: help.rows[Math.floor(index / 2)]
                    text: index % 2 === 0 ? row[0] : row[1]
                    color: index % 2 === 0 ? theme.accent : theme.foreground
                    font.family: index % 2 === 0 ? "monospace" : Qt.application.font.family
                    font.pixelSize: 14
                }
            }
        }
    }
}
