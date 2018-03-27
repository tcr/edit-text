// Commands

export function RenameGroupCommand(tag: string, curspan) {
  return {
    tag: 'RenameGroupCommand' as 'RenameGroupCommand',
    'RenameGroup': [tag, curspan],
  }
}

export function KeypressCommand(
  keyCode: number,
  metaKey: boolean,
  shiftKey: boolean,
) {
  return {
    tag: 'KeypressCommand' as 'KeypressCommand',
    'Keypress': [keyCode, metaKey, shiftKey],
  }
}

export function CharacterCommand(
  charCode: number,
) {
  return {
    tag: 'CharacterCommand' as 'CharacterCommand',
    'Character': charCode,
  }
}

export function TargetCommand(
  curspan: [any],
) {
  return {
    tag: 'TargetCommand' as 'TargetCommand',
    'Target': curspan,
  }
}

export function ButtonCommand(
  button: number,
) {
  return {
    tag: 'ButtonCommand' as 'ButtonCommand',
    'Button': button,
  }
}

export function LoadCommand(
  load: any,
) {
  return {
    tag: 'LoadCommand' as 'LoadCommand',
    'Load': load,
  }
}

export function MonkeyCommand(
  enabled: boolean,
) {
  return {
    tag: 'MonkeyCommand' as 'MonkeyCommand',
    'Monkey': enabled,
  };
}

export function ConnectCommand(
  client: string,
) {
  return {
    tag: 'ConnectCommand' as 'ConnectCommand',
    'Connect': client,
  };
}

export function RequestMarkdown(
) {
  return {
    tag: 'RequestMarkdown' as 'RequestMarkdown',
    RequestMarkdown: null,
  };
}

export type Command
  = ReturnType<typeof MonkeyCommand>
  | ReturnType<typeof RenameGroupCommand>
  | ReturnType<typeof KeypressCommand>
  | ReturnType<typeof CharacterCommand>
  | ReturnType<typeof TargetCommand>
  | ReturnType<typeof ButtonCommand>
  | ReturnType<typeof LoadCommand>
  | ReturnType<typeof ConnectCommand>
  | ReturnType<typeof RequestMarkdown>
  ;
