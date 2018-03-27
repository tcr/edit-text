// Commands

export function RenameGroup(tag: string, curspan) {
  return {
    tag: 'RenameGroup' as 'RenameGroup',
    'RenameGroup': [tag, curspan],
  }
}

export function Keypress(
  keyCode: number,
  metaKey: boolean,
  shiftKey: boolean,
) {
  return {
    tag: 'Keypress' as 'Keypress',
    'Keypress': [keyCode, metaKey, shiftKey],
  }
}

export function Character(
  charCode: number,
) {
  return {
    tag: 'Character' as 'Character',
    'Character': charCode,
  }
}

export function Target(
  curspan: [any],
) {
  return {
    tag: 'Target' as 'Target',
    'Target': curspan,
  }
}

export function Button(
  button: number,
) {
  return {
    tag: 'Button' as 'Button',
    'Button': button,
  }
}

export function Load(
  load: any,
) {
  return {
    tag: 'Load' as 'Load',
    'Load': load,
  }
}

export function Monkey(
  enabled: boolean,
) {
  return {
    tag: 'Monkey' as 'Monkey',
    'Monkey': enabled,
  };
}

export function Connect(
  client: string,
) {
  return {
    tag: 'Connect' as 'Connect',
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
  = ReturnType<typeof Monkey>
  | ReturnType<typeof RenameGroup>
  | ReturnType<typeof Keypress>
  | ReturnType<typeof Character>
  | ReturnType<typeof Target>
  | ReturnType<typeof Button>
  | ReturnType<typeof Load>
  | ReturnType<typeof Connect>
  | ReturnType<typeof RequestMarkdown>
  ;
