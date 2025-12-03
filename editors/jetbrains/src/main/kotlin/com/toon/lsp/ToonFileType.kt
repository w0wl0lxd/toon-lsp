package com.toon.lsp

import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

/**
 * File type registration for .toon files.
 */
object ToonFileType : LanguageFileType(ToonLanguage) {
    override fun getName(): String = "TOON"
    override fun getDescription(): String = "TOON (Token-Oriented Object Notation) file"
    override fun getDefaultExtension(): String = "toon"
    override fun getIcon(): Icon? = null // TODO: Add icon

    @JvmField
    val INSTANCE = ToonFileType
}
