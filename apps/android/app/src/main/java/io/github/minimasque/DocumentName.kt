package io.github.minimasque

import android.content.ContentResolver
import android.net.Uri
import android.provider.OpenableColumns

fun documentName(resolver: ContentResolver, uri: Uri): String =
    resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
        if (cursor.moveToFirst()) cursor.getString(0) else null
    } ?: uri.lastPathSegment ?: "certificate"
