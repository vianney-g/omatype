// Labels and numbers shared by the views. Numbers go through Qt.locale() so
// they take the language's separators.

function num(value, decimals) {
    return Number(value || 0).toLocaleString(Qt.locale(), 'f', decimals === undefined ? 0 : decimals)
}

function quoteLength(value) {
    switch (value) {
    case 1: return qsTr("short")
    case 2: return qsTr("medium")
    case 3: return qsTr("long")
    }
    return qsTr("any length")
}

// "time 30", "words 25", "quote short", "zen".
function modeLabel(mode, value) {
    switch (mode) {
    case "time": return qsTr("time %1").arg(value)
    case "words": return qsTr("words %1").arg(value)
    case "quote": return qsTr("quote %1").arg(quoteLength(value))
    }
    return qsTr("zen")
}

function languageName(code) {
    return code === "fr" ? qsTr("French") : qsTr("English")
}

// What sets a run apart beyond its mode: language, punctuation, numbers.
function extras(t) {
    var parts = [languageName(t.language)]
    if (t.punctuation) parts.push(qsTr("punctuation"))
    if (t.numbers) parts.push(qsTr("numbers"))
    if (t.stopOnError) parts.push(qsTr("stop on error"))
    if (t.confidence) parts.push(qsTr("confidence"))
    return parts.join(" · ")
}

function duration(seconds) {
    seconds = Math.round(seconds || 0)
    var h = Math.floor(seconds / 3600), m = Math.floor(seconds % 3600 / 60), s = seconds % 60
    if (h > 0) return qsTr("%1 h %2 min").arg(h).arg(m)
    if (m > 0) return qsTr("%1 min %2 s").arg(m).arg(s)
    return qsTr("%1 s").arg(s)
}

function dateTime(ms) {
    return new Date(ms).toLocaleString(Qt.locale(), Locale.ShortFormat)
}

// "2026-11-09" from SQLite, shown as a short local date.
function day(text) {
    var p = text.split("-")
    var d = new Date(Number(p[0]), Number(p[1]) - 1, Number(p[2]))
    return d.toLocaleDateString(Qt.locale(), "d MMM")
}

function escape(c) {
    switch (c) {
    case "&": return "&amp;"
    case "<": return "&lt;"
    case ">": return "&gt;"
    case "\"": return "&quot;"
    }
    return c
}

// One word as styled text: typed letters in the text color, wrong ones in the
// error color (showing the letter that was asked for, so the line doesn't
// shift), extra ones dimmer, the rest muted. A finished wrong word is underlined.
function wordHtml(target, typed, index, current, zen, colors) {
    var t = Array.from(target), y = Array.from(typed)
    if (zen) return "<font color=\"" + colors.fg + "\">" + y.map(escape).join("") + "</font>"
    var html = "", run = "", runColor = ""
    function flush() {
        if (run.length) html += "<font color=\"" + runColor + "\">" + run + "</font>"
        run = ""
    }
    var n = Math.max(t.length, y.length)
    for (var i = 0; i < n; ++i) {
        var color, ch
        if (i < y.length && i < t.length) {
            color = y[i] === t[i] ? colors.fg : colors.error
            ch = t[i]
        } else if (i < y.length) {
            color = colors.extra
            ch = y[i]
        } else {
            color = colors.muted
            ch = t[i]
        }
        if (color !== runColor) flush()
        runColor = color
        run += escape(ch)
    }
    flush()
    if (index < current && typed !== target) html = "<u>" + html + "</u>"
    return html
}

// A round upper bound for a chart axis.
function niceMax(value) {
    if (value <= 10) return 10
    var step = value <= 50 ? 10 : value <= 150 ? 25 : 50
    return Math.ceil(value / step) * step
}
