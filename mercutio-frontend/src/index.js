// Global CSS
import '../styles/mercutio.scss';
import {booted} from './bindgen/mercutio_bg.js';

// Workaround for webpack
export function getWasmModule() {
    return booted
    .then(() => import('./bindgen/mercutio'));
}

// Launch the application.
import * as app from './app';
app.start();
