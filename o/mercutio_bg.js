let imports = {};
imports['env'] = require('env');

            const bytes = require('fs').readFileSync('mercutio_bg.wasm');
            const wasmModule = new WebAssembly.Module(bytes);
            const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
            module.exports = wasmInstance.exports;
        