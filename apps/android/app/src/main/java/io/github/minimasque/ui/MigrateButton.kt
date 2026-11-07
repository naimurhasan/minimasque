package io.github.minimasque.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun MigrateButton(onMigrate: () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        OutlinedButton(onClick = onMigrate, modifier = Modifier.fillMaxWidth()) {
            Text("Migrate path")
        }
        Text(
            "Moves QUIC to a new UDP socket, like switching Wi-Fi to mobile data. The tunnel stays open.",
            style = MaterialTheme.typography.bodySmall,
        )
    }
}
