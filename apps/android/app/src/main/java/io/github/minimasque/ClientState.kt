package io.github.minimasque

import uniffi.masque_ffi.StatsRecord

data class ClientState(
    val proxy: String = "192.168.1.50:4433",
    val serverName: String = "192.168.1.50",
    val target: String = "1.1.1.1:53",
    val transport: String = "auto",
    val certificateName: String? = null,
    val connecting: Boolean = false,
    val connected: Boolean = false,
    val transportLabel: String? = null,
    val localAddress: String? = null,
    val canMigrate: Boolean = false,
    val error: String? = null,
    val stats: StatsRecord? = null,
    val log: List<String> = emptyList(),
    val probe: String? = null,
)
