import 'bootstrap/dist/css/bootstrap.min.css';
import './mote.scss';
import * as commands from './commands.ts';
import Editor from './editor.ts';
import Parent from './parent.ts';
import * as interop from './interop.ts';

import $ from 'jquery';
import bootstrap from 'bootstrap';
import bootbox from 'bootbox';

// Consume bootstrap so bootbox works.
bootstrap;


declare var WebAssembly: any;
declare var TextEncoder: any;
declare var TextDecoder: any;

let Module = (<any>{});

// .then(_ => {
//   let input = document.getElementById("input");
//   let output = document.getElementById("output");
//   let number_out = document.getElementById("number-out");
 
//   function calcFact() {
//     value = input.value|0;
//     number_out.innerText = "fact("+value+") = ";
//     if (value < 0) {
//       output.innerText = "[Value too small.]"
//       return;
//     }
//     output.innerText = JSON.stringify(Module.command({
//       'RenameGroup': 
//         [{"CurSkip":1},{"CurWithGroup":[{"CurWithGroup":['CurGroup']}]}],
//     }));
//   }

//   calcFact();
//   input.addEventListener("keyup", calcFact);
// });







// Entry.
if ((<any>window).MOTE_ENTRY == 'index') {
  new Parent();
}
else if ((<any>window).MOTE_ENTRY == 'client') {
  if (window.parent != window) {
    // Blur/Focus classes.
    $(window).on('focus', () => $(document.body).removeClass('blurred'));
    $(window).on('blur', () => $(document.body).addClass('blurred'));
    $(document.body).addClass('blurred');
  }

  let editorID = (location.search || '').substr(1) || 'unknown';
  let editor = new Editor($('#mote'), editorID);


  console.log('start');
  interop.fetchAndInstantiate("/mercutio.wasm", {
    env: {
      js_command: function (inptr) {
        let data = interop.copyCStr(Module, inptr);
        console.log('----> js_command:', data);
        setImmediate(() => {
          editor.onNativeMessage({
            data: data,
          });
        });
      }
    }
  })
  .then(mod => {
    console.log('hi', mod),
    Module.alloc = mod.exports.alloc;
    Module.dealloc_str = mod.exports.dealloc_str;
    Module.memory = mod.exports.memory;
    Module.wasm_command = function(req) {
      let json = JSON.stringify(req);
      let out = mod.exports.wasm_command(interop.newString(Module, json));
      console.log('----- from wasm_command>', out);
      // let result = copyCStr(Module, outptr);
      // return JSON.parse(result);

    }

    // TODO encapsulate this
    mod.exports.wasm_setup(interop.newString(Module, editorID));

    setImmediate(() => {
      let syncSocket = new WebSocket('ws://' + window.location.host.replace(/\:\d+/, ':8001') + '/');
      editor.Module = Module;
      editor.syncSocket = syncSocket;
      syncSocket.onopen = function (event) {
        console.log('Editor "%s" is connected.', editor.editorID);
      };
      syncSocket.onmessage = function (event) {
        console.log('GOT SYNC SCOKET MESSAGE:', event.data);
        Module.wasm_command({
          SyncClientCommand: JSON.parse(event.data),
        });
      };
      syncSocket.onclose = function () {
        $('body').css('background', 'red');
      }
    });

    // alert('done');
  })

  // editor.syncConnect();
  // editor.nativeConnect();
}