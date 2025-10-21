import init, { run_example } from './pkg/example.js';

async function run() {
    try {
        await init();
        console.log('🦀 RSX Example loaded successfully!');
        run_example();
    } catch (error) {
        console.error('❌ Failed to load RSX example:', error);
        const appDiv = document.getElementById('app');
        if (appDiv) {
            appDiv.innerHTML = 
                '<p style="color: red; padding: 10px; background: #ffe6e6; border-radius: 4px;">' +
                '❌ Failed to load WebAssembly module. Make sure to build the Rust code first with:<br>' +
                '<code>wasm-pack build --target web --out-dir pkg</code></p>';
        }
    }
}

run();
