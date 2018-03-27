// Global CSS
import '../styles/mercutio.scss';

// Workaround for webpack
export function getWasmModule() {
    return import('./bindgen/mercutio');
}

// Launch the application.
import * as frame from './frame';
frame.start();
