// Interface for talking to WebAssembly.

import * as commands from './commands';

export interface ClientImpl {
  onNativeMessage: (any) => void;
  onNativeClose: () => void;

  nativeConnect(onError: () => void): Promise<void>;
  nativeCommand(command: commands.Command): void;
}
