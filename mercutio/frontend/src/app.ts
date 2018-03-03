import 'bootstrap/dist/css/bootstrap.min.css';
import './mote.scss';
import * as commands from './commands';
import Editor from './editor';
import Multi from './multi';
import * as interop from './interop';

import $ from 'jquery';
import bootstrap from 'bootstrap';
import bootbox from 'bootbox';

// Consume bootstrap so bootbox works.
bootstrap;


declare var WebAssembly: any;
declare var TextEncoder: any;
declare var TextDecoder: any;

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
      <iframe src="/"></iframe>
    </td>
    <td>
      <iframe src="/"></iframe>
    </td>
    <td>
      <iframe src="/"></iframe>
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


  if (window.parent != window) {
    // Blur/Focus classes.
    $(window).on('focus', () => $(document.body).removeClass('blurred'));
    $(window).on('blur', () => $(document.body).addClass('blurred'));
    $(document.body).addClass('blurred');
  }

  let editor = new Editor(document.getElementById('mote'), '$$$$$$');

  console.log('start');

  if (!window['CONFIG'].configured) {
    alert('The window.CONFIG variable was not configured by the server!')
  }

  // Use cross-compiled WASM bundle.
  let WASM = window['CONFIG'].wasm;
  if (!WASM) {
    editor.syncConnect();
    editor.nativeConnect();
  } else {
    interop.instantiate(function (data) {
      // console.log('----> js_command:', data);

      // Make this async so we don't have deeply nested call stacks from Rust<->JS interop.
      setImmediate(() => {
        editor.onNativeMessage({
          data: data,
        });
      });
    })
    .then(Module => {
      Module.wasm_setup();

      setImmediate(() => {
        // Websocket port
        let url = window.location.host.match(/localhost/) ?
          'ws://' + window.location.host.replace(/:\d+$|$/, ':8001') + '/' :
          'ws://' + window.location.host + '/ws/';

        let syncSocket = new WebSocket(url);
        editor.Module = Module; 
        editor.syncSocket = syncSocket;
        syncSocket.onopen = function (event) {
          console.log('Editor "%s" is connected.', editor.editorID);
        };

        // Keepalive
        setInterval(() => {
          syncSocket.send(JSON.stringify({
            Keepalive: null,
          }));
        }, 1000);

        syncSocket.onmessage = function (event) {
          // console.log('GOT SYNC SOCKET MESSAGE:', event.data);
          Module.wasm_command({
            SyncClientCommand: JSON.parse(event.data),
          });
        };
        syncSocket.onclose = function () {
          // TODO use a class
          $('html').css('background', '#f00');
          $('body').css('opacity', '0.7');
        }
      });
    })
  }
}
else if (document.body.id == 'presentation') {
  let url = 'ws://' + window.location.host.replace(/\:\d+/, ':8002') + '/$presenter';

  let nativeSocket = new WebSocket(url);

  let md = null;
  nativeSocket.onmessage = function (event) {
    let packet = JSON.parse(event.data);
    if (packet.MarkdownUpdate) {
      md = packet.MarkdownUpdate;

      (<any>window).remark.create({
        source: md,
      });
    }
    // Module.wasm_command({
    //   SyncClientCommand: JSON.parse(event.data),
    // });
  };

  setInterval(() => {
    if (md == null) {
      nativeSocket.send(JSON.stringify({RequestMarkdown: null}));
    }
  }, 500);
  nativeSocket.send(JSON.stringify({RequestMarkdown: null}));
}
else {
  document.body.innerHTML = '404';
}

$('#footer').html(`
⚠️ You are viewing a sandbox for <b><a href="https://github.com/tcr/edit-text">edit-text</a></b>.
There is a high chance of data loss, so don't store anything important here.
Thanks for trying it out!
`);
