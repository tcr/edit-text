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

console.log('start');
fetchAndInstantiate("/mercutio.wasm", {
  env: {
    js_command: function () {
      
    }
  }
})
.then(mod => {
  console.log('hi', mod),
  Module.alloc = mod.exports.alloc;
  Module.dealloc_str = mod.exports.dealloc_str;
  Module.memory = mod.exports.memory;
  // Module.command = function(req) {
  //   let json = JSON.stringify(req);
  //   let outptr = mod.exports.command(newString(Module, json));
  //   let result = copyCStr(Module, outptr);
  //   return JSON.parse(result);
  // }
  alert(mod.exports.wasm_test());
})
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







// Blur/Focus classes.
$(window).on('focus', () => $(document.body).addClass('focused'));
$(window).on('blur', () => $(document.body).removeClass('focused'));

// Entry.
if ((<any>window).MOTE_ENTRY == 'index') {
  new Parent();
}
else if ((<any>window).MOTE_ENTRY == 'client') {
  // let editor = new Editor($('#mote'), (location.search || '').substr(1));

  // editor.syncConnect();
  // editor.nativeConnect();
}