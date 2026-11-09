import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import "format.js" as Fmt

// The test settings in one line. Reachable from the keyboard: Up (or
// Shift+Tab) from the text, Left/Right to move, Enter or Space to pick,
// Down or Escape to go back to typing.
FocusScope {
    id: bar
    implicitWidth: row.implicitWidth + 24
    implicitHeight: 44

    property int cursor: 0
    signal done()

    readonly property bool wordsLike: backend.mode === "time" || backend.mode === "words"

    readonly property var entries: {
        const e = []
        const add = (label, active, run, show, group) => { if (show !== false) e.push({ label, active, run, group, separator: false }) }
        const sep = () => e.push({ separator: true })
        add(qsTr("punctuation"), backend.punctuation, () => backend.punctuation = !backend.punctuation, wordsLike)
        add(qsTr("numbers"), backend.numbers, () => backend.numbers = !backend.numbers, wordsLike)
        if (wordsLike) sep()
        for (const m of ["time", "words", "quote", "zen"])
            add(m === "time" ? qsTr("time") : m === "words" ? qsTr("words") : m === "quote" ? qsTr("quote") : qsTr("zen"),
                backend.mode === m, () => backend.mode = m, true, "mode")
        if (backend.mode !== "zen") sep()
        const values = backend.mode === "time" ? [15, 30, 60, 120] : backend.mode === "words" ? [10, 25, 50, 100]
                     : backend.mode === "quote" ? [0, 1, 2, 3] : []
        for (const v of values)
            add(backend.mode === "quote" ? Fmt.quoteLength(v) : Fmt.num(v), backend.value === v, () => backend.value = v)
        sep()
        add(qsTr("stop on error"), backend.stopOnError, () => backend.stopOnError = !backend.stopOnError)
        add(qsTr("confidence"), backend.confidence, () => backend.confidence = !backend.confidence)
        sep()
        add("en", backend.language === "en", () => backend.language = "en")
        add("fr", backend.language === "fr", () => backend.language = "fr")
        return e
    }

    readonly property var selectable: entries.map((e, i) => e.separator ? -1 : i).filter(i => i >= 0)

    function enter() {
        // Start on the active mode, where most changes happen.
        const modeIndex = entries.findIndex(e => e.group === "mode" && e.active)
        cursor = Math.max(0, selectable.indexOf(modeIndex))
        forceActiveFocus()
    }

    function move(step) {
        cursor = (cursor + step + selectable.length) % selectable.length
    }

    Keys.onPressed: (event) => {
        switch (event.key) {
        case Qt.Key_Left: case Qt.Key_H: move(-1); break
        case Qt.Key_Right: case Qt.Key_L: move(1); break
        case Qt.Key_Return: case Qt.Key_Enter: case Qt.Key_Space: {
            const before = cursor
            entries[selectable[cursor]].run()
            cursor = Math.min(before, selectable.length - 1)
            break
        }
        case Qt.Key_Down: case Qt.Key_J: bar.done(); break
        default: return
        }
        event.accepted = true
    }

    Rectangle {
        anchors.fill: parent
        radius: theme.radius
        color: theme.surface
        border.width: bar.activeFocus ? 1 : 0
        border.color: Qt.alpha(theme.accent, 0.6)
    }

    Row {
        id: row
        anchors.centerIn: parent
        spacing: 2

        Repeater {
            model: bar.entries
            delegate: Item {
                required property var modelData
                required property int index
                readonly property bool focused: bar.activeFocus && bar.selectable[bar.cursor] === index
                width: modelData.separator ? 14 : label.implicitWidth + 16
                height: 32

                Rectangle {
                    visible: modelData.separator
                    anchors.centerIn: parent
                    width: 1
                    height: 16
                    color: Qt.alpha(theme.muted, 0.5)
                }
                Rectangle {
                    visible: parent.focused
                    anchors.fill: parent
                    radius: Math.min(4, theme.radius)
                    color: Qt.alpha(theme.accent, 0.16)
                }
                Text {
                    id: label
                    visible: !modelData.separator
                    anchors.centerIn: parent
                    text: modelData.label || ""
                    color: modelData.active ? theme.accent : mouse.containsMouse || parent.focused ? theme.foreground : theme.muted
                    font.pixelSize: 14
                }
                MouseArea {
                    id: mouse
                    anchors.fill: parent
                    enabled: !modelData.separator
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: modelData.run()
                }
            }
        }
    }
}
