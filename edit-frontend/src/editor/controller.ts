// Interfaces from frontend (JS) to client (wasm or proxy) and server (wasm or empty).

import 'react';

import { ControllerCommand } from '../bindgen/edit_client';

export interface ControllerImpl {
  onMessage: (msg: any) => void | null;
  onClose: () => void | null;
  onError: (error: any) => void | null;

  connect(onError: () => void): Promise<void>;
  sendCommand(command: ControllerCommand): void;
}
