
import 'react';

import * as index from '..';
import { WasmClient as WasmClientModule } from '../bindgen/edit_client';
import { getWasmModule } from '../index';

import {Command} from './commands';
import {ControllerImpl, ServerImpl} from './network';
import DEBUG from '../debug';

let _convertMarkdownToDoc: ((x: string) => any) | null = null;
let _convertMarkdownToHtml: ((x: string) => any) | null = null;
getWasmModule()
.then(Module => {
  _convertMarkdownToDoc = Module.convertMarkdownToDoc;
  _convertMarkdownToHtml = Module.convertMarkdownToHtml;
});

export function convertMarkdownToDoc(input: string): any {
  return JSON.parse(_convertMarkdownToDoc!(input));
}


export function convertMarkdownToHtml(input: string): string {
  return _convertMarkdownToHtml!(input);
}



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

export class WasmClient implements ControllerImpl {
  // public
  server: ServerImpl | null;
  onMessage: (msg: any) => void | null;
  onClose: () => void | null; // unused

  // Private

  editorID: string;

  // TODO refactor wasmClient, remove Module
  Module: any;
  clientBindings: WasmClientModule;

  sendCommand(command: Command) {
    delete command.tag;
    if (forwardWasmTaskCallback != null) {
      this.clientBindings.command(JSON.stringify({
        ControllerCommand: command,
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
        // setImmediate(() => {
          // Parse the packet.
          let parse = JSON.parse(data);

          if (parse.ServerCommand && client.server != null) {
            client.server.sendCommand(parse.ServerCommand);
          } else {
            if (client.onMessage != null) {
              client.onMessage(parse);
            }
          }
        // });
      });

      index.getWasmModule()
      .then(Module => {
        let clientBindings = Module.wasm_setup();
        DEBUG.setGlobalClientBindings(clientBindings);
  
        setImmediate(() => {
          // Websocket port
          client.Module = Module;
          client.clientBindings = clientBindings;

          forwardWasmTaskCallback = (msg: any) => {
            try {
              clientBindings.command(msg);
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
