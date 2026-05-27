package com.toon.lsp

import com.intellij.extapi.psi.PsiFileBase
import com.intellij.openapi.fileTypes.FileType
import com.intellij.psi.FileViewProvider

/**
 * PSI file representation for TOON files.
 */
class ToonFile(viewProvider: FileViewProvider) : PsiFileBase(viewProvider, ToonLanguage) {
    override fun getFileType(): FileType = ToonFileType
    override fun toString(): String = "TOON File"
}
