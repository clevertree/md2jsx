package com.clevertree.md2jsx

import android.util.Log

object MarkdownParser {
    init {
        try {
            System.loadLibrary("md2jsx")
        } catch (e: Exception) {
            Log.e("MarkdownParser", "Failed to load native library md2jsx", e)
        }
    }

    fun parse(markdown: String, allowedTags: List<String> = emptyList()): String {
        val allowedTagsJson = "[\"" + allowedTags.joinToString("\",\"") + "\"]"
        return nativeParse(markdown, allowedTagsJson)
    }

    private external fun nativeParse(markdown: String, allowedTagsJson: String): String
}
