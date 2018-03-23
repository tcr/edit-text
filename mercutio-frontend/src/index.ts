// Global CSS
import '../styles/mercutio.scss';

import {ProxyNetwork, WasmNetwork} from './network';
import * as page from './views/page';
import * as multi from './views/multi';
import * as presentation from './views/presentation';

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
    page.start();
    break;
  }
  case 'presentation': {
    presentation.start();
    break;
  }
  default: {
    document.body.innerHTML = `<h1>404</h1>`;
    break;
  }
}
