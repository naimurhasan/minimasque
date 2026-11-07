package io.github.minimasque

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import io.github.minimasque.dns.probe
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.masque_ffi.Forwarder
import uniffi.masque_ffi.ForwarderConfig

private const val LOG_CAPACITY = 200
private const val POLL_MS = 500L
private const val LOCAL_ADDRESS = "127.0.0.1:0"

class ClientViewModel : ViewModel() {
    private val mutableState = MutableStateFlow(ClientState())
    val state: StateFlow<ClientState> = mutableState.asStateFlow()

    private var forwarder: Forwarder? = null
    private var certificate: ByteArray? = null
    private var poller: Job? = null

    fun edit(transform: (ClientState) -> ClientState) = mutableState.update(transform)

    fun setCertificate(name: String, bytes: ByteArray) {
        certificate = bytes
        mutableState.update { it.copy(certificateName = name) }
    }

    fun connect() {
        val ca = certificate ?: return mutableState.update { it.copy(error = "Pick the proxy certificate first") }
        val current = state.value
        mutableState.update { it.copy(connecting = true, error = null) }
        viewModelScope.launch {
            runCatching {
                withContext(Dispatchers.IO) {
                    Forwarder.start(
                        ForwarderConfig(
                            proxy = current.proxy,
                            serverName = current.serverName,
                            caDer = ca,
                            target = current.target,
                            local = LOCAL_ADDRESS,
                            transport = current.transport,
                            template = null,
                        ),
                    )
                }
            }.onSuccess { started ->
                forwarder = started
                mutableState.update {
                    it.copy(
                        connecting = false,
                        connected = true,
                        transportLabel = started.transport(),
                        localAddress = started.localAddress(),
                        canMigrate = started.canMigrate(),
                    )
                }
                poll(started)
            }.onFailure { error ->
                mutableState.update { it.copy(connecting = false, error = error.message) }
            }
        }
    }

    fun disconnect() {
        poller?.cancel()
        forwarder?.stop()
        forwarder?.close()
        forwarder = null
        mutableState.update { it.copy(connected = false, transportLabel = null, localAddress = null, canMigrate = false) }
    }

    fun migrate() {
        val started = forwarder ?: return
        viewModelScope.launch {
            runCatching { withContext(Dispatchers.IO) { started.migrate() } }
                .onFailure { error -> mutableState.update { it.copy(error = error.message) } }
        }
    }

    fun probe(name: String) {
        val local = state.value.localAddress ?: return
        viewModelScope.launch {
            val result = runCatching { probe(local, name) }.getOrElse { "failed: ${it.message}" }
            mutableState.update { it.copy(probe = result) }
        }
    }

    private fun poll(started: Forwarder) {
        poller = viewModelScope.launch {
            while (isActive) {
                val lines = started.drainEvents()
                val stats = started.stats()
                mutableState.update { it.copy(stats = stats, log = (it.log + lines).takeLast(LOG_CAPACITY)) }
                delay(POLL_MS)
            }
        }
    }

    override fun onCleared() = disconnect()
}
