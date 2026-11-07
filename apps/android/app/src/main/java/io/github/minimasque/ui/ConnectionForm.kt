package io.github.minimasque.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import io.github.minimasque.ClientState
import io.github.minimasque.documentName

@Composable
fun ConnectionForm(
    state: ClientState,
    onEdit: ((ClientState) -> ClientState) -> Unit,
    onCertificate: (String, ByteArray) -> Unit,
    onConnect: () -> Unit,
    onDisconnect: () -> Unit,
) {
    val resolver = LocalContext.current.contentResolver
    val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        val bytes = uri?.let { resolver.openInputStream(it)?.use { stream -> stream.readBytes() } }
        if (uri != null && bytes != null) onCertificate(documentName(resolver, uri), bytes)
    }
    val editable = !state.connected && !state.connecting
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        OutlinedTextField(
            value = state.proxy,
            onValueChange = { value -> onEdit { it.copy(proxy = value) } },
            label = { Text("Proxy (ip:port)") },
            enabled = editable,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = state.serverName,
            onValueChange = { value -> onEdit { it.copy(serverName = value) } },
            label = { Text("Certificate name") },
            enabled = editable,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = state.target,
            onValueChange = { value -> onEdit { it.copy(target = value) } },
            label = { Text("UDP target") },
            enabled = editable,
            modifier = Modifier.fillMaxWidth(),
        )
        TransportPicker(
            selected = state.transport,
            enabled = editable,
            onSelect = { value -> onEdit { it.copy(transport = value) } },
        )
        OutlinedButton(onClick = { picker.launch(arrayOf("*/*")) }, enabled = editable) {
            Text(state.certificateName?.let { "Certificate: $it" } ?: "Pick proxy certificate (.der)")
        }
        if (state.connected) {
            Button(onClick = onDisconnect) { Text("Disconnect") }
        } else {
            Button(onClick = onConnect, enabled = !state.connecting) {
                Text(if (state.connecting) "Connecting…" else "Connect")
            }
        }
        state.error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
    }
}
