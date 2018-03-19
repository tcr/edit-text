// Global CSS
import '../styles/mote.scss';

import * as commands from './commands';
import {Editor} from './editor';
import Multi from './multi';
import * as interop from './interop';
import * as util from './util';
import {ProxyNetwork, WasmNetwork} from './network';

// Check page configuration.
if (!window['CONFIG'].configured) {
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

  new Multi();
}
else if (document.body.id == 'client') {
  document.body.innerHTML = `
<div id="footer"></div>
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
  let network = window['CONFIG'].wasm ?
    new WasmNetwork() :
    new ProxyNetwork()

  // Create the editor.
  let editor = new Editor(document.getElementById('mote'), '$$$$$$', network);
  // Connect to parent window (if exists).
  editor.multiConnect();

  // Background colors.
  network.onNativeClose = function () {
    $('body').css('background', 'red');
  };
  network.onSyncClose = function () {
    $('body').css('background', 'red');
  };

  // Connect to remote sockets.
  network.nativeConnect()
  .then(() => network.syncConnect())
  .then(() => {
    console.log('edit-text initialized.');
  });
}
else if (document.body.id == 'presentation') {
  // Connects to the network.
  let network = window['CONFIG'].wasm ?
    new WasmNetwork() :
    new ProxyNetwork();

  let md = null;
  network.onNativeMessage = function (msg) {
    console.log(msg);

    if (!md && msg.MarkdownUpdate) {
      md = msg.MarkdownUpdate;

      // Start the remark.js presentation.
      (<any>window).remark.create({
        source: md,
      });

      // Adds fullscreen button after remark is created.
      $('<button>↕️</button>').on('click', function () {
        console.log('fullscreen attempt');
        let a = document.querySelector('.remark-slides-area');
        try {
          (<any>a).mozRequestFullScreen();
        } catch (e) {
          (<any>a).requestFullscreen();
        }
      })
        .css('position', 'fixed')
        .css('top', 10)
        .css('left', 10)
        .css('z-index', 1000)
        .appendTo($('body'));
    }
  }

  // Connect to remote sockets.
  network.nativeConnect()
  .then(() => network.syncConnect())
  .then(() => {
    console.log('edit-text initialized.');

    // Request markdown source immediately.
    let id = setInterval(function () {
      if (md !== null) {
        clearInterval(id);
      } else {
        network.nativeCommand(commands.RequestMarkdown());
      }
    }, 250);
  });
}
else {
  document.body.innerHTML = '<h1>404</h1>';
}

$('#footer').html(`
⚠️ You are viewing a sandbox for <b><a href="https://github.com/tcr/edit-text">edit-text</a></b>.
There is a high chance of data loss, so don't store anything important here.
Thanks for trying it out!
`);
