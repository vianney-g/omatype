import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import "format.js" as Fmt

// The test itself: three lines of words, a caret, and the live figures.
// Keys go to a hidden text input so compose keys and dead keys (é, ê, ç)
// arrive as the letters they make.
Item {
    id: view

    property real fontSize: Math.max(22, Math.min(34, width / 34))
    // A property, not FontMetrics.advanceWidth(), so it follows the font size.
    readonly property real charWidth: charMetrics.advanceWidth
    readonly property real lineHeight: Math.round(fontSize * 1.7)
    readonly property bool zen: backend.mode === "zen"
    readonly property var colors: ({
        fg: theme.foreground,
        muted: Qt.alpha(theme.muted, 0.85).toString(),
        error: theme.error,
        extra: Qt.darker(theme.error, theme.dark ? 1.5 : 0.75).toString()
    })

    signal upFromText()

    function focusText() { input.forceActiveFocus() }

    FontMetrics {
        id: metrics
        font.family: "monospace"
        font.pixelSize: view.fontSize
    }
    TextMetrics {
        id: charMetrics
        font: metrics.font
        text: "m"
    }

    // The figure that matters during a test: time left, or words done.
    Row {
        id: status
        anchors.left: box.left
        anchors.bottom: box.top
        anchors.bottomMargin: 18
        spacing: 28

        Text {
            color: theme.accent
            font.family: "monospace"
            font.pixelSize: view.fontSize * 0.9
            text: {
                const p = backend.live.progress || {}
                if (backend.mode === "time") return p.remaining !== undefined ? Fmt.num(p.remaining) : Fmt.num(backend.value)
                if (view.zen) return Fmt.num(backend.current)
                return Fmt.num(p.done || 0) + "/" + Fmt.num(p.total || 0)
            }
        }
        Text {
            visible: backend.running
            color: theme.muted
            font.family: "monospace"
            font.pixelSize: view.fontSize * 0.9
            text: qsTr("%1 wpm").arg(Fmt.num(backend.live.wpm || 0))
        }
        Text {
            visible: backend.running
            color: theme.muted
            font.family: "monospace"
            font.pixelSize: view.fontSize * 0.9
            text: Fmt.num(backend.live.accuracy || 0) + "%"
        }
    }

    Item {
        id: box
        // Typing text always reads left to right, even in a mirrored window.
        LayoutMirroring.enabled: false
        LayoutMirroring.childrenInherit: true
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -view.lineHeight * 0.3
        width: Math.min(parent.width - 96, view.charWidth * 58)
        height: view.lineHeight * 3
        clip: true

        // The current word reports its line, which scrolls the text so the
        // caret stays on the middle line once past the first.
        property real currentY: 0
        readonly property int currentLine: Math.round(currentY / view.lineHeight)
        property real caretOpacity: 1

        // Blinks while waiting, stays solid while typing.
        SequentialAnimation on caretOpacity {
            running: !backend.started && input.activeFocus
            loops: Animation.Infinite
            NumberAnimation { to: 0; duration: 500; easing.type: Easing.InOutQuad }
            NumberAnimation { to: 1; duration: 500; easing.type: Easing.InOutQuad }
            onRunningChanged: if (!running) box.caretOpacity = 1
        }

        Flow {
            id: flow
            width: box.width
            spacing: view.charWidth
            y: -Math.max(0, box.currentLine - 1) * view.lineHeight
            Behavior on y { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }

            Repeater {
                model: backend.words
                delegate: Text {
                    id: word
                    required property int index
                    required property string target
                    required property string typed
                    readonly property bool isCurrent: index === backend.current
                    height: view.lineHeight
                    verticalAlignment: Text.AlignVCenter
                    textFormat: Text.StyledText
                    font: metrics.font
                    text: Fmt.wordHtml(target, typed, index, backend.current, view.zen, view.colors)

                    function report() { if (isCurrent) box.currentY = y }
                    onYChanged: report()
                    onIsCurrentChanged: report()
                    Component.onCompleted: report()

                    // The caret lives in the word it's in, so it can't lag
                    // behind the layout of the others.
                    Rectangle {
                        visible: word.isCurrent
                        width: 2.5
                        height: view.fontSize * 1.15
                        radius: 1
                        color: theme.accent
                        opacity: box.caretOpacity
                        anchors.verticalCenter: parent.verticalCenter
                        x: backend.caret * view.charWidth - 1
                        Behavior on x { enabled: backend.started; NumberAnimation { duration: 60 } }
                    }
                }
            }
        }

        MouseArea {
            anchors.fill: parent
            onClicked: input.forceActiveFocus()
        }
    }

    // Lost focus (a click elsewhere) is shown, not silently swallowing keys.
    Text {
        anchors.centerIn: box
        visible: !input.activeFocus && view.visible && !backend.running
        text: qsTr("Click or press Tab to focus")
        color: theme.foreground
        font.pixelSize: 16
        Rectangle {
            anchors.fill: parent
            anchors.margins: -12
            z: -1
            radius: Math.min(4, theme.radius)
            color: theme.surface
        }
    }

    Text {
        anchors.top: box.bottom
        anchors.topMargin: 18
        anchors.left: box.left
        width: box.width
        visible: backend.mode === "quote" && backend.quoteSource !== ""
        text: "— " + backend.quoteSource
        color: theme.muted
        font.pixelSize: 15
        elide: Text.ElideRight
    }

    Text {
        anchors.top: box.bottom
        anchors.topMargin: 18
        anchors.left: box.left
        visible: view.zen
        text: qsTr("Shift+Enter ends a zen test")
        color: theme.muted
        font.pixelSize: 15
    }

    Text {
        anchors.top: box.bottom
        anchors.topMargin: 18
        anchors.right: box.right
        visible: backend.confidence || backend.stopOnError
        text: [backend.stopOnError ? qsTr("stop on error") : "", backend.confidence ? qsTr("confidence mode") : ""]
              .filter(s => s !== "").join(" · ")
        color: theme.muted
        font.pixelSize: 15
    }

    TextInput {
        id: input
        width: 1
        height: 1
        opacity: 0
        focus: true
        inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase | Qt.ImhPreferLowercase
        onTextChanged: {
            if (text.length === 0) return
            const t = text
            text = ""
            backend.type(t)
        }
        Keys.onPressed: (event) => {
            if (event.key === Qt.Key_Backspace) {
                backend.backspace((event.modifiers & (Qt.ControlModifier | Qt.AltModifier)) !== 0)
                event.accepted = true
            } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                if (event.modifiers & Qt.ShiftModifier) backend.finishTest()
                event.accepted = true
            } else if (event.key === Qt.Key_Up && !backend.running) {
                view.upFromText()
                event.accepted = true
            } else if (event.modifiers & Qt.ControlModifier) {
                // Ctrl+A, Ctrl+V and the like would edit the hidden field.
                event.accepted = true
            }
        }
    }
}
