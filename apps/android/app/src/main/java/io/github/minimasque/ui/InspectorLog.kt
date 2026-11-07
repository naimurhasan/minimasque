package io.github.minimasque.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp

private fun colorFor(line: String): Color = when (line.firstOrNull()) {
    '▶' -> Color(0xFFF9E2AF)
    '◀' -> Color(0xFFA6E3A1)
    '●' -> Color(0xFF94E2D5)
    '◆' -> Color(0xFFF5C2E7)
    else -> Color(0xFF9399B2)
}

@Composable
fun InspectorLog(lines: List<String>) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Protocol inspector", style = MaterialTheme.typography.titleMedium)
            lines.forEach { line ->
                Text(line, color = colorFor(line), fontFamily = FontFamily.Monospace, style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}
