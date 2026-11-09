import QtQuick
import "format.js" as Fmt

// One test, second by second: net speed (accent), raw speed (muted), and
// errors as crosses on their own scale at the padRight. Hover shows the values.
Item {
    id: chart
    property var seconds: []
    property int hover: -1

    readonly property real padLeft: 48
    readonly property real padRight: 40
    readonly property real padTop: 12
    readonly property real padBottom: 30
    readonly property real plotWidth: width - padLeft - padRight
    readonly property real plotHeight: height - padTop - padBottom
    readonly property real maxSpeed: Fmt.niceMax(Math.max(10, ...seconds.map(s => Math.max(s.wpm, s.raw))) * 1.05)
    readonly property int maxErrors: Math.max(2, ...seconds.map(s => s.errors))
    readonly property real span: seconds.length ? seconds[seconds.length - 1].t : 1

    function xAt(t) { return padLeft + (seconds.length > 1 ? (t - seconds[0].t) / (span - seconds[0].t) : 0.5) * plotWidth }
    function yAt(v) { return padTop + plotHeight * (1 - v / maxSpeed) }
    function yError(e) { return padTop + plotHeight * (1 - e / maxErrors) }

    onSecondsChanged: canvas.requestPaint()
    onWidthChanged: canvas.requestPaint()
    onHeightChanged: canvas.requestPaint()
    Connections { target: theme; function onChanged() { canvas.requestPaint() } }

    Canvas {
        id: canvas
        anchors.fill: parent
        onPaint: {
            const ctx = getContext("2d")
            ctx.reset()
            const s = chart.seconds
            ctx.font = "12px sans-serif"
            ctx.lineWidth = 1

            // Grid and the speed axis.
            ctx.textAlign = "right"
            ctx.textBaseline = "middle"
            for (let i = 0; i <= 4; ++i) {
                const v = chart.maxSpeed * i / 4, y = chart.yAt(v)
                ctx.strokeStyle = Qt.alpha(theme.muted, 0.18)
                ctx.beginPath(); ctx.moveTo(chart.padLeft, y); ctx.lineTo(chart.width - chart.padRight, y); ctx.stroke()
                ctx.fillStyle = theme.muted
                ctx.fillText(Fmt.num(v), chart.padLeft - 10, y)
            }
            // The errors axis.
            ctx.textAlign = "left"
            for (let e = 0; e <= chart.maxErrors; e += Math.max(1, Math.ceil(chart.maxErrors / 4))) {
                ctx.fillStyle = Qt.alpha(theme.error, 0.8)
                ctx.fillText(Fmt.num(e), chart.width - chart.padRight + 10, chart.yError(e))
            }
            if (!s.length) return

            // Seconds along the padBottom, about every 60 px.
            ctx.textAlign = "center"
            ctx.textBaseline = "top"
            ctx.fillStyle = theme.muted
            const every = Math.max(1, Math.ceil(s.length / Math.max(1, chart.plotWidth / 60)))
            for (let i = 0; i < s.length; i += every)
                ctx.fillText(Fmt.num(Math.round(s[i].t)), chart.xAt(s[i].t), chart.height - chart.padBottom + 8)

            const line = (key, color, width) => {
                ctx.strokeStyle = color
                ctx.lineWidth = width
                ctx.lineJoin = "round"
                ctx.beginPath()
                s.forEach((p, i) => { const x = chart.xAt(p.t), y = chart.yAt(p[key]); i ? ctx.lineTo(x, y) : ctx.moveTo(x, y) })
                ctx.stroke()
            }
            // Soft fill under net speed, then the lines on padTop.
            ctx.fillStyle = Qt.alpha(theme.accent, 0.08)
            ctx.beginPath()
            ctx.moveTo(chart.xAt(s[0].t), chart.yAt(0))
            s.forEach(p => ctx.lineTo(chart.xAt(p.t), chart.yAt(p.wpm)))
            ctx.lineTo(chart.xAt(s[s.length - 1].t), chart.yAt(0))
            ctx.closePath()
            ctx.fill()
            line("raw", Qt.alpha(theme.muted, 0.9), 1.5)
            line("wpm", theme.accent, 2.5)

            ctx.strokeStyle = theme.error
            ctx.lineWidth = 2
            for (const p of s) {
                if (!p.errors) continue
                const x = chart.xAt(p.t), y = chart.yError(p.errors), r = 4
                ctx.beginPath()
                ctx.moveTo(x - r, y - r); ctx.lineTo(x + r, y + r)
                ctx.moveTo(x + r, y - r); ctx.lineTo(x - r, y + r)
                ctx.stroke()
            }

            if (chart.hover >= 0 && chart.hover < s.length) {
                const x = chart.xAt(s[chart.hover].t)
                ctx.strokeStyle = Qt.alpha(theme.foreground, 0.35)
                ctx.lineWidth = 1
                ctx.beginPath(); ctx.moveTo(x, chart.padTop); ctx.lineTo(x, chart.padTop + chart.plotHeight); ctx.stroke()
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onPositionChanged: (mouse) => {
            if (!chart.seconds.length) return
            let best = 0
            for (let i = 1; i < chart.seconds.length; ++i)
                if (Math.abs(chart.xAt(chart.seconds[i].t) - mouse.x) < Math.abs(chart.xAt(chart.seconds[best].t) - mouse.x)) best = i
            chart.hover = best
            canvas.requestPaint()
        }
        onExited: { chart.hover = -1; canvas.requestPaint() }
    }

    ChartTip {
        visible: chart.hover >= 0 && chart.hover < chart.seconds.length
        anchorX: visible ? chart.xAt(chart.seconds[chart.hover].t) : 0
        boundsWidth: chart.width
        y: chart.padTop
        lines: {
            if (!visible) return []
            const p = chart.seconds[chart.hover]
            return [qsTr("%1 s").arg(Fmt.num(p.t, p.t % 1 ? 1 : 0)),
                    qsTr("%1 wpm").arg(Fmt.num(p.wpm)),
                    qsTr("%1 raw").arg(Fmt.num(p.raw)),
                    qsTr("%n errors", "", p.errors)]
        }
    }
}
