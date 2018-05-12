// Global CSS
import '../styles/mercutio.scss';
import {booted} from './bindgen/mercutio_client_bg.js';

// Workaround for webpack
export function getWasmModule() {
    return booted
    .then(() => import('./bindgen/mercutio_client'));
}

// Launch the application.
import * as app from './app';
app.start();
