// Interfaces from frontend (JS) to client (wasm or proxy) and server (wasm or empty).

import 'react';

import {Command} from './commands';

export interface ControllerImpl {
  onMessage: (msg: any) => void | null;
  onClose: () => void | null;

  connect(onError: () => void): Promise<void>;
  sendCommand(command: Command): void;
}

export interface ServerImpl {
  onClose: () => void | null;
  connect(onError: (message: React.ReactNode) => void): Promise<void>;
  sendCommand(command: any): Promise<void>;
}

export class NullServer implements ServerImpl {
  onClose: () => void; // unused

  // The native server (the client proxy) handles sync traffic directly
  connect(onError: (message: React.ReactNode) => void): Promise<void> {
    return Promise.resolve();
  }

  // The native server (the client proxy) handles sync traffic directly
  sendCommand(command: any): Promise<void> {
    return Promise.resolve();
  }
}
