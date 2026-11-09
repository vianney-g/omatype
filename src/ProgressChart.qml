import QtQuick
import "format.js" as Fmt

// Progression over time. Each point is a day (average and best speed, with
// the number of tests as faint bars) or a single test (its speed, with a
// running average of the last ten).
Item {
    id: chart
    // [{label, value, best?, count?, tip: [lines]}], oldest first.
    property var points: []
    property string averageLabel: ""
    property bool runningAverage: false
    property int hover: -1

    readonly property real padLeft: 48
    readonly property real padRight: 16
    readonly property real padTop: 12
    readonly property real padBottom: 30
    readonly property real plotWidth: width - padLeft - padRight
    readonly property real plotHeight: height - padTop - padBottom
    readonly property real maxSpeed: Fmt.niceMax(Math.max(10, ...points.map(p => Math.max(p.value, p.best || 0))) * 1.05)
    readonly property int maxCount: Math.max(1, ...points.map(p => p.count || 0))
    readonly property var smoothed: {
        if (!runningAverage) return []
        const out = []
        for (let i = 0; i < points.length; ++i) {
            const from = Math.max(0, i - 9)
            let sum = 0
            for (let j = from; j <= i; ++j) sum += points[j].value
            out.push(sum / (i - from + 1))
        }
        return out
    }

    function xAt(i) { return padLeft + (points.length > 1 ? i / (points.length - 1) : 0.5) * plotWidth }
    function yAt(v) { return padTop + plotHeight * (1 - v / maxSpeed) }

    onPointsChanged: canvas.requestPaint()
    onWidthChanged: canvas.requestPaint()
    onHeightChanged: canvas.requestPaint()
    Connections { target: theme; function onChanged() { canvas.requestPaint() } }

    Canvas {
        id: canvas
        anchors.fill: parent
        onPaint: {
            const ctx = getContext("2d")
            ctx.reset()
            const p = chart.points
            ctx.font = "12px sans-serif"
            ctx.textAlign = "right"
            ctx.textBaseline = "middle"
            ctx.lineWidth = 1
            for (let i = 0; i <= 4; ++i) {
                const v = chart.maxSpeed * i / 4, y = chart.yAt(v)
                ctx.strokeStyle = Qt.alpha(theme.muted, 0.18)
                ctx.beginPath(); ctx.moveTo(chart.padLeft, y); ctx.lineTo(chart.width - chart.padRight, y); ctx.stroke()
                ctx.fillStyle = theme.muted
                ctx.fillText(Fmt.num(v), chart.padLeft - 10, y)
            }
            if (!p.length) return

            const slot = chart.plotWidth / Math.max(1, p.length)
            // Tests per day as faint bars along the padBottom third.
            p.forEach((pt, i) => {
                if (!pt.count) return
                const h = chart.plotHeight * 0.33 * pt.count / chart.maxCount, w = Math.max(2, Math.min(18, slot * 0.6))
                ctx.fillStyle = Qt.alpha(theme.accent, i === chart.hover ? 0.3 : 0.14)
                ctx.fillRect(chart.xAt(i) - w / 2, chart.padTop + chart.plotHeight - h, w, h)
            })

            ctx.textAlign = "center"
            ctx.textBaseline = "top"
            ctx.fillStyle = theme.muted
            const every = Math.max(1, Math.ceil(p.length / Math.max(1, chart.plotWidth / 70)))
            for (let i = 0; i < p.length; i += every)
                ctx.fillText(p[i].label, chart.xAt(i), chart.height - chart.padBottom + 8)

            const dots = p.length <= 60
            if (chart.runningAverage) {
                // Each test as a dot, the trend as the line.
                p.forEach((pt, i) => {
                    ctx.fillStyle = Qt.alpha(theme.foreground, i === chart.hover ? 0.9 : 0.35)
                    ctx.beginPath(); ctx.arc(chart.xAt(i), chart.yAt(pt.value), 2.5, 0, 2 * Math.PI); ctx.fill()
                })
                ctx.strokeStyle = theme.accent
                ctx.lineWidth = 2.5
                ctx.beginPath()
                chart.smoothed.forEach((v, i) => i ? ctx.lineTo(chart.xAt(i), chart.yAt(v)) : ctx.moveTo(chart.xAt(i), chart.yAt(v)))
                ctx.stroke()
            } else {
                // Best of the day as small marks above the average line.
                ctx.fillStyle = Qt.alpha(theme.foreground, 0.5)
                p.forEach((pt, i) => { if (pt.best) ctx.fillRect(chart.xAt(i) - 4, chart.yAt(pt.best) - 1, 8, 2) })
                ctx.strokeStyle = theme.accent
                ctx.lineWidth = 2.5
                ctx.beginPath()
                p.forEach((pt, i) => i ? ctx.lineTo(chart.xAt(i), chart.yAt(pt.value)) : ctx.moveTo(chart.xAt(i), chart.yAt(pt.value)))
                ctx.stroke()
                if (dots) {
                    ctx.fillStyle = theme.accent
                    p.forEach((pt, i) => { ctx.beginPath(); ctx.arc(chart.xAt(i), chart.yAt(pt.value), i === chart.hover ? 5 : 3.5, 0, 2 * Math.PI); ctx.fill() })
                }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onPositionChanged: (mouse) => {
            if (!chart.points.length) return
            const i = Math.round((mouse.x - chart.padLeft) / Math.max(1, chart.plotWidth) * (chart.points.length - 1))
            chart.hover = Math.max(0, Math.min(chart.points.length - 1, i))
            canvas.requestPaint()
        }
        onExited: { chart.hover = -1; canvas.requestPaint() }
    }

    ChartTip {
        visible: chart.hover >= 0 && chart.hover < chart.points.length
        anchorX: visible ? chart.xAt(chart.hover) : 0
        boundsWidth: chart.width
        y: chart.padTop
        lines: visible ? chart.points[chart.hover].tip : []
    }
}
