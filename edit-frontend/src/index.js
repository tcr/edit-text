// Global CSS
import '../styles/edit.scss';
// import {booted} from './bindgen/edit_client_bg.wasm';

import Raven from 'raven-js';

// Workaround for webpack
export function getWasmModule() {
    return import('./bindgen/edit_client');
}

Raven.config('https://c221eba12d7b4e279b01764577063af1@sentry.io/1227661').install();

// Launch the application.
import * as app from './app/app';
import DEBUG from './debug';

DEBUG.measureTime('wasm-request');
getWasmModule()
.then(Module => {
    DEBUG.measureTime('wasm-ready');
});

Raven.context(() => {
    if (window.location.pathname.match(/^\/\$\/react/)) {
        app.start_standalone();
    } else {
        app.start();
    }
});
