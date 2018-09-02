import * as React from 'react';
import copy from 'clipboard-copy';

import * as commands from './commands';
import * as util from './util';
import { ClientImpl } from './network';

const ROOT_SELECTOR = '.edit-text';

// TODO define this better
type Cursor = any;

function isTextNode(
  el: Node | null,
) {
  return el !== null && el.nodeType == 3;
}

function isBlock(
  el: Node | null,
) {
  //window.getComputedStyle((el as HTMLElement), null).display === 'block'

  if (el !== null && el.nodeType == 1 && (el as Element).tagName.toLowerCase() == 'div') {
    if ((el as HTMLElement).dataset['tag'] === 'caret') {
      return false;
    }
    return true;
  }
  return false;
}

function isEmptyBlock(
  el: Node | null
) {
  return isBlock(el) && (el as Element).querySelector('span') === null;
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

export type CurElement = any;

export type CurSpan = Array<CurElement>;

function curto(
  el: Node | null,
  textOffset: number | null = null,
): Cursor {
  // Null elements are null cursors.
  if (!el) {
    return null;
  }

  // Normalize text nodes at 0 to be spans...
  if (isTextNode(el) && textOffset == 0) {
    textOffset = null;
  }

  // Normalize leading spans to be their predecessor text node...
  // while (el !== null && textOffset === null) {
  //   if (isElement(el) && isBlock(el)) {
  //     break;
  //   }

  //   if (isTextNode(el)) {
  //     el = el.parentNode!;
  //     continue;
  //   }

  //   let prev: Node | null = el!.previousSibling;
  //   if (prev === null) {
  //     el = el!.parentNode!;
  //   } else if (isSpan(prev)) {
  //     el = prev.firstChild!;
  //     textOffset = charLength(el!);
  //     break;
  //   } else {
  //     el = prev;
  //   }
  // }

  // Is our cursor at a group or at a char?
  let cur: CurSpan = [
    isElement(el) ? {
      'CurGroup': null
    } : {
      'CurChar': null
    }
  ];

  // What is the character (or element) offset?
  if (textOffset !== null && textOffset > 0) {
    cur.unshift({
      "CurSkip": textOffset,
    });
  }

  if (isTextNode(el) && isSpan(el.parentNode)) {
    el = el.parentNode;
  }
  // TODO if isTextNode but !isSpan there should be an invariant thrown

  function place_skip(cur: CurSpan, value: number) {
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
      if (isSpan(el.previousSibling) || isTextNode(el.previousSibling)) {
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

  // console.log('result:', JSON.stringify(cur));
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
    
    // Enter empty block nodes.
    if (isEmptyBlock(node)) {
      return curto(node);
    }

    // Skip empty groups.
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
      offset,
    );
  }
}

export class Editor extends React.Component {
  props: {
    content: string,
    client: ClientImpl,
    KEY_WHITELIST: Array<any>,
    editorID: string,
    disabled: boolean,
  };

  el: HTMLElement;
  mouseDown = false;
  mouseDownActive = false;

  onClick(e: MouseEvent) {
    let option = e.ctrlKey || e.metaKey;
    let isAnchor = e.target ? util.matchesSelector(e.target as Node, '[data-style-Link]') : false;
    if (option && isAnchor) {
      let url = (e.target as HTMLElement).dataset['styleLink'];
      (window as any).open(url, '_blank').focus();
    }
  }

  onMouseDown(e: MouseEvent) {
    let option = e.ctrlKey || e.metaKey;
    if (option) {
      // Ignore, handle this in onClick
    } else {
      this.el.focus();
      this.mouseDown = true;
      this.onMouseMove(e, true);
    }
  }

  onMouseUp(e: MouseEvent) {
    this.mouseDown = false;
  }

  onMouseMove(e: MouseEvent, drop_anchor: boolean = false) {
    if (!this.mouseDown) {
      return;
    }

    if (this.mouseDownActive) {
      return;
    }
    this.mouseDownActive = true;

    // (window as any).EH = e;

    let text = util.textNodeAtPoint(e.clientX, e.clientY);
    let target = document.elementFromPoint(e.clientX, e.clientY);

    let dest = null;
    if (text !== null) {
      // We can focus on all text nodes.
      dest = resolveCursorFromPosition(text.textNode, text.offset);
    } else if (isEmptyBlock(target)) {
      // Or empty elements, which don't have a rooting text node.
      dest = curto(target as any);
    }

    // Generate the native commands.
    // console.log('### SUBMITTED:', JSON.stringify(resolveCursorFromPosition(pos.textNode, pos.offset)));
    if (dest !== null) {
      if (drop_anchor) {
        this.props.client.nativeCommand(commands.Cursor(dest, dest));
      } else {
        this.props.client.nativeCommand(commands.Cursor(dest, null));
      }
    } else {
      // No target found, stop dragging.
      this.mouseDownActive = false;
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
    // console.log('REFRESH');
    this.mouseDownActive = false;

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

      this.props.client.nativeCommand(commands.Character(e.charCode));
  
      e.preventDefault();
    });

    document.addEventListener('paste', (e: ClipboardEvent) => {
      if (self.props.disabled) {
        return;
      }

      const text = e.clipboardData.getData('text/plain');
      console.log('(c) got pasted text: ', text);
      this.props.client.nativeCommand(commands.InsertText(text));
    });
  
    document.addEventListener('keydown', (e) => {
      let current = document.querySelector('div.current[data-tag="caret"]');

      // Don't interfere when clicking the header.
      if (e.target !== null) {
        if (document.querySelector('#toolbar') && document.querySelector('#toolbar')!.contains(e.target! as Node)) {
          return;
        }
      }

      if (self.props.disabled) {
        return;
      }

      // Navigate command+c
      if (e.keyCode == 67 && e.metaKey) {
        // Generate string from selected text.
        let str = Array.from(
          document.querySelectorAll('span.Selected')
        )
          .map(x => (x as HTMLElement).innerText)
          .join('');
        
        // Debug
        console.error('copied: ' + JSON.stringify(str));

        copy(str)
        .then(res => {
          console.info('(c) clipboard successful copy');
        })
        .catch(err => {
          console.info('(c) clipboard unsuccessful copy:', err);
        });

        e.preventDefault();
        return;
      }

      // Check if this event exists in the list of whitelisted key combinations.
      let isWhitelisted = this.props.KEY_WHITELIST
        .some((x: any) => Object.keys(x).every((key: any) => (e as any)[key] == (x as any)[key]));
      if (!isWhitelisted) {
        return;
      }

      // Navigate up and down for text at the same column.
      let UP = e.keyCode == 38;
      let DOWN = e.keyCode == 40;
      if (UP || DOWN) {
        let root = document.querySelector('.edit-text')!;
        let current = document.querySelector('div.current[data-tag="caret"][data-focus="true"]');
        if (current !== null) {
          let rect = current.getBoundingClientRect();
          let y = UP ? rect.top + 5 : rect.bottom - 5;
          let x = rect.right;

          // Attempt to get the text node we are closest to
          let first: any = util.textNodeAtPoint(x, y);
          // In doc "# Most of all\n\nThe world is a place where parts of wholes are perscribed"
          // When you hit the down key for any character in the first line, it works,
          // until the last character (end of the line), where if you hit the down key it 
          // no longer works and the above turns null. Instead, this check once for the main case,
          // check at this offset for the edge case is weird but works well enough.
          if (first == null) {
            first = util.textNodeAtPoint(x - 2, y);
          }
          // Select empty blocks directly, which have no anchoring text nodes.
          if (first == null) {
            let el = document.elementFromPoint( x + 2, y);
            if (isEmptyBlock(el)) {
              first = el;
            }
          }

          if (first !== null) { // Or we have nothing to compare to and we'll loop all day
            while (true) {
              // Step a reasonable increment in each direction.
              const STEP = 10;
              y += UP ? -STEP : STEP;

              let el = document.elementFromPoint(x, y);
              // console.log('locating element at %d, %d:', x, y, el);
              if (!root.contains(el) || el === null) { // Off the page!
                break;
              }
              if (root !== el) {
                // Check when we hit a text node.
                let caret = util.textNodeAtPoint(x, y); // TODO should we reuse `first` here?
                let isTextNode = caret !== null && (first.textNode !== caret.textNode || first.offset !== caret.offset); // TODO would this comparison even work lol

                // Check for the "empty div" scenario
                let isEmptyDiv = isEmptyBlock(el);
                // if (isEmptyDiv) {
                //   console.log('----->', el.getBoundingClientRect());
                // }

                // console.log('attempted caret at', x, y, caret);
                if (isTextNode || isEmptyDiv) {
                  // console.log('CARET', caret);
                  e.preventDefault();

                  // TODO don't replicate events, because properties might
                  // be omitted that are desired (like target, etc.)
                  let mouseEvent = new MouseEvent('mousedown', {
                    clientX: x,
                    clientY: y,

                  });
                  this.onMouseDown(mouseEvent);
                  let mouseEvent2 = new MouseEvent('mouseup', {
                    clientX: x,
                    clientY: y,
                  });
                  this.onMouseUp(mouseEvent2);
                  return;
                }
              }
            }
          }
        }

        // Don't forward up/down keys.
        return;
      }
  
      // Forward the keypress to native.
      this.props.client.nativeCommand(commands.Keypress(
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
        tabIndex={0}
        ref={(el) => el && this.onMount(el)}
        onClick={this.onClick.bind(this)}
        onMouseDown={this.onMouseDown.bind(this)}
        onMouseUp={this.onMouseUp.bind(this)}
        onMouseMove={this.onMouseMove.bind(this)}
      />
    );
  }
}
