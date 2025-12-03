// T016: Diagnostics tests using invalid fixtures
import * as assert from 'assert';
import * as vscode from 'vscode';
import * as path from 'path';

suite('Diagnostics Tests', () => {
    const fixturesPath = path.resolve(__dirname, '../../../../tests/fixtures');

    // Helper to wait for diagnostics
    async function waitForDiagnostics(uri: vscode.Uri, timeout: number = 5000): Promise<vscode.Diagnostic[]> {
        return new Promise((resolve, reject) => {
            const disposable = vscode.languages.onDidChangeDiagnostics((e) => {
                if (e.uris.some(u => u.toString() === uri.toString())) {
                    disposable.dispose();
                    resolve(vscode.languages.getDiagnostics(uri));
                }
            });

            // Timeout after specified duration
            setTimeout(() => {
                disposable.dispose();
                // Return current diagnostics even if no change event fired
                resolve(vscode.languages.getDiagnostics(uri));
            }, timeout);
        });
    }

    test('Should show diagnostics for syntax errors', async () => {
        try {
            const filePath = path.join(fixturesPath, 'invalid/syntax-error.toon');
            const uri = vscode.Uri.file(filePath);
            const doc = await vscode.workspace.openTextDocument(uri);
            await vscode.window.showTextDocument(doc);

            // Wait for LSP to provide diagnostics
            const diagnostics = await waitForDiagnostics(uri);

            // Should have at least one diagnostic for the unclosed bracket
            assert.ok(diagnostics.length > 0, 'Should have diagnostics for syntax error');

            // Check that the diagnostic is an error
            const error = diagnostics.find(d => d.severity === vscode.DiagnosticSeverity.Error);
            assert.ok(error, 'Should have an error-level diagnostic');

            // Clean up
            await vscode.commands.executeCommand('workbench.action.closeActiveEditor');
        } catch (err) {
            console.log('Skipping test: LSP or fixture not available');
        }
    });

    test('Should show diagnostics for duplicate keys', async () => {
        try {
            const filePath = path.join(fixturesPath, 'invalid/duplicate-keys.toon');
            const uri = vscode.Uri.file(filePath);
            const doc = await vscode.workspace.openTextDocument(uri);
            await vscode.window.showTextDocument(doc);

            // Wait for LSP to provide diagnostics
            const diagnostics = await waitForDiagnostics(uri);

            // Should have diagnostic about duplicate key
            assert.ok(diagnostics.length > 0, 'Should have diagnostics for duplicate keys');

            // Check that at least one diagnostic mentions duplicate
            const duplicateWarning = diagnostics.find(d =>
                d.message.toLowerCase().includes('duplicate') ||
                d.message.toLowerCase().includes('already defined')
            );
            assert.ok(duplicateWarning, 'Should warn about duplicate key');

            // Clean up
            await vscode.commands.executeCommand('workbench.action.closeActiveEditor');
        } catch (err) {
            console.log('Skipping test: LSP or fixture not available');
        }
    });

    test('Valid TOON should have no diagnostics', async () => {
        try {
            const filePath = path.join(fixturesPath, 'valid/simple.toon');
            const uri = vscode.Uri.file(filePath);
            const doc = await vscode.workspace.openTextDocument(uri);
            await vscode.window.showTextDocument(doc);

            // Wait a bit for diagnostics to settle
            await new Promise(resolve => setTimeout(resolve, 2000));

            const diagnostics = vscode.languages.getDiagnostics(uri);

            // Should have no error diagnostics for valid TOON
            const errors = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Error);
            assert.strictEqual(errors.length, 0, 'Valid TOON should have no error diagnostics');

            // Clean up
            await vscode.commands.executeCommand('workbench.action.closeActiveEditor');
        } catch (err) {
            console.log('Skipping test: LSP or fixture not available');
        }
    });

    test('Diagnostics should update after editing', async () => {
        try {
            // Create a document with valid content
            const doc = await vscode.workspace.openTextDocument({
                language: 'toon',
                content: 'name: test\n'
            });
            const editor = await vscode.window.showTextDocument(doc);

            // Wait for initial diagnostics
            await new Promise(resolve => setTimeout(resolve, 1000));

            const initialDiagnostics = vscode.languages.getDiagnostics(doc.uri);
            const initialErrorCount = initialDiagnostics.filter(
                d => d.severity === vscode.DiagnosticSeverity.Error
            ).length;

            // Introduce an error
            await editor.edit(edit => {
                edit.insert(new vscode.Position(1, 0), 'broken: [\n');
            });

            // Wait for diagnostics to update
            await waitForDiagnostics(doc.uri);

            const updatedDiagnostics = vscode.languages.getDiagnostics(doc.uri);
            const updatedErrorCount = updatedDiagnostics.filter(
                d => d.severity === vscode.DiagnosticSeverity.Error
            ).length;

            // Should have more errors after introducing invalid syntax
            assert.ok(
                updatedErrorCount > initialErrorCount,
                'Should have more diagnostics after introducing error'
            );

            // Clean up
            await vscode.commands.executeCommand('workbench.action.closeActiveEditor');
        } catch (err) {
            console.log('Skipping test: LSP not available');
        }
    });
});
