import * as commands from './commands';
import * as util from './util';
import {Network, ProxyNetwork, WasmNetwork} from './network';
import * as React from 'react';

const ROOT_SELECTOR = '.edit-text';

// TODO define this better
type Cursor = any;

function isTextNode(
  el: Node | null,
) {
  return el !== null && el.nodeType == 3;
}

function isSpan(
  el: Node | null,
) {
  return el !== null && el.nodeType == 1 && (el as Element).tagName.toLowerCase() == 'span';
}

function isElement(
  el: Node | null,
) {
  return el !== null && el.nodeType == 1;
}

function charLength(
  node: Node,
): number {
  if (isTextNode(node)) {
    return (node as Text).data.length
  } else if (isSpan(node)) {
    return (node.childNodes[0] as Text).data.length;
  } else {
    throw new Error('Invalid');
  }
}

function curto(
  el: Node | null,
  textOffset: number | null = null,
): Cursor {
  if (!el) {
    return null;
  }

  // Move carets inside the element.
  if (isSpan(el)) {
    // assert(el.childNodes.length == 1);
    // assert(el.childNodes[0].nodeType === 3);
    el = el.firstChild;
    textOffset = 0;
  }

  // Is our cursor at a group or at a char?
  let cur: any = [
    isElement(el) ? {
      'CurGroup': null
    } : {
      'CurChar': null
    }
  ];

  // What is the character (or element) offset?
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

  // Find the offset to the start of the parent element by
  // iterating previous siblings.
  while (el !== null) {
    if (el.previousSibling) {
      // Skip previous sibling.
      if (isSpan(el.previousSibling)) {
        place_skip(cur, charLength(el.previousSibling));
      } else if (isElement(el.previousSibling)) {
        place_skip(cur, 1);
      } else {
        throw new Error('unreachable');
      }
      el = el.previousSibling;
    } else {
      // Move to parent node.
      el = el.parentNode;
      if (el === null) {
        throw new Error('Unexpectedly reached root');
      }
      if (!isSpan(el)) {
        if (isElement(el) && util.matchesSelector(el, ROOT_SELECTOR)) {
          break;
        }
        cur = [{
          "CurWithGroup": cur,
        }];
      }
    }
  }
  el = el!;

  if (!(isElement(el) && util.matchesSelector(el, ROOT_SELECTOR))) {
    console.error('Invalid selection!!!');
  }

  console.log('cursor', JSON.stringify(cur));
  return cur;
}

function resolveCursorFromPositionInner(
  node: Node | null,
  parent: Node,
): Cursor {
  if (node === null) {
    // Text node is first in element, so select parent node.
    return curto(parent);
  } else if (isTextNode(node)) {
    // Text node has a preceding text element; move to end.
    return resolveCursorFromPosition((node as Text), charLength(node));
  } else {
    // TODO can this be made simpler?
    // Skip empty elements.
    if (node.childNodes.length == 0) {
      return resolveCursorFromPositionInner(node.previousSibling, parent);
    }

    // Inspect previous element last to first.
    let child = node.childNodes[node.childNodes.length - 1];
    if (isTextNode(child)) {
      // Text node
      return resolveCursorFromPosition((child as Text), charLength(child));
    } else {
      // Element node
      return resolveCursorFromPositionInner(child, node);
    }
  }
}

function resolveCursorFromPosition(
  textNode: Text,
  offset: number,
): Cursor {
  if (offset == 0) {
    return resolveCursorFromPositionInner(textNode.previousSibling, textNode.parentElement!);
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
  mouseDown = false;

  onMouseDown(e: MouseEvent) {
    this.mouseDown = true;
    this.onMouseMove(e);
  }

  onMouseUp(e: MouseEvent) {
    this.mouseDown = false;
  }

  onMouseMove(e: MouseEvent) {
    if (!this.mouseDown) {
      return;
    }

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
      
      // Don't accept non-characters.
      if (e.charCode === 0) {
        return;
      }

      this.props.network.nativeCommand(commands.Character(e.charCode));
  
      e.preventDefault();
    });
  
    document.addEventListener('keydown', (e) => {
      let current = document.querySelector('div.current[data-tag="caret"]');

      if (self.props.disabled) {
        return;
      }

      // Check if this event exists in the list of whitelisted key combinations.
      if (!this.props.KEY_WHITELIST.some(x => Object.keys(x).every(key => e[key] == x[key]))) {
        return;
      }

      // Navigate up and down for text at the same column.
      let UP = e.keyCode == 38;
      let DOWN = e.keyCode == 40;
      if (UP || DOWN) {
        let root = document.querySelector('.edit-text')!;
        let current = document.querySelector('div.current[data-tag="caret"]');
        if (current !== null) {
          let rect = current.getBoundingClientRect();
          let y = UP ? rect.top : rect.bottom;
          let x = rect.right;

          // Temporary hack until the cursor at point can be focused on the cursor for real
          x += 1;

          let first = util.textNodeAtPoint(x, y);
          if (first !== null) { // Or we have nothing to compare to and we'll loop all day
            while (true) {
              y += UP ? -10 : 10; // STEP

              let el = document.elementFromPoint(x, y);
              console.log('locating element at %d, %d:', x, y, el);
              if (!root.contains(el) || el === null) { // Off the page!
                break;
              }
              if (root !== el) {
                let caret = util.textNodeAtPoint(x, y);
                // console.log('attempted caret at', x, y, caret);
                if (caret !== null && (first.textNode !== caret.textNode || first.offset !== caret.offset)) { // TODO would this comparison even work lol
                  // console.log('CARET', caret);
                  e.preventDefault();

                  let mouseEvent = new MouseEvent('mousedown', {
                    clientX: x,
                    clientY: y,
                  });
                  this.onMouseDown(mouseEvent);
                  return;
                }
              }
            }
          }
        }
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
        onMouseUp={this.onMouseUp.bind(this)}
        onMouseMove={this.onMouseMove.bind(this)}
      />
    );
  }
}
