

// Commands
type RenameGroupCommand = {RenameGroup: any};

export function RenameGroupCommand(tag: string, curspan): RenameGroupCommand {
  return {
    'RenameGroup': [tag, curspan],
  }
}

type KeypressCommand = {Keypress: [number, boolean, boolean]};

export function KeypressCommand(
  keyCode: number,
  metaKey: boolean,
  shiftKey: boolean,
): KeypressCommand {
  return {
    'Keypress': [keyCode, metaKey, shiftKey],
  }
}

type CharacterCommand = {Character: number};

export function CharacterCommand(
  charCode: number,
): CharacterCommand {
  return {
    'Character': charCode,
  }
}

type TargetCommand = {Target: [any]};

export function TargetCommand(
  curspan,
): TargetCommand {
  return {
    'Target': curspan,
  }
}

type ButtonCommand = {Button: number};

export function ButtonCommand(
  button: number,
): ButtonCommand {
  return {
    'Button': button,
  }
}

type LoadCommand = {Load: any};

export function LoadCommand(
  load: any,
): LoadCommand {
  return {
    'Load': load,
  }
}

type MonkeyCommand = {Monkey: boolean};

export function MonkeyCommand(
  enabled: boolean,
): MonkeyCommand {
  return {
    'Monkey': enabled,
  };
}

type ConnectCommand = {Connect: string};

export function ConnectCommand(
  client: string,
): ConnectCommand {
  return {
    'Connect': client,
  };
}

type RequestMarkdown = {RequestMarkdown: null};

export function RequestMarkdown(
): RequestMarkdown {
  return {
    RequestMarkdown: null,
  };
}

export type Command
  = MonkeyCommand
  | RenameGroupCommand
  | KeypressCommand
  | CharacterCommand
  | TargetCommand
  | ButtonCommand
  | LoadCommand
  | ConnectCommand
  | RequestMarkdown;