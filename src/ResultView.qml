import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import "format.js" as Fmt

// A finished test: the headline figures, the per-second graph, and the rest.
FocusScope {
    id: view
    property var result: ({})
    property bool fromHistory: false
    signal next()
    signal back()

    readonly property bool hasResult: result && result.wpm !== undefined

    Keys.onPressed: (event) => {
        // Not Space: a test can end mid-word, and the next keystroke must not
        // skip the result.
        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
            view.next()
            event.accepted = true
        } else if ((event.key === Qt.Key_Backspace || event.key === Qt.Key_Left) && view.fromHistory) {
            view.back()
            event.accepted = true
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 40
        anchors.topMargin: 24
        spacing: 24
        visible: view.hasResult

        RowLayout {
            Layout.fillWidth: true
            spacing: 56

            ColumnLayout {
                spacing: 16
                Layout.alignment: Qt.AlignTop
                StatBlock {
                    big: true
                    label: qsTr("wpm")
                    value: Fmt.num(view.result.wpm)
                    valueColor: theme.accent
                }
                StatBlock {
                    big: true
                    label: qsTr("acc")
                    value: Fmt.num(view.result.accuracy) + "%"
                    valueColor: theme.accent
                }
                Rectangle {
                    visible: view.result.pb === true
                    implicitWidth: pbText.implicitWidth + 20
                    implicitHeight: pbText.implicitHeight + 10
                    radius: Math.min(4, theme.radius)
                    color: theme.accent
                    Text {
                        id: pbText
                        anchors.centerIn: parent
                        text: qsTr("New personal best (was %1)").arg(Fmt.num(view.result.previousBest || 0, 1))
                        color: theme.accentForeground
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                    }
                }
            }

            SpeedChart {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 220
                seconds: view.result.seconds || []
            }
        }

        Flow {
            Layout.fillWidth: true
            spacing: 44

            StatBlock {
                label: qsTr("test type")
                value: Fmt.modeLabel(view.result.mode, view.result.value)
                detail: view.hasResult && view.result.language !== undefined ? Fmt.extras(view.result) : ""
            }
            StatBlock {
                label: qsTr("raw")
                value: Fmt.num(view.result.raw)
            }
            StatBlock {
                label: qsTr("characters")
                value: [view.result.correct, view.result.incorrect, view.result.extra, view.result.missed]
                       .map(v => Fmt.num(v)).join("/")
                detail: qsTr("correct / incorrect / extra / missed")
            }
            StatBlock {
                label: qsTr("consistency")
                value: Fmt.num(view.result.consistency) + "%"
            }
            StatBlock {
                label: qsTr("time")
                value: Fmt.num(view.result.duration, view.result.duration < 100 ? 1 : 0) + " s"
                detail: view.result.startedAt ? Fmt.dateTime(view.result.startedAt) : ""
            }
            StatBlock {
                visible: !!view.result.quote
                label: qsTr("quote")
                value: view.result.quote || ""
            }
        }

        // What was typed, with its mistakes, as it looked on screen.
        Text {
            Layout.fillWidth: true
            LayoutMirroring.enabled: false
            visible: text !== ""
            wrapMode: Text.Wrap
            textFormat: Text.StyledText
            maximumLineCount: 4
            elide: Text.ElideRight
            font.family: "monospace"
            font.pixelSize: 15
            lineHeight: 1.3
            text: {
                const words = view.result.words || [], typed = view.result.typed || []
                const colors = { fg: theme.foreground, muted: theme.muted, error: theme.error,
                                 extra: Qt.darker(theme.error, theme.dark ? 1.5 : 0.75).toString() }
                const zen = view.result.mode === "zen"
                // The last word may have been cut off by the clock; it is not marked wrong.
                return typed.map((t, i) => Fmt.wordHtml(words[i] || "", t, i, typed.length - 1, zen, colors)).join(" ")
            }
        }

        Text {
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            color: theme.muted
            font.pixelSize: 13
            text: view.fromHistory
                  ? qsTr("Tab new test · Backspace back to history · Esc commands")
                  : qsTr("Tab or Enter next test · Ctrl+H history · Esc commands")
        }
    }
}
