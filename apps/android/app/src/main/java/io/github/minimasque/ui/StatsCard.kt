package io.github.minimasque.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import io.github.minimasque.ClientState
import io.github.minimasque.formatBytes

@Composable
fun StatsCard(state: ClientState) {
    val stats = state.stats
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text("● Connected · ${state.transportLabel}", style = MaterialTheme.typography.titleMedium)
            Text("Apps can send UDP to ${state.localAddress}")
            if (stats != null) {
                Text("↑ ${formatBytes(stats.bytesUp.toLong())} (${stats.packetsUp} packets)")
                Text("↓ ${formatBytes(stats.bytesDown.toLong())} (${stats.packetsDown} packets)")
                Text("tunnels open ${stats.openTunnels} · dropped ${stats.dropped}")
            }
        }
    }
}
