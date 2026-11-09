import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import "format.js" as Fmt

ApplicationWindow {
    id: win
    width: 1180
    height: 760
    minimumWidth: 760
    minimumHeight: 520
    visible: true
    title: "omatype"
    color: theme.background

    Material.theme: theme.dark ? Material.Dark : Material.Light
    Material.accent: theme.accent
    Material.background: theme.background
    Material.foreground: theme.foreground
    Material.roundedScale: theme.radius

    // "test", "result", or "history".
    property string view: "test"
    property bool resultFromHistory: false
    readonly property bool overlayOpen: commandPalette.visible || help.visible || confirmDelete.visible

    function newTest() {
        commandPalette.close()
        help.close()
        backend.restart()
        view = "test"
        Qt.callLater(typing.focusText)
    }

    function showHistory() {
        view = "history"
        Qt.callLater(() => history.forceActiveFocus())
    }

    function toggleHistory() {
        if (view === "history") newTest()
        else showHistory()
    }

    function openTest(id) {
        backend.showResult(id)
        resultFromHistory = true
        view = "result"
        Qt.callLater(() => result.forceActiveFocus())
    }

    function focusCurrent() {
        if (view === "test") typing.focusText()
        else if (view === "result") result.forceActiveFocus()
        else history.forceActiveFocus()
    }

    Connections {
        target: backend
        function onFinished() {
            win.resultFromHistory = false
            win.view = "result"
            Qt.callLater(() => result.forceActiveFocus())
        }
        function onConfigChanged() { if (win.view !== "test") win.view = "test"; Qt.callLater(typing.focusText) }
    }

    Shortcut { sequence: "Tab"; context: Qt.ApplicationShortcut; enabled: !win.overlayOpen; onActivated: win.newTest() }
    Shortcut {
        sequence: "Esc"
        context: Qt.ApplicationShortcut
        enabled: !win.overlayOpen
        onActivated: configBar.activeFocus ? typing.focusText() : commandPalette.open()
    }
    Shortcut {
        sequence: "Shift+Tab"
        context: Qt.ApplicationShortcut
        enabled: !win.overlayOpen && win.view === "test" && !backend.running
        onActivated: configBar.enter()
    }
    Shortcut { sequence: "Ctrl+H"; context: Qt.ApplicationShortcut; enabled: !win.overlayOpen; onActivated: win.toggleHistory() }
    Shortcut { sequences: ["F1", "Ctrl+/"]; context: Qt.ApplicationShortcut; onActivated: help.visible ? help.close() : help.open() }
    Shortcut { sequences: [StandardKey.Quit, "Ctrl+Q"]; context: Qt.ApplicationShortcut; onActivated: Qt.quit() }

    // Right-to-left languages mirror the window; the Translator sets the
    // application's layout direction. The typed text itself stays left to right.
    Item {
        anchors.fill: parent
        LayoutMirroring.enabled: Qt.application.layoutDirection === Qt.RightToLeft
        LayoutMirroring.childrenInherit: true

        // Brand and view switcher.
        Row {
            id: header
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.margins: 24
            spacing: 18
            opacity: backend.running && win.view === "test" ? 0.25 : 1
            Behavior on opacity { NumberAnimation { duration: 200 } }

            Text {
                text: "omatype"
                color: theme.foreground
                font.family: "monospace"
                font.pixelSize: 24
                font.weight: Font.DemiBold
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: win.newTest() }
            }
            Text {
                anchors.baseline: parent.children[0].baseline
                text: qsTr("history")
                color: win.view === "history" ? theme.accent : historyMouse.containsMouse ? theme.foreground : theme.muted
                font.pixelSize: 15
                MouseArea {
                    id: historyMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: win.toggleHistory()
                }
            }
        }

        ConfigBar {
            id: configBar
            // Centered, unless that would run into the header.
            x: Math.max(header.x + header.width + 24, (parent.width - width) / 2)
            anchors.top: parent.top
            anchors.topMargin: 20
            visible: win.view === "test"
            opacity: backend.running ? 0 : 1
            Behavior on opacity { NumberAnimation { duration: 200 } }
            onDone: typing.focusText()
        }

        StackLayout {
            anchors.fill: parent
            anchors.topMargin: 76
            anchors.bottomMargin: 40
            currentIndex: win.view === "test" ? 0 : win.view === "result" ? 1 : 2

            TypingView {
                id: typing
                onUpFromText: configBar.enter()
            }
            ResultView {
                id: result
                result: backend.result
                fromHistory: win.resultFromHistory
                onNext: win.newTest()
                onBack: win.showHistory()
            }
            HistoryView {
                id: history
                onOpen: (id) => win.openTest(id)
                onRemove: (test) => { confirmDelete.test = test; confirmDelete.open() }
            }
        }

        // A problem with the history is shown, never hidden.
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: footer.top
            anchors.bottomMargin: 6
            visible: backend.error !== ""
            text: backend.error
            color: theme.error
            font.pixelSize: 13
        }

        Text {
            id: footer
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 14
            visible: win.view === "test"
            opacity: backend.running ? 0 : 1
            Behavior on opacity { NumberAnimation { duration: 200 } }
            color: theme.muted
            font.pixelSize: 13
            text: qsTr("Tab restart · Esc commands · ↑ settings · Ctrl+H history · F1 help")
        }
    }

    CommandPalette {
        id: commandPalette
        onClosed: win.focusCurrent()
        commands: {
            const c = []
            const mode = backend.mode, value = backend.value
            const add = (section, label, active, run, keys) => c.push({ section, label, active, run, keys: keys || "" })
            add(qsTr("Test"), qsTr("New test"), false, () => win.newTest(), "Tab")
            if (mode === "zen" && backend.running)
                add(qsTr("Test"), qsTr("End zen test"), false, () => backend.finishTest(), "Shift+Enter")
            add(qsTr("Test"), qsTr("History and stats"), win.view === "history", () => win.showHistory(), "Ctrl+H")
            for (const v of [15, 30, 60, 120])
                add(qsTr("Mode"), Fmt.modeLabel("time", v), mode === "time" && value === v, () => { backend.mode = "time"; backend.value = v })
            for (const v of [10, 25, 50, 100])
                add(qsTr("Mode"), Fmt.modeLabel("words", v), mode === "words" && value === v, () => { backend.mode = "words"; backend.value = v })
            for (const v of [0, 1, 2, 3])
                add(qsTr("Mode"), Fmt.modeLabel("quote", v), mode === "quote" && value === v, () => { backend.mode = "quote"; backend.value = v })
            add(qsTr("Mode"), Fmt.modeLabel("zen", 0), mode === "zen", () => backend.mode = "zen")
            add(qsTr("Options"), qsTr("Punctuation"), backend.punctuation, () => backend.punctuation = !backend.punctuation)
            add(qsTr("Options"), qsTr("Numbers"), backend.numbers, () => backend.numbers = !backend.numbers)
            add(qsTr("Options"), qsTr("Stop on error"), backend.stopOnError, () => backend.stopOnError = !backend.stopOnError)
            add(qsTr("Options"), qsTr("Confidence mode (no backspace)"), backend.confidence, () => backend.confidence = !backend.confidence)
            add(qsTr("Words"), qsTr("English"), backend.language === "en", () => backend.language = "en")
            add(qsTr("Words"), qsTr("French"), backend.language === "fr", () => backend.language = "fr")
            add(qsTr("App"), qsTr("Keyboard shortcuts"), false, () => help.open(), "F1")
            add(qsTr("App"), qsTr("Quit"), false, () => Qt.quit(), "Ctrl+Q")
            return c
        }
    }

    HelpOverlay {
        id: help
        onClosed: win.focusCurrent()
    }

    Dialog {
        id: confirmDelete
        property var test: null
        anchors.centerIn: parent
        modal: true
        Overlay.modal: Rectangle { color: Qt.alpha("black", 0.55) }
        title: qsTr("Delete this test?")
        Material.roundedScale: theme.radius
        standardButtons: Dialog.Yes | Dialog.No
        onOpened: standardButton(Dialog.No).forceActiveFocus()
        onAccepted: if (test) backend.deleteTest(test.id)
        onClosed: win.focusCurrent()
        Text {
            color: theme.foreground
            font.pixelSize: 14
            text: confirmDelete.test
                  ? qsTr("%1, %2 wpm, %3. Its keystrokes go with it.")
                        .arg(Fmt.modeLabel(confirmDelete.test.mode, confirmDelete.test.value))
                        .arg(Fmt.num(confirmDelete.test.wpm, 1))
                        .arg(Fmt.dateTime(confirmDelete.test.startedAt))
                  : ""
        }
    }

    Component.onCompleted: typing.focusText()
}
