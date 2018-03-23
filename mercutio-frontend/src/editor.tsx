import * as commands from './commands';
import Clipboard from 'clipboard';
import * as util from './util';
import * as interop from './interop';
import {Network, ProxyNetwork, WasmNetwork} from './network';
import * as React from 'react';

const ROOT_SELECTOR = '.edit-text';

// TODO define this better
type Cursor = any;

function curto(
  el: Node | null,
  textOffset: number | null = null,
): Cursor {
  if (!el) {
    return null;
  }

  let cur: any = [
    el.nodeType == 1 ? {
      'CurGroup': null
    } : {
      'CurChar': null
    }
  ];

  if (textOffset !== null) {
    console.log('help', textOffset);
    cur.unshift({
      "CurSkip": textOffset,
    });
  }

  function place_skip(cur, value) {
    if ('CurSkip' in cur[0]) {
      cur[0].CurSkip += value;
    } else {
      cur.unshift({
        "CurSkip": value,
      });
    }
  }

  while (el !== null) {
    if (el.previousSibling) {
      if (el.previousSibling.nodeType == 3) {
        place_skip(cur, (el.previousSibling as Text).data.length);
      } else {
        place_skip(cur, 1);
      }
      el = el.previousSibling;
    } else {
      el = el.parentNode;
      if (el === null) {
        throw new Error('Unexpectedly reached root');
      }
      if (el.nodeType == 1 && util.matchesSelector(el, ROOT_SELECTOR)) {
        break;
      }
      cur = [{
        "CurWithGroup": cur,
      }];
    }
  }
  el = el!;

  if (!(el.nodeType == 1 && util.matchesSelector(el, ROOT_SELECTOR))) {
    console.error('Invalid selection!!!');
  }

  console.log('cursor', JSON.stringify(cur));
  return cur;
}

function resolveCursorFromPosition(
  textNode: Text,
  offset: number,
): Cursor {
  if (offset == 0) {
    if (textNode.previousSibling === null) {
      // Text node is first in element, so select parent node.
      return curto(textNode.parentNode);
    } else if (textNode.previousSibling.nodeType === 3) {
      // Text node has a preceding text elemnt; move to end.
      return curto(
        textNode.previousSibling,
        (textNode.previousSibling as Text).data.length,
      );
    } else {
      // If it's an element...
      //TODO do something here,
      return curto(
        // This is literally just a random node
        // TODO replace this
        textNode.parentNode,
      );
    };
  } else {
    // Move to offset of this text node.
    return curto(
      textNode,
      offset - 1,
    );
  }
}

export class Editor extends React.Component {
  props: {
    content: string,
    network: Network,
    KEY_WHITELIST: any,
  };

  onMouseDown(e: MouseEvent) {
    let pos = util.textNodeAtPoint(e.clientX, e.clientY);

    // Only support text elements.
    if (pos !== null) {
      this.props.network.nativeCommand(commands.TargetCommand(
        resolveCursorFromPosition(pos.textNode, pos.offset),
      ));
    }

    // Focus the window despite us cancelling the event.
    window.focus();
    // Cancel the event; prevent text selection.
    e.preventDefault();

  }

  onMount(el: HTMLElement) {
    if (this.props.content) {
      el.innerHTML = this.props.content;
    }
  }

  componentDidMount() {
    document.addEventListener('keypress', (e: KeyboardEvent) => {
      // Don't accept keypresses when a modifier key is pressed w/keypress, except shift.
      if (e.metaKey) {
        return;
      }
  
      this.props.network.nativeCommand(commands.CharacterCommand(e.charCode));
  
      e.preventDefault();
    });
  
    document.addEventListener('keydown', (e) => {
      // Check if this event exists in the list of whitelisted key combinations.
      if (!this.props.KEY_WHITELIST.some(x => Object.keys(x).every(key => e[key] == x[key]))) {
        return;
      }
  
      // Forward the keypress to native.
      this.props.network.nativeCommand(commands.KeypressCommand(
        e.keyCode,
        e.metaKey,
        e.shiftKey,
      ));
      
      e.preventDefault();
    });
  }

  render() {
    return (
      <div
        ref={(el) => el && this.onMount(el)}
        onMouseDown={this.onMouseDown.bind(this)}
      />
    );
  }
}
