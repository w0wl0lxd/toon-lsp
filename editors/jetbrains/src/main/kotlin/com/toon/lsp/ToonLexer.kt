package com.toon.lsp

import com.intellij.lexer.LexerBase
import com.intellij.psi.tree.IElementType

/**
 * Minimal lexer for TOON files.
 *
 * Note: The actual parsing is done by the LSP server.
 * This lexer provides basic tokenization for IDE features.
 */
class ToonLexer : LexerBase() {
    private var buffer: CharSequence = ""
    private var bufferEnd: Int = 0
    private var tokenStart: Int = 0
    private var tokenEnd: Int = 0
    private var tokenType: IElementType? = null

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        this.buffer = buffer
        this.bufferEnd = endOffset
        this.tokenStart = startOffset
        this.tokenEnd = startOffset
        advance()
    }

    override fun getState(): Int = 0

    override fun getTokenType(): IElementType? = tokenType

    override fun getTokenStart(): Int = tokenStart

    override fun getTokenEnd(): Int = tokenEnd

    override fun advance() {
        tokenStart = tokenEnd
        if (tokenStart >= bufferEnd) {
            tokenType = null
            return
        }

        val c = buffer[tokenStart]
        tokenType = when {
            c == '#' -> {
                // Comment - consume until end of line
                tokenEnd = findEndOfLine()
                ToonElementTypes.COMMENT
            }
            c == '\n' || c == '\r' -> {
                tokenEnd = tokenStart + 1
                if (c == '\r' && tokenEnd < bufferEnd && buffer[tokenEnd] == '\n') {
                    tokenEnd++
                }
                ToonElementTypes.NEWLINE
            }
            c.isWhitespace() -> {
                // Consume whitespace
                tokenEnd = tokenStart + 1
                while (tokenEnd < bufferEnd && buffer[tokenEnd].isWhitespace() &&
                       buffer[tokenEnd] != '\n' && buffer[tokenEnd] != '\r') {
                    tokenEnd++
                }
                ToonElementTypes.WHITESPACE
            }
            c == ':' -> {
                tokenEnd = tokenStart + 1
                ToonElementTypes.COLON
            }
            c == '-' -> {
                tokenEnd = tokenStart + 1
                ToonElementTypes.DASH
            }
            c == '|' -> {
                tokenEnd = tokenStart + 1
                ToonElementTypes.PIPE
            }
            c == '[' -> {
                tokenEnd = tokenStart + 1
                ToonElementTypes.LBRACKET
            }
            c == ']' -> {
                tokenEnd = tokenStart + 1
                ToonElementTypes.RBRACKET
            }
            c == ',' -> {
                tokenEnd = tokenStart + 1
                ToonElementTypes.COMMA
            }
            c == '"' || c == '\'' -> {
                tokenEnd = findStringEnd(c)
                ToonElementTypes.STRING
            }
            c.isDigit() || c == '-' && tokenStart + 1 < bufferEnd && buffer[tokenStart + 1].isDigit() -> {
                tokenEnd = findNumberEnd()
                ToonElementTypes.NUMBER
            }
            isIdentifierStart(c) -> {
                tokenEnd = findIdentifierEnd()
                val text = buffer.subSequence(tokenStart, tokenEnd).toString()
                when (text) {
                    "true", "false" -> ToonElementTypes.BOOLEAN
                    "null" -> ToonElementTypes.NULL
                    else -> ToonElementTypes.KEY
                }
            }
            else -> {
                tokenEnd = tokenStart + 1
                ToonElementTypes.BAD_CHARACTER
            }
        }
    }

    private fun findEndOfLine(): Int {
        var pos = tokenStart + 1
        while (pos < bufferEnd && buffer[pos] != '\n' && buffer[pos] != '\r') {
            pos++
        }
        return pos
    }

    private fun findStringEnd(quote: Char): Int {
        var pos = tokenStart + 1
        while (pos < bufferEnd) {
            val c = buffer[pos]
            if (c == quote) {
                return pos + 1
            }
            if (c == '\\' && pos + 1 < bufferEnd) {
                pos += 2
                continue
            }
            if (c == '\n' || c == '\r') {
                return pos
            }
            pos++
        }
        return pos
    }

    private fun findNumberEnd(): Int {
        var pos = tokenStart
        if (buffer[pos] == '-') pos++
        while (pos < bufferEnd && buffer[pos].isDigit()) pos++
        if (pos < bufferEnd && buffer[pos] == '.') {
            pos++
            while (pos < bufferEnd && buffer[pos].isDigit()) pos++
        }
        if (pos < bufferEnd && (buffer[pos] == 'e' || buffer[pos] == 'E')) {
            pos++
            if (pos < bufferEnd && (buffer[pos] == '+' || buffer[pos] == '-')) pos++
            while (pos < bufferEnd && buffer[pos].isDigit()) pos++
        }
        return pos
    }

    private fun findIdentifierEnd(): Int {
        var pos = tokenStart + 1
        while (pos < bufferEnd && isIdentifierPart(buffer[pos])) {
            pos++
        }
        return pos
    }

    private fun isIdentifierStart(c: Char): Boolean = c.isLetter() || c == '_'
    private fun isIdentifierPart(c: Char): Boolean = c.isLetterOrDigit() || c == '_' || c == '-'

    override fun getBufferSequence(): CharSequence = buffer

    override fun getBufferEnd(): Int = bufferEnd
}
