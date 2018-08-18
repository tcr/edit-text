// Interfaces from frontend (JS) to client (wasm or proxy) and server (wasm or empty).

import 'react';

import {Command} from './commands';

export interface ClientImpl {
  onNativeMessage: (any) => void;
  onNativeClose: () => void;

  connect(onError: () => void): Promise<void>;
  nativeCommand(command: Command): void;
}

export interface ServerImpl {
  onSyncClose: () => void;
  syncConnect(onError: (message: React.ReactNode) => void): Promise<void>;
  syncCommand(command: any): Promise<void>;
}

export class NullServer implements ServerImpl {
  onSyncClose: () => void; // unused

  // The native server (the client proxy) handles sync traffic directly
  syncConnect(onError: (message: React.ReactNode) => void): Promise<void> {
    return Promise.resolve();
  }

  // The native server (the client proxy) handles sync traffic directly
  syncCommand(command: any): Promise<void> {
    return Promise.resolve();
  }
}
