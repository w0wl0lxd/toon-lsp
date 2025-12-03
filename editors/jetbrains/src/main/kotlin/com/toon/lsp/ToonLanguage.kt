package com.toon.lsp

import com.intellij.lang.Language

/**
 * TOON language definition for JetBrains IDEs.
 */
object ToonLanguage : Language("TOON") {
    override fun getDisplayName(): String = "TOON"
    override fun isCaseSensitive(): Boolean = true
}
