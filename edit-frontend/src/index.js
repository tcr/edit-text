// Global CSS
import '../styles/edit.scss';
import {booted} from './bindgen/edit_client_bg.js';

// Workaround for webpack
export function getWasmModule() {
    return booted
    .then(() => import('./bindgen/edit_client'));
}

// Launch the application.
import * as app from './app';

window.Raven.context(() => {
    app.start();
});
