package com.toon.lsp

import com.intellij.lang.ASTNode
import com.intellij.lang.ParserDefinition
import com.intellij.lang.PsiParser
import com.intellij.lexer.Lexer
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IFileElementType
import com.intellij.psi.tree.TokenSet

/**
 * Parser definition for TOON files.
 *
 * Note: The actual parsing is done by the LSP server.
 * This provides minimal support for the IDE's PSI infrastructure.
 */
class ToonParserDefinition : ParserDefinition {

    override fun createLexer(project: Project?): Lexer {
        return ToonLexer()
    }

    override fun createParser(project: Project?): PsiParser {
        return ToonParser()
    }

    override fun getFileNodeType(): IFileElementType {
        return FILE
    }

    override fun getCommentTokens(): TokenSet {
        return COMMENTS
    }

    override fun getStringLiteralElements(): TokenSet {
        return STRINGS
    }

    override fun createElement(node: ASTNode?): PsiElement {
        return ToonPsiElement(node!!)
    }

    override fun createFile(viewProvider: FileViewProvider): PsiFile {
        return ToonFile(viewProvider)
    }

    companion object {
        val FILE = IFileElementType(ToonLanguage)
        val COMMENTS = TokenSet.create(ToonElementTypes.COMMENT)
        val STRINGS = TokenSet.create(ToonElementTypes.STRING)
    }
}
