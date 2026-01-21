#!/usr/bin/env bash

# RSX VS Code Extension Installation Script

echo "Building RSX VS Code Extension..."

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "Error: This script must be run from the extension directory"
    exit 1
fi

# Install dependencies
echo "Installing dependencies..."
npm install

# Compile TypeScript
echo "Compiling TypeScript..."
npm run compile

# Package extension (requires vsce to be installed)
if command -v vsce &> /dev/null; then
    echo "Packaging extension..."
    vsce package
    echo "Extension packaged successfully!"
    echo "Install the .vsix file in VS Code or run:"
    echo "  code --install-extension rsx-syntax-support-*.vsix"
else
    echo "To package the extension, install vsce:"
    echo "  npm install -g vsce"
    echo ""
    echo "Then run:"
    echo "  vsce package"
    echo ""
    echo "For development, you can also:"
    echo "1. Open this folder in VS Code"
    echo "2. Press F5 to launch Extension Development Host"
    echo "3. Test the extension with .rs files containing rsx! macros"
fi

echo ""
echo "Extension compilation complete!"
echo "The extension provides:"
echo "  - HTML syntax highlighting in rsx! macros"  
echo "  - Rust expression highlighting in single braces {variable}"
echo "  - Auto-completion for HTML elements"
echo "  - Hover documentation for rsx macros"