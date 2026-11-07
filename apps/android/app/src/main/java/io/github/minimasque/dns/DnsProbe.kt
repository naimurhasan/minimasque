package io.github.minimasque.dns

import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

private const val TIMEOUT_MS = 3000

suspend fun probe(local: String, name: String): String = withContext(Dispatchers.IO) {
    val host = local.substringBeforeLast(':')
    val port = local.substringAfterLast(':').toInt()
    val query = dnsQuery((0..0xffff).random(), name)
    DatagramSocket().use { socket ->
        socket.soTimeout = TIMEOUT_MS
        val started = System.nanoTime()
        socket.send(DatagramPacket(query, query.size, InetAddress.getByName(host), port))
        val buffer = ByteArray(1500)
        val reply = DatagramPacket(buffer, buffer.size)
        socket.receive(reply)
        val millis = (System.nanoTime() - started) / 1_000_000
        val answer = parseDnsResponse(buffer, reply.length) ?: return@use "invalid reply"
        "$name · ${rcodeName(answer.rcode)} · ${answer.answers} answers · $millis ms"
    }
}
