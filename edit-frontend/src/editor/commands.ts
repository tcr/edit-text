// Commands

import {CurSpan} from './editor';

export type Command
  = {type: 'Monkey', enabled: boolean}
  | {type: 'RenameGroup', tag: string, curspan: CurSpan}
  | {type: 'Keypress', key_code: number, meta_key: boolean, shift_key: boolean, alt_key: boolean}
  | {type: 'Character', char_code: number}
  | {type: 'Cursor', focus: Array<any> | null, anchor: Array<any> | null}
  | {type: 'Button', button: number}
  | {type: 'Load', load: any}
  | {type: 'Connect', client: string}
  | {type: 'InsertText', text: string}
  ;
