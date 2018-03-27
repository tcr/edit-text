import * as commands from './commands';
import * as route from './route';
import * as index from './index';

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
        console.log('Editor "%s" is connected.', network.editorID);
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

export class WasmNetwork implements Network {
  editorID: string;

  nativeSocket: WebSocket;
  syncSocket: WebSocket;

  // Create a deferred object for the sync socket
  // because we may receive SyncServerCommand payloads earlier
  deferSync: Promise<WebSocket>;
  deferSyncResolve: Function;

  // TODO remove this
  Module: any;

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
    this.Module.wasm_command(JSON.stringify({
      NativeCommand: command,
    }));
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

          if (parse.SyncServerCommand) {
            network.deferSync.then(syncSocket => {
              syncSocket.send(JSON.stringify(parse.SyncServerCommand));
            });
          } else {
            network.onNativeMessage(parse);
          }
        });
      });

      index.getWasmModule()
      .then(Module => {
        Module.wasm_setup();
  
        setImmediate(() => {
          // Websocket port
          network.Module = Module;
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
        console.log('Editor "%s" is connected.', network.editorID);
      };

      syncSocket.onmessage = function (event) {
        // console.log('Got message from sync:', event.data);
        network.Module.wasm_command(JSON.stringify({
          SyncClientCommand: JSON.parse(event.data),
        }));
      };

      syncSocket.onclose = network.onSyncClose;

      this.deferSyncResolve(syncSocket);
    });
  }
}