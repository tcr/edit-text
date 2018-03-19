import * as commands from './commands';
import HashState from './hashstate';
import Clipboard from 'clipboard';
import * as util from './util';

function curto(el: Node | null, textOffset: number | null = null) {
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
        place_skip(cur, (<any>el.previousSibling).data.length);
      } else {
        place_skip(cur, 1);
      }
      el = el.previousSibling;
    } else {
      el = el.parentNode;
      if (el.nodeType == 1 && (<any>el).mozMatchesSelector('.mote')) {
        break;
      }
      cur = [{
        "CurWithGroup": cur,
      }];
    }
  }

  if (!(el.nodeType == 1 && (<any>el).mozMatchesSelector('.mote'))) {
    console.error('Invalid selection!!!');
  }

  console.log('cursor', JSON.stringify(cur));
  return cur;
}

// Initialize child editor.
export default class Editor {
  $elem: any;
  editorID: string;
  ops: Array<any>;
  nativeSocket: WebSocket;
  syncSocket: WebSocket;
  KEY_WHITELIST: any;
  markdown: string;

  // TODO remove this
  Module: any;

  constructor(elem: HTMLElement, editorID: string) {
    this.$elem = $(elem);
    this.editorID = editorID;
    this.ops = [];
    this.KEY_WHITELIST = [];
    this.markdown = '';

    let editor = this;
    let $elem = this.$elem;

    {
      new Clipboard('#save-markdown', {
        text: function(trigger) {
          return editor.markdown;
        }
      });

      // Request markdown source.
      setInterval(() => {
        editor.nativeCommand(commands.RequestMarkdown());
      }, 2000);
      setTimeout(() => {
        // Early request
        editor.nativeCommand(commands.RequestMarkdown());
      }, 500);
    }

    // Markdown
    $('<button id="save-markdown">Save Markdown</button>')
      .appendTo($('#local-buttons'))
      .on('click', function () {
        let self = $(this);
        self.css('width', self.outerWidth());
        self.text('Copied!');
        setTimeout(() => {
          requestAnimationFrame(() => {
            self.text('Save Markdown');
            self.css('width', '');
          })
        }, 2000);
      });

    // CSS switch button
    $('<button>X-Ray</button>')
      .appendTo($('#local-buttons'))
      .on('click', function () {
        $elem.toggleClass('theme-mock');
        $elem.toggleClass('theme-block');

        const settings = HashState.get();
        if (settings.has(`${editorID}-theme-block`)) {
          settings.delete(`${editorID}-theme-v`);
        } else {
          settings.add(`${editorID}-theme-block`);
        }
        HashState.set(settings);
      });

    // Client Id.
    $('<b>Client: <kbd>' + editorID + '</kbd></b>')
      .appendTo($('#local-buttons'));

    // theme
    if (HashState.get().has(`${editorID}-theme-block`)) {
      $elem.addClass('theme-block');
    } else {
      $elem.addClass('theme-mock');
    }

    this.setupEditor();
  }

