// Global CSS
import '../styles/mercutio.scss';

import * as commands from './commands';
import * as interop from './interop';
import * as util from './util';
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
if (document.body.id == 'multi') {
  document.body.innerHTML = `

<h1>Multimonkey
  <button id="action-monkey">🙈🙉🙊</button>
  <span style="font-family: monospace; padding: 3px 5px;" id="timer"></span>
</h1>

<table id="clients">
  <tr>
    <td>
      <iframe src="/monkey"></iframe>
    </td>
    <td>
      <iframe src="/monkey"></iframe>
    </td>
    <td>
      <iframe src="/monkey"></iframe>
    </td>
  </tr>
</table>

`;

  multi.start();
}
else if (document.body.id == 'client') {
  // Connects to the network.
  let network = CONFIG.wasm ?
    new WasmNetwork() :
    new ProxyNetwork()

  editorFrame.start(network);
}
else if (document.body.id == 'presentation') {
  // Connects to the network.
  let network = CONFIG.wasm ?
    new WasmNetwork() :
    new ProxyNetwork();

  presentation.start(network);
}
else {
  document.body.innerHTML = `

<h1>404</h1>

`;
}
