// Interfaces from frontend (JS) to client (wasm or proxy) and server (wasm or empty).

import 'react';

import {ControllerCommand, ServerCommand} from '../bindgen/edit_client';

export interface ControllerImpl {
  onMessage: (msg: any) => void | null;
  onClose: () => void | null;

  connect(onError: () => void): Promise<void>;
  sendCommand(command: ControllerCommand): void;
}

export interface ServerImpl {
  onClose: () => void | null;
  connect(onError: (message: React.ReactNode) => void): Promise<void>;
  sendCommand(command: ServerCommand): void;
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
