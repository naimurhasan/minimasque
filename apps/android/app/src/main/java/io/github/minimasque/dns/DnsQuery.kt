package io.github.minimasque.dns

data class DnsAnswer(val rcode: Int, val answers: Int)

private val RCODES = listOf("NOERROR", "FORMERR", "SERVFAIL", "NXDOMAIN", "NOTIMP", "REFUSED")

fun dnsQuery(id: Int, name: String): ByteArray {
    val header = byteArrayOf(
        (id shr 8).toByte(), id.toByte(), 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    )
    val labels = name.trimEnd('.').split('.').flatMap { label ->
        listOf(label.length.toByte()) + label.toByteArray().toList()
    }
    val question = labels + listOf<Byte>(0x00, 0x00, 0x01, 0x00, 0x01)
    return header + question.toByteArray()
}

fun parseDnsResponse(bytes: ByteArray, length: Int): DnsAnswer? {
    if (length < 12) return null
    val rcode = bytes[3].toInt() and 0x0f
    val answers = ((bytes[6].toInt() and 0xff) shl 8) or (bytes[7].toInt() and 0xff)
    return DnsAnswer(rcode, answers)
}

fun rcodeName(rcode: Int): String = RCODES.getOrElse(rcode) { "RCODE $rcode" }
