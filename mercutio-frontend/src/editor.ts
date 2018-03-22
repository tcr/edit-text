import * as commands from './commands';
import Clipboard from 'clipboard';
import * as util from './util';
import * as interop from './interop';
import {Network, ProxyNetwork, WasmNetwork} from './network';

const ROOT_SELECTOR = '.edit-text';

function curto(
  el: Node | null,
  textOffset: number | null = null,
) {
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
        place_skip(cur, (<Text>el.previousSibling).data.length);
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

export function editorSetup(
  element: Element,
  network: Network,
  KEY_WHITELIST: any,
) {
  element.addEventListener('mousedown', (e: MouseEvent) => {
    let pos = util.textNodeAtPoint(e.clientX, e.clientY);

    // Only support text elements.
    if (pos !== null) {
      // Text node
      let target = pos.textNode.parentNode;
      if (pos.offset == 0) {
        if (pos.textNode.previousSibling === null) {
          // Text node is first in element, so select parent node.
          network.nativeCommand(commands.TargetCommand(curto(
            pos.textNode.parentNode,
          )));
        } else if (pos.textNode.previousSibling.nodeType === 3) {
          // Text node has a preceding text elemnt; move to end.
          network.nativeCommand(commands.TargetCommand(curto(
            pos.textNode.previousSibling,
            (<Text>pos.textNode.previousSibling).data.length,
          )))
        } else {
          // If it's an element...
          //TODO do something here
          console.log('recursive depth');
        };
      } else {
        // Move to offset of this text node.
        network.nativeCommand(commands.TargetCommand(curto(
          pos.textNode,
          pos.offset - 1,
        )));
      }
    }

    // TODO Why do we call window.focus?
    window.focus();
    // TODO Why do we call e.preventDefault() ?
    e.preventDefault();
  });

  // Click outside the document area.
  // $('#client').on('click', (e) => {
  //   if (e.target == $('#client')[0]) {
  //     let last = this.$elem.find('*').last()[0];
  //     network.nativeCommand(commands.TargetCommand(curto(last)));
  //   }
  // });

  document.addEventListener('keypress', (e: KeyboardEvent) => {
    // Don't accept keypresses when a modifier key is pressed, except shift.
    if (e.metaKey) {
      return;
    }

    network.nativeCommand(commands.CharacterCommand(e.charCode));

    e.preventDefault();
  });

  document.addEventListener('keydown', (e) => {
    console.log('KEYDOWN:', e.keyCode);

    // Match against whitelisted key entries.
    if (!KEY_WHITELIST.some(x => Object.keys(x).every(key => e[key] == x[key]))) {
      return;
    }

    // Forward the keypress to native.
    network.nativeCommand(commands.KeypressCommand(
      e.keyCode,
      e.metaKey,
      e.shiftKey,
    ));
    
    e.preventDefault();
  });
}
