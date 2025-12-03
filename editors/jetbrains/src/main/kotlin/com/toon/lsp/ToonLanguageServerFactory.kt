package com.toon.lsp

import com.intellij.openapi.project.Project
import com.redhat.devtools.lsp4ij.LanguageServerFactory
import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider
import com.redhat.devtools.lsp4ij.server.ProcessStreamConnectionProvider
import java.io.File
import java.nio.file.Paths

/**
 * Factory for creating TOON language server connections.
 */
class ToonLanguageServerFactory : LanguageServerFactory {

    override fun createConnectionProvider(project: Project): StreamConnectionProvider {
        val binaryPath = findBinary()
        return ProcessStreamConnectionProvider(listOf(binaryPath))
    }

    /**
     * Find the toon-lsp binary.
     *
     * Search order:
     * 1. Bundled binary in plugin resources
     * 2. System PATH
     * 3. Cargo bin directory
     */
    private fun findBinary(): String {
        // Try bundled binary
        val bundledPath = getBundledBinaryPath()
        if (bundledPath != null && File(bundledPath).exists()) {
            return bundledPath
        }

        // Try system PATH
        val systemPath = findInPath()
        if (systemPath != null) {
            return systemPath
        }

        // Try cargo bin
        val cargoPath = getCargoPath()
        if (cargoPath != null && File(cargoPath).exists()) {
            return cargoPath
        }

        throw IllegalStateException(
            "Could not find toon-lsp binary. " +
            "Please install via 'cargo install toon-lsp' or download from releases."
        )
    }

    private fun getBundledBinaryPath(): String? {
        val os = System.getProperty("os.name").lowercase()
        val binaryName = if (os.contains("win")) "toon-lsp.exe" else "toon-lsp"

        val platform = when {
            os.contains("win") -> "win32-x64"
            os.contains("mac") -> {
                val arch = System.getProperty("os.arch")
                if (arch == "aarch64") "darwin-arm64" else "darwin-x64"
            }
            else -> "linux-x64"
        }

        // Get plugin resources path
        val pluginPath = javaClass.protectionDomain.codeSource.location.toURI()
        val binaryPath = Paths.get(pluginPath).parent.resolve("binaries/$platform/$binaryName")
        return binaryPath.toString()
    }

    private fun findInPath(): String? {
        val pathEnv = System.getenv("PATH") ?: return null
        val separator = if (System.getProperty("os.name").lowercase().contains("win")) ";" else ":"
        val binaryName = if (System.getProperty("os.name").lowercase().contains("win"))
            "toon-lsp.exe" else "toon-lsp"

        for (dir in pathEnv.split(separator)) {
            val candidate = File(dir, binaryName)
            if (candidate.exists() && candidate.canExecute()) {
                return candidate.absolutePath
            }
        }
        return null
    }

    private fun getCargoPath(): String? {
        val home = System.getProperty("user.home")
        val binaryName = if (System.getProperty("os.name").lowercase().contains("win"))
            "toon-lsp.exe" else "toon-lsp"
        return Paths.get(home, ".cargo", "bin", binaryName).toString()
    }
}
