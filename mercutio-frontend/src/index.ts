// Global CSS
import '../styles/mercutio.scss';

import {ProxyNetwork, WasmNetwork} from './network';
import * as editorFrame from './views/editor-frame';
import * as multi from './views/multi';
import * as presentation from './views/presentation';

declare var remark: any;
declare var CONFIG: any;

// Check page configuration.
if (!CONFIG.configured) {
  alert('The window.CONFIG variable was not configured by the server!')
}

// Entry.
switch (document.body.id) {
  case 'multi': {
    multi.start();
    break;
  }
  
  case 'client': {
    let network = CONFIG.wasm ? new WasmNetwork() : new ProxyNetwork();
    editorFrame.start(network);
    break;
  }

  case 'presentation': {
    let network = CONFIG.wasm ? new WasmNetwork() : new ProxyNetwork();
    presentation.start(network);
    break;
  }
  
  default: {
    document.body.innerHTML = `<h1>404</h1>`;
    break;
  }
}
