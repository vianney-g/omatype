import QtQuick

// The values under the pointer, kept inside the chart.
Rectangle {
    id: tip
    property var lines: []
    property real anchorX: 0
    property real boundsWidth: 0

    x: Math.max(0, Math.min(boundsWidth - width, anchorX + 12 + width > boundsWidth ? anchorX - width - 12 : anchorX + 12))
    width: column.implicitWidth + 20
    height: column.implicitHeight + 14
    radius: Math.min(4, theme.radius)
    color: theme.background
    border.color: Qt.alpha(theme.muted, 0.4)

    Column {
        id: column
        anchors.centerIn: parent
        spacing: 2
        Repeater {
            model: tip.lines
            Text {
                required property string modelData
                required property int index
                text: modelData
                color: index === 0 ? theme.muted : theme.foreground
                font.pixelSize: 13
            }
        }
    }
}
