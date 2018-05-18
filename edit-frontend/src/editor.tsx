import * as commands from './commands';
import * as util from './util';
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

function resolveCursorFromPositionPrevious(
  node: Node | null,
  parent: Node,
): Cursor {
  if (node === null) {
    // Text node is first in element, so select parent node.
    return curto(parent);
  } else if (node.nodeType === 3) {
    // Text node has a preceding text element; move to end.
    return resolveCursorFromPosition((node as Text), (node as Text).data.length);
  } else {
    // TODO can this be made simpler?
    // Skip empty elements.
    if (node.childNodes.length == 0) {
      return resolveCursorFromPositionPrevious(node.previousSibling, parent);
    }

    // Inspect previous element last to first.
    let child = node.childNodes[node.childNodes.length - 1];
    if (child.nodeType === 3) {
      // Text node
      return resolveCursorFromPosition((child as Text), (child as Text).data.length);
    } else {
      // Element node
      return resolveCursorFromPositionPrevious(child, node);
    }
  }
}

function resolveCursorFromPosition(
  textNode: Text,
  offset: number,
): Cursor {
  if (offset == 0) {
    return resolveCursorFromPositionPrevious(textNode.previousSibling, textNode.parentElement!);
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
    editorID: string,
    disabled: boolean,
  };

  el: HTMLElement;

  onMouseDown(e: MouseEvent) {
    let pos = util.textNodeAtPoint(e.clientX, e.clientY);

    // Only support text elements.
    if (pos !== null) {
      this.props.network.nativeCommand(commands.Target(
        resolveCursorFromPosition(pos.textNode, pos.offset),
      ));
    }

    // Focus the window despite us cancelling the event.
    window.focus();
    // Cancel the event; prevent text selection.
    e.preventDefault();

  }

  onMount(el: HTMLElement) {
    this.el = el;
    this.el.innerHTML = this.props.content;
  }
  
  componentDidUpdate() {
    this.el.innerHTML = this.props.content;

    // Highlight our own caret.
    document.querySelectorAll(
      `div[data-tag="caret"][data-client=${JSON.stringify(this.props.editorID)}]`,
    ).forEach(caret => {
      caret.classList.add("current");
    });
  }

  componentDidMount() {
    let self = this;
    document.addEventListener('keypress', (e: KeyboardEvent) => {
      if (self.props.disabled) {
        return;
      }

      // Don't accept keypresses when a modifier key is pressed w/keypress, except shift.
      if (e.metaKey) {
        return;
      }
  
      this.props.network.nativeCommand(commands.Character(e.charCode));
  
      e.preventDefault();
    });
  
    document.addEventListener('keydown', (e) => {
      if (self.props.disabled) {
        return;
      }

      // Check if this event exists in the list of whitelisted key combinations.
      if (!this.props.KEY_WHITELIST.some(x => Object.keys(x).every(key => e[key] == x[key]))) {
        return;
      }
  
      // Forward the keypress to native.
      this.props.network.nativeCommand(commands.Keypress(
        e.keyCode,
        e.metaKey,
        e.shiftKey,
        e.altKey,
      ));
      
      e.preventDefault();
    });
  }

  render() {
    return (
      <div
        className="edit-text theme-mock"
        ref={(el) => el && this.onMount(el)}
        onMouseDown={this.onMouseDown.bind(this)}
      />
    );
  }
}
