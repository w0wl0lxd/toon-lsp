package com.toon.lsp

import com.intellij.lang.ASTNode
import com.intellij.lang.PsiBuilder
import com.intellij.lang.PsiParser
import com.intellij.psi.tree.IElementType

/**
 * Minimal parser for TOON files.
 *
 * Note: The actual parsing is done by the LSP server.
 * This parser just builds a simple AST for IDE features.
 */
class ToonParser : PsiParser {
    override fun parse(root: IElementType, builder: PsiBuilder): ASTNode {
        val rootMarker = builder.mark()

        while (!builder.eof()) {
            builder.advanceLexer()
        }

        rootMarker.done(root)
        return builder.treeBuilt
    }
}
