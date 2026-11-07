package io.github.minimasque.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import io.github.minimasque.ClientViewModel

@Composable
fun ClientScreen(model: ClientViewModel = viewModel()) {
    val state by model.state.collectAsState()
    Surface(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .safeDrawingPadding()
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Text("MiniMASQUE", style = MaterialTheme.typography.headlineMedium)
            Text("CONNECT-UDP over HTTP/3, with HTTP/2 fallback", style = MaterialTheme.typography.bodyMedium)
            ConnectionForm(
                state = state,
                onEdit = model::edit,
                onCertificate = model::setCertificate,
                onConnect = model::connect,
                onDisconnect = model::disconnect,
            )
            if (state.connected) {
                StatsCard(state)
                ProbeCard(result = state.probe, onProbe = model::probe)
            }
            InspectorLog(state.log)
        }
    }
}
