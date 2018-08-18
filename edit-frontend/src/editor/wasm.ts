
import 'react';

import * as index from '..';
import {WasmClient as WasmClientModule} from '../bindgen/edit_client';
import {Command} from './commands';
import {ClientImpl, ServerImpl} from './network';




// TODO what are all of these things vvv

let sendCommandToJSList: Array<(value: any) => void> = [];

export function sendCommandToJS(msg: any) {
  sendCommandToJSList.forEach(handler => handler(msg));
}

let forwardWasmTaskCallback: any = null;

export function getForwardWasmTaskCallback(): any {
  return forwardWasmTaskCallback;
}

export function setForwardWasmTaskCallback(value: any) {
  forwardWasmTaskCallback = value;
}

export function forwardWasmTask(msg: any) {
  if (forwardWasmTaskCallback) {
    forwardWasmTaskCallback(msg);
  }
}

// ^^^^^


export class WasmError extends Error {
    constructor(e: Error, message: any) {
        super(message);

        // Set the prototype explicitly.
        this.name = 'WasmError';
        this.message = message;
        this.stack = message + ' ' + e.stack;
        Object.setPrototypeOf(this, WasmError.prototype);
    }
}

export class WasmClient implements ClientImpl {
  // public
  server: ServerImpl | null;
  onNativeMessage: (msg: any) => void;
  onNativeClose: () => void; // unused

  // Private

  editorID: string;

  // TODO refactor wasmClient, remove Module
  Module: any;
  wasmClient: WasmClientModule;

  nativeCommand(command: Command) {
    delete command.tag;
    if (forwardWasmTaskCallback != null) {
      this.wasmClient.command(JSON.stringify({
        FrontendToUserCommand: command,
      }));
    }
  }

  // Wasm connector.
  connect(onError: () => void): Promise<void> {
    const client = this;
    return new Promise((resolve, reject) => {
      sendCommandToJSList.push((data) => {
        
        // console.log('----> js_command:', data);

        // Make this async so we don't have deeply nested call stacks from Rust<->JS interop.
        setImmediate(() => {
          // Parse the packet.
          let parse = JSON.parse(data);

          if (parse.UserToSyncCommand && client.server != null) {
            client.server.syncCommand(parse.UserToSyncCommand);
          } else {
            client.onNativeMessage(parse);
          }
        });
      });

      index.getWasmModule()
      .then(Module => {
        let wasmClient = Module.wasm_setup();
  
        setImmediate(() => {
          // Websocket port
          client.Module = Module;
          client.wasmClient = wasmClient;

          forwardWasmTaskCallback = (msg: any) => {
            try {
              wasmClient.command(msg);
            } catch (e) {
              forwardWasmTaskCallback = null;

              onError();

              throw new WasmError(e, `Error during client command: ${e.message}`);
            }
          };

          resolve();
        });
      });
    });
  }
}
