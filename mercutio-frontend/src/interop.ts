

declare var WebAssembly: any;
declare var TextEncoder: any;
declare var TextDecoder: any;


export function fetchAndInstantiate(url, importObject) {
  return fetch(url).then(response =>
    response.arrayBuffer()
  ).then(bytes =>
    WebAssembly.instantiate(bytes, importObject)
  ).then(results =>
    results.instance
  );
}

export function copyCStr(module, ptr) {
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

export function getStr(module, ptr, len) {
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

export function newString(module, str) {
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

// TODO better strategy for having a js command callback
export function instantiate(js_command_callback) {
  let Module = (<any>{});

  return fetchAndInstantiate("/$/mercutio.wasm", {
    env: {
      js_command: function (inptr) {
        let data = copyCStr(Module, inptr);
        js_command_callback(data);
      }
    }
  })
  .then(mod => {
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
    Module.wasm_setup = function (editorID) {
      mod.exports.wasm_setup(newString(Module, editorID));
    }
    return Module;
  })
}