import 'bootstrap/dist/css/bootstrap.min.css';
import './mote.scss';
import * as commands from './commands.ts';
import Editor from './editor.ts';
import Parent from './parent.ts';

import $ from 'jquery';
import bootstrap from 'bootstrap';
import bootbox from 'bootbox';

// Consume bootstrap so bootbox works.
bootstrap;


declare var WebAssembly: any;
declare var TextEncoder: any;
declare var TextDecoder: any;


function fetchAndInstantiate(url, importObject) {
  return fetch(url).then(response =>
    response.arrayBuffer()
  ).then(bytes =>
    WebAssembly.instantiate(bytes, importObject)
  ).then(results =>
    results.instance
  );
}

function copyCStr(module, ptr) {
  let orig_ptr = ptr;
  const collectCString = function* () {
    let memory = new Uint8Array(module.memory.buffer);
    while (memory[ptr] !== 0) {
      if (memory[ptr] === undefined) {
        throw new Error("Tried to read undef mem");
      }
      yield memory[ptr];
      ptr += 1
    }
  }

  const buffer_as_u8 = new Uint8Array(collectCString());
  const utf8Decoder = new TextDecoder("UTF-8");
  const buffer_as_utf8 = utf8Decoder.decode(buffer_as_u8);
  module.dealloc_str(orig_ptr);
  return buffer_as_utf8
}

function getStr(module, ptr, len) {
  const getData = function* (ptr, len) {
    let memory = new Uint8Array(module.memory.buffer);
    for (let index = 0; index < len; index++) {
      if (memory[ptr] === undefined) {
        throw new Error(`Tried to read undef mem at ${ptr}`);
      }
      yield memory[ptr + index];
    }
  }

  const buffer_as_u8 = new Uint8Array(getData(ptr/8, len/8));
  const utf8Decoder = new TextDecoder("UTF-8");
  const buffer_as_utf8 = utf8Decoder.decode(buffer_as_u8);
  return buffer_as_utf8;
}

function newString(module, str) {
  const utf8Encoder = new TextEncoder("UTF-8");
  let string_buffer = utf8Encoder.encode(str);
  let len = string_buffer.length;
  let ptr = module.alloc(len+1);

  let memory = new Uint8Array(module.memory.buffer);
  for (let i = 0; i < len; i++) {
    memory[ptr+i] = string_buffer[i];
  }

  memory[ptr+len] = 0;

  return ptr;
}

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
  fetchAndInstantiate("/mercutio.wasm", {
    env: {
      js_command: function (inptr) {
        let data = copyCStr(Module, inptr);
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
      let out = mod.exports.wasm_command(newString(Module, json));
      console.log('----- from wasm_command>', out);
      // let result = copyCStr(Module, outptr);
      // return JSON.parse(result);

    }

    // TODO encapsulate this
    mod.exports.wasm_setup(newString(Module, editorID));

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