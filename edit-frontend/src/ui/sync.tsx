// Talk to sync server, or to client proxy.

import * as React from 'react';

import * as route from './route';
import * as app from './app';
import {EditorFrame} from './app';
import * as commands from '../editor/commands';
import {ServerImpl, ClientImpl } from '../editor/network';
import {WasmClient, WasmError, getForwardWasmTaskCallback, setForwardWasmTaskCallback} from '../editor/wasm';

export class AppServer implements ServerImpl {
  client: WasmClient | null;
  
  onSyncClose: () => void;

  private nativeSocket: WebSocket;

  // Create a deferred object for the sync socket
  // because we may receive UserToSyncCommand payloads earlier
  private deferSync: Promise<WebSocket>;
  private deferSyncResolve: Function;

  private editorFrame: EditorFrame | null;

  constructor() {
    this.deferSync = new Promise(function(resolve, reject){
      this.deferSyncResolve = resolve;
    }.bind(this));
  }

  syncCommand(command: any) {
    return this.deferSync.then(syncSocket => {
      syncSocket.send(JSON.stringify(command));
    });
  }

  syncConnect(onError: (message: React.ReactNode) => void): Promise<void> {
    let server = this;

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
          if (getForwardWasmTaskCallback() != null) {
            if (server.client != null) {
              server.client.wasmClient.command(JSON.stringify({
                SyncToUserCommand: JSON.parse(event.data),
              }));
            }
          }
        } catch (e) {
          // Kill the current process, we triggered an exception.
          setForwardWasmTaskCallback(null);
          if (server.client != null) {
            server.client.Module.wasm_close();
          }
          // syncSocket.close();

          // TODO this is the wrong place to put this
          (document as any).body.background = 'red';

          if (server.editorFrame) {
            onError(
              <div>The client experienced an error talking to the server and you are now disconnected. We're sorry. You can <a href="?">refresh your browser</a> to continue.</div>
            );
          }

          throw new WasmError(e, `Error during sync command: ${e.message}`);
        }
      };

      syncSocket.onclose = function () {
        if (server.editorFrame) { 
          onError(
            <div>The editor has disconnected from the server. We're sorry. You can <a href="?">refresh your browser</a>, or we'll refresh once the server is reachable.</div>
          );
        }

        setTimeout(() => {
          setInterval(() => {
            app.graphqlPage('home').then(() => {
              // Can access server, continue
              window.location.reload();
            });
          }, 2000);
        }, 3000);

        server.onSyncClose();
      };

      this.deferSyncResolve(syncSocket);
    });
  }
}

export class ProxyClient implements ClientImpl {
  // TODO shouldn't these be nullable?
  onNativeMessage: (any) => void;
  onNativeClose: () => void;

  private editorID: string;

  private socket: WebSocket;

  nativeCommand(command: commands.Command) {
    delete command.tag;
    this.socket.send(JSON.stringify(command));
  }

  connect(onError: () => void): Promise<void> {
    let network = this;
    return Promise.resolve()
    .then(() => {
      this.socket = new WebSocket(
        route.clientProxyUrl()
      );
      this.socket.onopen = function (event) {
        console.debug('client-proxy socket opened.');
      };
      this.socket.onmessage = function (event) {
        let parse = JSON.parse(event.data);
        network.onNativeMessage(parse);
      };
      this.socket.onclose = network.onNativeClose;
    });
  }
}
