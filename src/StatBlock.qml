import QtQuick

// A label over a figure, as the result and history pages show them.
Column {
    property string label
    property string value
    property string detail
    property bool big: false
    property color valueColor: theme.foreground
    spacing: big ? 0 : 2

    Text {
        text: parent.label
        color: theme.muted
        font.pixelSize: parent.big ? 18 : 13
    }
    Text {
        text: parent.value
        color: parent.valueColor
        font.family: "monospace"
        font.pixelSize: parent.big ? 56 : 24
        font.weight: parent.big ? Font.Medium : Font.Normal
    }
    Text {
        visible: text !== ""
        text: parent.detail
        color: theme.muted
        font.pixelSize: 12
    }
}
