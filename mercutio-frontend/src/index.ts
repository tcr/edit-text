// Global CSS
import '../styles/mote.scss';

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
  document.body.innerHTML = `

<div id="footer">
⚠️ You are viewing a sandbox for <b><a href="https://github.com/tcr/edit-text">edit-text</a></b>.
There is a high chance of data loss, so don't store anything important here.
Thanks for trying it out!
</div>

<div id="toolbar">
  <a href="https://github.com/tcr/edit-text" id="logo">edit-text</a>
  <div id="native-buttons"></div>
  <div id="local-buttons"></div>
</div>

<div class="mote" id="mote"></div>

`;

  // Utility classes for Multi
  if (window.parent != window) {
    // Blur/Focus classes.
    $(window).on('focus', () => $(document.body).removeClass('blurred'));
    $(window).on('blur', () => $(document.body).addClass('blurred'));
    $(document.body).addClass('blurred');
  }

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
