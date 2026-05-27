// TOON Language Server - VSCode Extension
// Provides LSP client integration for TOON files

import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import {
    workspace,
    ExtensionContext,
    window,
    commands,
    OutputChannel
} from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: OutputChannel;

/**
 * Detect the current platform and return the appropriate binary name
 */
function getPlatformInfo(): { platform: string; binaryName: string } {
    const platform = os.platform();
    const arch = os.arch();

    let platformDir: string;
    let binaryName: string;

    switch (platform) {
        case 'win32':
            platformDir = 'win32-x64';
            binaryName = 'toon-lsp.exe';
            break;
        case 'darwin':
            platformDir = arch === 'arm64' ? 'darwin-arm64' : 'darwin-x64';
            binaryName = 'toon-lsp';
            break;
        case 'linux':
            platformDir = 'linux-x64';
            binaryName = 'toon-lsp';
            break;
        default:
            throw new Error(`Unsupported platform: ${platform}`);
    }

    return { platform: platformDir, binaryName };
}

/**
 * Find the toon-lsp binary, checking:
 * 1. User-configured path
 * 2. Bundled binary for current platform
 * 3. System PATH
 */
function findServerBinary(context: ExtensionContext): string | undefined {
    const config = workspace.getConfiguration('toon-lsp');

    // 1. Check user-configured path
    const configuredPath = config.get<string>('path');
    if (configuredPath && fs.existsSync(configuredPath)) {
        outputChannel.appendLine(`Using configured binary: ${configuredPath}`);
        return configuredPath;
    }

    // 2. Check bundled binary
    const { platform, binaryName } = getPlatformInfo();
    const bundledPath = path.join(context.extensionPath, 'binaries', platform, binaryName);

    if (fs.existsSync(bundledPath)) {
        outputChannel.appendLine(`Using bundled binary: ${bundledPath}`);
        return bundledPath;
    }

    // 3. Check system PATH
    const pathEnv = process.env.PATH || '';
    const pathSeparator = os.platform() === 'win32' ? ';' : ':';
    const pathDirs = pathEnv.split(pathSeparator);

    for (const dir of pathDirs) {
        const candidate = path.join(dir, binaryName);
        if (fs.existsSync(candidate)) {
            outputChannel.appendLine(`Using system binary: ${candidate}`);
            return candidate;
        }
    }

    // Check if cargo-installed
    const cargoPath = path.join(os.homedir(), '.cargo', 'bin', binaryName);
    if (fs.existsSync(cargoPath)) {
        outputChannel.appendLine(`Using cargo-installed binary: ${cargoPath}`);
        return cargoPath;
    }

    return undefined;
}

/**
 * Activate the extension
 */
export async function activate(context: ExtensionContext): Promise<void> {
    outputChannel = window.createOutputChannel('TOON Language Server');
    context.subscriptions.push(outputChannel);

    outputChannel.appendLine('Activating TOON Language Server extension...');

    // Find the server binary
    const serverPath = findServerBinary(context);

    if (!serverPath) {
        const message = 'Could not find toon-lsp binary. Please install it via cargo (cargo install toon-lsp) or configure the path in settings.';
        window.showErrorMessage(message);
        outputChannel.appendLine(`ERROR: ${message}`);
        return;
    }

    // Server options - run toon-lsp as stdio server
    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio,
            options: {
                env: { ...process.env, RUST_LOG: 'debug' }
            }
        }
    };

    // Client options
    const config = workspace.getConfiguration('toon-lsp');
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'toon' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.toon')
        },
        outputChannel,
        traceOutputChannel: outputChannel,
        initializationOptions: {
            formatting: {
                tabSize: config.get<number>('formatting.tabSize', 2),
                useTabs: config.get<boolean>('formatting.useTabs', false)
            }
        }
    };

    // Create the language client
    client = new LanguageClient(
        'toon-lsp',
        'TOON Language Server',
        serverOptions,
        clientOptions
    );

    // Register restart command
    context.subscriptions.push(
        commands.registerCommand('toon-lsp.restart', async () => {
            outputChannel.appendLine('Restarting TOON Language Server...');
            if (client) {
                await client.stop();
                await client.start();
            }
        })
    );

    // Start the client
    try {
        await client.start();
        outputChannel.appendLine('TOON Language Server started successfully');
    } catch (error) {
        const message = `Failed to start TOON Language Server: ${error}`;
        window.showErrorMessage(message);
        outputChannel.appendLine(`ERROR: ${message}`);
    }
}

/**
 * Deactivate the extension
 */
export async function deactivate(): Promise<void> {
    if (client) {
        await client.stop();
    }
}
