import * as path from 'path';
import Mocha from 'mocha';
import { glob } from 'glob';

export async function run(): Promise<void> {
    // Create the mocha test
    const mocha = new Mocha({
        ui: 'tdd',
        color: true,
        timeout: 60000 // 60 seconds for LSP startup
    });

    const testsRoot = path.resolve(__dirname, '.');

    return new Promise((resolve, reject) => {
            glob('**/**.test.js', { cwd: testsRoot }).then((files: string[]) => {
            // Add files to the test suite
            for (const f of files) {
                mocha.addFile(path.resolve(testsRoot, f));
            }

            try {
                // Run the mocha tests
                mocha.run((failures: number) => {
                    if (failures > 0) {
                        reject(new Error(`${failures} tests failed.`));
                    } else {
                        resolve();
                    }
                });
            } catch (err) {
                console.error(err);
                reject(err);
            }
        }).catch((err) => {
            reject(err);
        });
    });
}
