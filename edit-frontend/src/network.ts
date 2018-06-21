import * as commands from './commands';
import * as route from './route';
import * as index from './index';
import {WasmClient} from './bindgen/edit_client';

export interface Network {
  onNativeMessage: (any) => void;
  onNativeClose: () => void;
  onSyncClose: () => void;

  nativeConnect(): Promise<void>;
  nativeCommand(command: commands.Command): void;
  syncConnect(): Promise<void>;
}

export class ProxyNetwork implements Network {
  editorID: string;

  nativeSocket: WebSocket;

  onNativeMessage: (any) => void;
  onNativeClose: () => void;
  onSyncClose: () => void; // unused

  nativeCommand(command: commands.Command) {
    delete command.tag;
    this.nativeSocket.send(JSON.stringify(command));
  }

  nativeConnect(): Promise<void> {
    let network = this;
    return Promise.resolve()
    .then(() => {
      this.nativeSocket = new WebSocket(
        route.clientProxyUrl()
      );
      this.nativeSocket.onopen = function (event) {
        console.debug('client-proxy socket opened.');
      };
      this.nativeSocket.onmessage = function (event) {
        let parse = JSON.parse(event.data);
        network.onNativeMessage(parse);
      };
      this.nativeSocket.onclose = network.onNativeClose;
    });
  }

  // The native server (the client proxy) handles sync traffic directly
  syncConnect(): Promise<void> {
    return Promise.resolve();
  }
}


let sendCommandToJSList: Array<(any) => void> = [];

export function sendCommandToJS(msg) {
  sendCommandToJSList.forEach(handler => handler(msg));
}

let forwardWasmTaskCallback: any = null;

export function forwardWasmTask(msg) {
  if (forwardWasmTaskCallback) {
    forwardWasmTaskCallback(msg);
  }
}

function WasmError(e, message) {
    this.name = 'WasmError';
    this.message = message;
    this.stack = message + ' ' + e.stack;
}
WasmError.prototype = new Error;

export class WasmNetwork implements Network {
  editorID: string;

  nativeSocket: WebSocket;
  syncSocket: WebSocket;

  // Create a deferred object for the sync socket
  // because we may receive UserToSyncCommand payloads earlier
  deferSync: Promise<WebSocket>;
  deferSyncResolve: Function;

  // TODO refactor wasmClient, remove Module
  Module: any;
  wasmClient: WasmClient;

  onNativeMessage: (any) => void;
  onNativeClose: () => void; // unused
  onSyncClose: () => void;

  constructor() {
    this.deferSync = new Promise(function(resolve, reject){
      this.deferSyncResolve = resolve;
    }.bind(this));
  }

  nativeCommand(command: commands.Command) {
    delete command.tag;
    if (forwardWasmTaskCallback != null) {
      this.wasmClient.command(JSON.stringify({
        FrontendToUserCommand: command,
      }));
    }
  }

  // Wasm connector.
  nativeConnect(): Promise<void> {
    const network = this;
    return new Promise((resolve, reject) => {
      sendCommandToJSList.push((data) => {
        
        // console.log('----> js_command:', data);

        // Make this async so we don't have deeply nested call stacks from Rust<->JS interop.
        setImmediate(() => {
          // Parse the packet.
          let parse = JSON.parse(data);

          if (parse.UserToSyncCommand) {
            network.deferSync.then(syncSocket => {
              syncSocket.send(JSON.stringify(parse.UserToSyncCommand));
            });
          } else {
            network.onNativeMessage(parse);
          }
        });
      });

      index.getWasmModule()
      .then(Module => {
        let wasmClient = Module.wasm_setup();
  
        setImmediate(() => {
          // Websocket port
          network.Module = Module;
          network.wasmClient = wasmClient;

          forwardWasmTaskCallback = (msg) => {
            try {
              wasmClient.command(msg);
            } catch (e) {
              forwardWasmTaskCallback = null;

              throw new WasmError(e, `Error during client command: ${e.message}`);
            }
          };

          resolve();
        });
      });
    });
  }

  syncConnect(): Promise<void> {
    let network = this;

    return Promise.resolve()
    .then(() => {
      let syncSocket = new WebSocket(
        route.syncUrl()
      );
      syncSocket.onopen = function (event) {
        console.debug('server socket opened.');
      };

      syncSocket.onmessage = function (event) {
        // console.log('Got message from sync:', event.data);
        try {
          if (forwardWasmTaskCallback != null) {
            network.wasmClient.command(JSON.stringify({
              SyncToUserCommand: JSON.parse(event.data),
            }));
          }
        } catch (e) {
          // Kill the current process, we triggered an exception.
          forwardWasmTaskCallback = null;
          network.Module.wasm_close();
          // syncSocket.close();

          // TODO this is the wrong place to put this
          (document as any).body.background = 'red';

          throw new WasmError(e, `Error during sync command: ${e.message}`);
        }
      };

      syncSocket.onclose = network.onSyncClose;

      this.deferSyncResolve(syncSocket);
    });
  }
}