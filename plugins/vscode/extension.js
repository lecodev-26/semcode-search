const vscode = require('vscode');
const { execFile } = require('child_process');
const path = require('path');

/**
 * Ejecuta el binario semcode-search con los argumentos dados
 * @param {string[]} args - Argumentos del comando
 * @returns {Promise<string>} Salida del comando
 */
function runSemcode(args) {
    return new Promise((resolve, reject) => {
        execFile('semcode-search', args, (error, stdout, stderr) => {
            if (error) {
                reject(new Error(stderr || error.message));
            } else {
                resolve(stdout);
            }
        });
    });
}

/**
 * Obtiene la carpeta del workspace actual
 * @returns {string} Ruta de la carpeta
 */
function getWorkspacePath() {
    const folders = vscode.workspace.workspaceFolders;
    if (!folders || folders.length === 0) {
        throw new Error('No hay carpeta de proyecto abierta en VSCode');
    }
    return folders[0].uri.fsPath;
}

/**
 * Activa la extensión
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
    console.log('Semcode Search activado');

    // Comando: Búsqueda normal
    let searchCommand = vscode.commands.registerCommand('semcode.search', async () => {
        const query = await vscode.window.showInputBox({
            prompt: 'Buscar en el código',
            placeHolder: 'fn main, cache management, ...'
        });

        if (!query) return;

        try {
            const workspacePath = getWorkspacePath();
            const output = await runSemcode(['search', '--query', query, '--path', workspacePath]);

            // Mostrar en panel de salida
            const channel = vscode.window.createOutputChannel('Semcode Search');
            channel.clear();
            channel.append(output);
            channel.show();

            vscode.window.showInformationMessage(`✅ Búsqueda completada: "${query}"`);
        } catch (error) {
            vscode.window.showErrorMessage(`❌ Error: ${error.message}`);
        }
    });

    // Comando: Búsqueda con IA
    let searchAICommand = vscode.commands.registerCommand('semcode.searchAI', async () => {
        const query = await vscode.window.showInputBox({
            prompt: 'Buscar con IA (por significado)',
            placeHolder: 'función para validar emails, ...'
        });

        if (!query) return;

        try {
            const workspacePath = getWorkspacePath();
            const output = await runSemcode(['search', '--query', query, '--path', workspacePath, '--ai']);

            const channel = vscode.window.createOutputChannel('Semcode Search (AI)');
            channel.clear();
            channel.append(output);
            channel.show();

            vscode.window.showInformationMessage(`🧠 Búsqueda con IA completada: "${query}"`);
        } catch (error) {
            vscode.window.showErrorMessage(`❌ Error: ${error.message}`);
        }
    });

    // Comando: Indexar proyecto
    let indexCommand = vscode.commands.registerCommand('semcode.index', async () => {
        try {
            const workspacePath = getWorkspacePath();

            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Semcode: Indexando proyecto...',
                cancellable: false
            }, async (progress) => {
                const output = await runSemcode(['index', '--path', workspacePath]);

                const channel = vscode.window.createOutputChannel('Semcode Index');
                channel.clear();
                channel.append(output);
                channel.show();
            });

            vscode.window.showInformationMessage('✅ Proyecto indexado correctamente');
        } catch (error) {
            vscode.window.showErrorMessage(`❌ Error: ${error.message}`);
        }
    });

    // Comando: Ver estadísticas
    let statsCommand = vscode.commands.registerCommand('semcode.stats', async () => {
        try {
            const output = await runSemcode(['stats']);

            const channel = vscode.window.createOutputChannel('Semcode Stats');
            channel.clear();
            channel.append(output);
            channel.show();
        } catch (error) {
            vscode.window.showErrorMessage(`❌ Error: ${error.message}`);
        }
    });

    context.subscriptions.push(searchCommand);
    context.subscriptions.push(searchAICommand);
    context.subscriptions.push(indexCommand);
    context.subscriptions.push(statsCommand);
}

/**
 * Desactiva la extensión
 */
function deactivate() {
    console.log('Semcode Search desactivado');
}

module.exports = {
    activate,
    deactivate
};