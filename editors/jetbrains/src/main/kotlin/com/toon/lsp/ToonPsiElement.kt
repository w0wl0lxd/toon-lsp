package com.toon.lsp

import com.intellij.extapi.psi.ASTWrapperPsiElement
import com.intellij.lang.ASTNode

/**
 * Generic PSI element for TOON AST nodes.
 */
class ToonPsiElement(node: ASTNode) : ASTWrapperPsiElement(node)
