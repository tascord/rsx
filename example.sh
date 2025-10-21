#!/bin/bash

# RSX Example Demo Script
# This script builds and runs the _example crate with webpack

set -e

echo "🚀 Setting up RSX Example Demo..."

# Navigate to example directory
cd _example

echo "🔨 Building WebAssembly..."
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    source ~/.bashrc
fi

echo "Building with wasm-pack..."
wasm-pack build --target web --out-dir pkg

echo "📦 Installing npm dependencies..."
npm install

echo "🎉 Setup complete! Starting development server..."
echo "🌐 Opening http://localhost:8080 in your browser..."
echo ""
echo "To manually run the demo later:"
echo "  cd _example"
echo "  npm run serve"
echo ""

npm run serve
cd ..