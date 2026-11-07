package io.github.minimasque

private val UNITS = listOf("B", "KiB", "MiB", "GiB")

fun formatBytes(value: Long): String {
    var scaled = value.toDouble()
    var unit = 0
    while (scaled >= 1024 && unit < UNITS.lastIndex) {
        scaled /= 1024
        unit++
    }
    return if (unit == 0) "$value B" else "%.1f %s".format(scaled, UNITS[unit])
}
