import './mote.scss';
import * as commands from './commands';
import Editor from './editor';
import Multi from './multi';
import * as interop from './interop';
import * as util from './util';

import $ from 'jquery';

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

  let editor = new Editor(document.getElementById('mote'), '$$$$$$');

  if (!window['CONFIG'].configured) {
    alert('The window.CONFIG variable was not configured by the server!')
  }

  // Use cross-compiled WASM bundle.
  let WASM = window['CONFIG'].wasm;
  if (!WASM) {
    editor.multiConnect();
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
        let syncSocket = new WebSocket(
          util.syncUrl() + (window.location.hash == '#helloworld' ? '?helloworld' : '')
        );
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
  // Use cross-compiled WASM bundle.
  let WASM = window['CONFIG'].wasm;
  if (!WASM) {
    let nativeSocket = new WebSocket(util.clientProxyUrl());

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
  } else {
    let md;
    let M;
    interop.instantiate(function (data) {
      // console.log('----> js_command:', data);

      let json_data = JSON.parse(data);

      // Make this async so we don't have deeply nested call stacks from Rust<->JS interop.
      // setImmediate(() => {
      //   editor.onNativeMessage({
      //     data: data,
      //   });
      // });

      console.log(json_data);

      if (!md && json_data.MarkdownUpdate) {
        md = json_data.MarkdownUpdate;

        (<any>window).remark.create({
          source: md,
        });

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

      if (json_data.Init) {
        let id = setInterval(() => {
          M.wasm_command({
            NativeCommand: {
              RequestMarkdown: null,
            },
          });

          clearInterval(id);
        }, 100);
      }
    })
    .then(Module => {
      M = Module;
      Module.wasm_setup();

      setImmediate(() => {
        let syncSocket = new WebSocket(util.syncUrl());

        syncSocket.onopen = function (event) {
          // console.log('Editor "%s" is connected.', '$presenter');
        };

        syncSocket.onmessage = function (event) {
          Module.wasm_command({
            SyncClientCommand: JSON.parse(event.data),
          });
        };
      });
    })
  }
}
else {
  document.body.innerHTML = '<h1>404</h1>';
}

$('#footer').html(`
⚠️ You are viewing a sandbox for <b><a href="https://github.com/tcr/edit-text">edit-text</a></b>.
There is a high chance of data loss, so don't store anything important here.
Thanks for trying it out!
`);
