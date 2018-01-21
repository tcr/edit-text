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
if (document.body.id == 'multi') {
  document.body.innerHTML = `

<h1>Mercutio
  <button id="action-monkey">🙈🙉🙊</button>
  <span style="font-family: monospace; padding: 3px 5px;" id="timer"></span>
</h1>

<table id="clients">
  <!-- <tr> 
    <td>
      <h4>"left"</h4>
      <iframe style="border: none; width: 100%; height: 800px" src="/client/?left"></iframe>
    </td>
    <td>
      <h4>"middle"</h4>
      <iframe style="border: none; width: 100%; height: 800px" src="/client/?middle"></iframe>
    </td>
    <td>
      <h4>"right"</h4>
      <iframe style="border: none; width: 100%; height: 800px" src="/client/?right"></iframe>
    </td>
  </tr> -->

  <tr>
    <td>
      <iframe src="/client/?a"></iframe>
    </td>
    <td>
      <iframe src="/client/?b"></iframe>
    </td>
    <td>
      <iframe src="/client/?c"></iframe>
    </td>
    <!--
    <td>
      <iframe src="/client/?d"></iframe>
    </td>
    <td>
      <iframe src="/client/?e"></iframe>
    </td>
    -->
  </tr>
  <!--
  <tr>
    <td>
      <iframe src="/client/?f"></iframe>
    </td>
    <td>
      <iframe src="/client/?g"></iframe>
    </td>
    <td>
      <iframe src="/client/?h"></iframe>
    </td>
    <td>
      <iframe src="/client/?i"></iframe>
    </td>
    <td>
      <iframe src="/client/?j"></iframe>
    </td>
  </tr>
  -->
</table>

`;

  new Parent();
}
else if (document.body.id == 'client') {
  document.body.innerHTML = `
<div id="toolbar">
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
} else {
  document.body.innerHTML = '404';
}