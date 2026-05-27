package com.toon.lsp

import com.intellij.psi.tree.IElementType

/**
 * Element types for TOON tokens.
 */
object ToonElementTypes {
    val COMMENT = IElementType("COMMENT", ToonLanguage)
    val KEY = IElementType("KEY", ToonLanguage)
    val COLON = IElementType("COLON", ToonLanguage)
    val STRING = IElementType("STRING", ToonLanguage)
    val NUMBER = IElementType("NUMBER", ToonLanguage)
    val BOOLEAN = IElementType("BOOLEAN", ToonLanguage)
    val NULL = IElementType("NULL", ToonLanguage)
    val DASH = IElementType("DASH", ToonLanguage)
    val PIPE = IElementType("PIPE", ToonLanguage)
    val LBRACKET = IElementType("LBRACKET", ToonLanguage)
    val RBRACKET = IElementType("RBRACKET", ToonLanguage)
    val COMMA = IElementType("COMMA", ToonLanguage)
    val WHITESPACE = IElementType("WHITESPACE", ToonLanguage)
    val NEWLINE = IElementType("NEWLINE", ToonLanguage)
    val BAD_CHARACTER = IElementType("BAD_CHARACTER", ToonLanguage)
}