  setupEditor() {
    this.$elem.on('mousedown', (e) => {
      let pos = util.textNodeAtPoint(e.clientX, e.clientY);

      // Only support text elements.
      if (pos !== null) {
        // Text node
        let target = pos.textNode.parentNode;
        if (pos.offset == 0) {
          if (pos.textNode.previousSibling === null) {
            // Text node is first in element, so select parent node.
            this.nativeCommand(commands.TargetCommand(curto(
              pos.textNode.parentNode,
            )));
          } else if (pos.textNode.previousSibling.nodeType === 3) {
            // Text node has a preceding text elemnt; move to end.
            this.nativeCommand(commands.TargetCommand(curto(
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
          this.nativeCommand(commands.TargetCommand(curto(
            pos.textNode,
            pos.offset - 1,
          )));
        }
      }

      // TODO this bubbles if i use preventDEfault?
      window.focus();
      return false;
    });

    // Click outside the document area.
    $('#client').on('click', (e) => {
      if (e.target == $('#client')[0]) {
        let last = this.$elem.find('*').last()[0];
        this.nativeCommand(commands.TargetCommand(curto(last)));
      }
    });

    $(document).on('keypress', (e) => {
      if (e.metaKey) {
        return;
      }

      this.nativeCommand(commands.CharacterCommand(e.charCode,));

      e.preventDefault();
    });

    $(document).on('keydown', (e) => {
      console.log('KEYDOWN:', e.keyCode);

      // Match against whitelisted key entries.
      if (!this.KEY_WHITELIST.some(x => Object.keys(x).every(key => e[key] == x[key]))) {
        return;
      }

      this.nativeCommand(commands.KeypressCommand(
        e.keyCode,
        e.metaKey,
        e.shiftKey,
      ));
      
      e.preventDefault();
    });
  }

  setID(id: string) {
    // Update the client identifier
    $('kbd').text(id);
  }

  load(data: string) {
    let elem = this.$elem[0];
    requestAnimationFrame(() => {
      elem.innerHTML = data;
    });
  }

  nativeCommand(command: commands.Command) {
    if (this.Module) {
      this.Module.wasm_command({
        NativeCommand: command,
      });
    } else {
      this.nativeSocket.send(JSON.stringify(command));
    }
  }

  nativeConnect() {
    let editor = this;
    this.nativeSocket = new WebSocket(
      util.clientProxyUrl() + 
      (window.location.hash == '#helloworld' ? '?helloworld' : '')
    );
    this.nativeSocket.onopen = function (event) {
      console.log('Editor "%s" is connected.', editor.editorID);

      // editor.nativeCommand(commands.ConnectCommand(editor.editorID));

      // window.parent.postMessage({
      //   "Live": editor.editorID,
      // }, '*')
    };
    this.nativeSocket.onmessage = this.onNativeMessage.bind(this);
    this.nativeSocket.onclose = function () {
      $('body').css('background', 'red');
    }
  }

  // Received message on native socket
  onNativeMessage(event) {
    let editor = this;
    let parse = JSON.parse(event.data);
  
    if (parse.Init) {
      editor.setID(parse.Init);
    }

    else if (parse.Update) {
      editor.load(parse.Update[0]);
  
      if (parse.Update[1] == null) {
        console.log('Sync Update');
        editor.ops.splice(0, this.ops.length);

        requestAnimationFrame(() => {
          $(this.$elem).addClass('load-ping');
          requestAnimationFrame(() => {
            $(this.$elem).removeClass('load-ping');
          })
        });
      } else {
        editor.ops.push(parse.Update[1]);
      }
    }

    else if (parse.MarkdownUpdate) {
      editor.markdown = parse.MarkdownUpdate;
    }
    
    else if (parse.Setup) {
      console.log('SETUP', parse.Setup);
      editor.KEY_WHITELIST = parse.Setup.keys.map(x => ({
        keyCode: x[0],
        metaKey: x[1],
        shiftKey: x[2],
      }));
  
      $('#native-buttons').each((_, x) => {
        parse.Setup.buttons.forEach(btn => {
          $('<button>').text(btn[1]).appendTo(x).click(_ => {
            editor.nativeCommand(commands.ButtonCommand(btn[0]));
          });
        })
      });
    }

    else if (parse.SyncServerCommand) {
      editor.syncSocket.send(JSON.stringify(parse.SyncServerCommand));
    }

    else {
      console.error('Unknown packet:', parse);
    }
  }

  multiConnect() {
    window.onmessage = (event) => {
      let editor = this;

      // Sanity check.
      if (typeof event.data != 'object') {
        return;
      }

      // if ('Sync' in event.data) {
      //   // Push to native
      //   editor.nativeCommand(commands.LoadCommand(event.data.Sync))
      // }
      if ('Monkey' in event.data) {
        // TODO reflect this in the app
        editor.nativeCommand(commands.MonkeyCommand(event.data.Monkey));
      }
    };
  }
}
