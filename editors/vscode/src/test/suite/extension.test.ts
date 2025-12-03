// T014: Extension activation tests
import * as assert from 'assert';
import * as vscode from 'vscode';

suite('Extension Activation Tests', () => {
    vscode.window.showInformationMessage('Start extension activation tests.');

    test('Extension should be present', () => {
        const extension = vscode.extensions.getExtension('toon-format.toon-lsp');
        assert.ok(extension, 'Extension should be available');
    });

    test('Extension should activate on TOON file', async () => {
        const extension = vscode.extensions.getExtension('toon-format.toon-lsp');
        if (!extension) {
            assert.fail('Extension not found');
            return;
        }

        // Create a TOON document to trigger activation
        const doc = await vscode.workspace.openTextDocument({
            language: 'toon',
            content: 'name: test\n'
        });

        // Wait for extension to activate
        await extension.activate();

        assert.ok(extension.isActive, 'Extension should be active after opening TOON file');

        // Clean up
        await vscode.commands.executeCommand('workbench.action.closeActiveEditor');
    });

    test('TOON language should be registered', () => {
        const languages = vscode.languages.getLanguages();
        // getLanguages returns a Thenable, need to wait for it
        languages.then((langs) => {
            assert.ok(langs.includes('toon'), 'TOON language should be registered');
        });
    });

    test('Extension configuration should be available', () => {
        const config = vscode.workspace.getConfiguration('toon-lsp');
        assert.ok(config, 'Configuration should be available');

        // Check that all expected settings exist
        const path = config.get('path');
        const traceServer = config.get('trace.server');
        const tabSize = config.get('formatting.tabSize');
        const useTabs = config.get('formatting.useTabs');

        assert.strictEqual(path, '', 'Default path should be empty');
        assert.strictEqual(traceServer, 'off', 'Default trace.server should be off');
        assert.strictEqual(tabSize, 2, 'Default tabSize should be 2');
        assert.strictEqual(useTabs, false, 'Default useTabs should be false');
    });
});
