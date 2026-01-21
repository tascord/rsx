"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = require("vscode");
function activate(context) {
    console.log('RSX Syntax Support extension is now active!');
    // Language configuration for RSX
    const rsxLanguageConfig = vscode.languages.setLanguageConfiguration('rust', {
        wordPattern: /(-?\d*\.\d\w*)|([^\`\~\!\@\#\%\^\&\*\(\)\-\=\+\[\{\]\}\\\|\;\:\'\"\,\.\<\>\/\?\s]+)/,
    });
    // Register hover provider for RSX syntax information
    const hoverProvider = vscode.languages.registerHoverProvider('rust', {
        provideHover(document, position) {
            const range = document.getWordRangeAtPosition(position);
            const word = document.getText(range);
            if (word === 'rsx') {
                const line = document.lineAt(position);
                if (line.text.includes('rsx!')) {
                    return new vscode.Hover(new vscode.MarkdownString(`
**RSX Macro**

This macro provides HTML-like syntax for building DOM structures in Rust.

Features:
- HTML element syntax: \`<div>content</div>\`
- Rust expressions in single braces: \`{variable}\`
- Escaped braces for literal text: \`{{literal}}\`
- HTML attributes with Rust values: \`<div class={class_name}>\`

Example:
\`\`\`rust
rsx! {
    <div class="container">
        <h1>Hello {name}!</h1>
        <button onclick={|_| handle_click()}>
            Click me
        </button>
    </div>
}
\`\`\`
                        `));
                }
            }
            return undefined;
        }
    });
    // Register completion provider for common HTML elements
    const completionProvider = vscode.languages.registerCompletionItemProvider('rust', {
        provideCompletionItems(document, position) {
            const linePrefix = document.lineAt(position).text.substr(0, position.character);
            // Check if we're inside an rsx! macro
            const line = document.lineAt(position);
            const text = document.getText(new vscode.Range(0, 0, position.line, position.character));
            // Simple check for being inside rsx! macro
            const rsxMatches = text.match(/rsx!\s*\{/g);
            const closingBraces = text.match(/\}/g);
            if (rsxMatches && (!closingBraces || rsxMatches.length > closingBraces.length)) {
                // We're likely inside an rsx! macro
                if (linePrefix.endsWith('<')) {
                    const completionItems = [
                        'div', 'span', 'p', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
                        'button', 'input', 'form', 'label', 'select', 'option',
                        'ul', 'ol', 'li', 'table', 'tr', 'td', 'th', 'thead', 'tbody',
                        'img', 'a', 'br', 'hr', 'header', 'footer', 'main', 'section',
                        'article', 'nav', 'aside'
                    ].map(tag => {
                        const item = new vscode.CompletionItem(tag, vscode.CompletionItemKind.Snippet);
                        item.insertText = new vscode.SnippetString(`${tag}$1>$2</${tag}>`);
                        item.documentation = new vscode.MarkdownString(`Insert HTML ${tag} element`);
                        return item;
                    });
                    return completionItems;
                }
            }
            return [];
        }
    }, '<');
    // Register document formatter for RSX content
    const documentFormatter = vscode.languages.registerDocumentFormattingEditProvider('rust', {
        provideDocumentFormattingEdits(document) {
            const edits = [];
            // Basic formatting for RSX macros
            for (let i = 0; i < document.lineCount; i++) {
                const line = document.lineAt(i);
                const text = line.text;
                // Format RSX macro opening
                const rsxMatch = text.match(/(\s*)rsx!\s*\{/);
                if (rsxMatch) {
                    const formatted = `${rsxMatch[1]}rsx! {`;
                    if (text !== formatted) {
                        edits.push(vscode.TextEdit.replace(line.range, formatted));
                    }
                }
            }
            return edits;
        }
    });
    // Add disposables to context
    context.subscriptions.push(rsxLanguageConfig, hoverProvider, completionProvider, documentFormatter);
    // Show activation message
    vscode.window.showInformationMessage('RSX Syntax Support is now active!');
}
exports.activate = activate;
function deactivate() {
    console.log('RSX Syntax Support extension is now deactivated');
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map