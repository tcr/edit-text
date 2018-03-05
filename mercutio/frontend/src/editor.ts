import * as commands from './commands';
import HashState from './hashstate';
import Clipboard from 'clipboard';

function getActive() {
  var a = $('.active')
  return a[0] ? a : null;
}

function getTarget() {
  var a = $('.active')
  return a[0] ? a : null;
}

function isBlock ($active: JQuery) {
  return $active && $active[0].tagName == 'DIV';
}

function isChar ($active: JQuery) {
  return $active && $active[0].tagName == 'SPAN';
}

function isInline ($active) {
  return $active && $active.data('tag') == 'span';
}

function clearActive () {
  $(document).find('.active').removeClass('active');
}

function clearTarget () {
  $(document).find('.target').removeClass('target');
}

function curto(el: JQuery | null) {
  if (!el) {
    return null;
  }

  let then: any = el.is('div') ? {
    'CurGroup': null
  } : {
    'CurChar': null
  };

  var p = el.parents('.mote');
  if (Array.isArray(then)) {
    var cur = then;
  } else {
    var cur = [then];
  }
  while (!el.is(p)) {
    if (el.prevAll().length > 0) {
      cur.unshift({
        "CurSkip": el.prevAll().length,
      });
    }
    el = el.parent();
    if (el.is(p)) {
      break;
    }
    cur = [{
      "CurWithGroup": cur,
    }];
  }
  return cur;
}

// function serialize (parent) {
//   var out = []
//   $(parent).children().each(function () {
//     if ($(this).is('div')) {
//       out.push({
//         "DocGroup": [
//           serializeAttrs($(this)),
//           serialize(this),
//         ],
//       });
//     } else {
//       var txt = this.innerText
//       if (Object.keys(out[out.length - 1] || {})[0] == 'DocChars') {
//         txt = out.pop().DocChars + txt;
//       }
//       out.push({
//         "DocChars": txt
//       });
//     }
//   })
//   return out;
// }

function promptString(title, value, callback) {
  bootbox.prompt({
    title,
    value,
    callback,
  }).on("shown.bs.modal", function() {
    $(this).find('input').select();
  });
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

    let editor = this;
    let $elem = this.$elem;

    // monkey button
    // let monkey = false;
    // $('<button>Monkey</button>')
    //   .appendTo($('#local-buttons'))
    //   .on('click', function () {
    //     monkey = !monkey;
    //     editor.nativeCommand(commands.MonkeyCommand(monkey));
    //     $(this).css('font-weight') == '700'
    //       ? $(this).css('font-weight', 'normal')
    //       : $(this).css('font-weight', 'bold');
    //   });

    // MArkdown
    this.markdown = '';
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

    new (Clipboard as any)('#save-markdown', {
      text: function(trigger) {
        return editor.markdown;
      }
    });

    setInterval(() => {
      editor.nativeCommand(commands.RequestMarkdown());
    }, 2000);
    setTimeout(() => {
      // Early request
      editor.nativeCommand(commands.RequestMarkdown());
    }, 500);

    // switching button
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

    $elem.on('mousedown', 'span, div', function (e) {
      const active = getActive();
      const target = getTarget();

      if (e.shiftKey) {
        if (active && active.nextAll().add(active).is(this)) {
          clearTarget();
          $(this).addClass('target');

          // TODO
          // send target destination curspan
        }
      } else {
        clearActive();
        clearTarget();
        $(this).addClass('active').addClass('target');

        console.log('Cursor:', curto($(this)));
        let last = $(this).children().last();
        if (isBlock($(this))) {
          editor.nativeCommand(commands.TargetCommand(curto(last)));
        } else {
          editor.nativeCommand(commands.TargetCommand(curto($(this))));
        }
      }

      // TODO this bubbles if i use preventDEfault?
      window.focus();
      return false;
    })

    // Click outside the document area.
    $('#client').on('click', function (e) {
      if (e.target == this) {
        let last = $elem.find('*').last()[0];
        editor.nativeCommand(commands.TargetCommand(curto($(last))));
      }
    });

    $(document).on('keypress', (e) => {
      if ($(e.target).closest('.modal').length) {
        return;
      }

      const active = getActive();
      const target = getTarget();

      if (active && !active.parents('.mote').is($elem)) {
        return
      }

      if (e.metaKey) {
        return;
      }

      editor.nativeCommand(commands.CharacterCommand(e.charCode,));

      e.preventDefault();
    });

    $(document).on('keydown', (e) => {
      if ($(e.target).closest('.modal').length) {
        return;
      }

      const active = getActive();
      const target = getTarget();

      if (active && !active.parents('.mote').is($elem)) {
        return
      }

      console.log('KEYDOWN:', e.keyCode);

      // Match against whitelisted key entries.
      if (!editor.KEY_WHITELIST.some(x => Object.keys(x).every(key => e[key] == x[key]))) {
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
    let url =
      'ws://' +
      window.location.host.replace(/\:\d+/, ':8002') +
      '/' +
      window.location.pathname.replace(/^\/+/, '') +
      (window.location.hash == '#helloworld' ? '?helloworld' : '');
    this.nativeSocket = new WebSocket(url);
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
    
    else if (parse.PromptString) {
      promptString(parse.PromptString[0], parse.PromptString[1], (value) => {
        // Lookup actual key
        let key = Object.keys(parse.PromptString[2])[0];
        parse.PromptString[2][key][0] = value;
        editor.nativeCommand(parse.PromptString[2]);
      });
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

  syncConnect() {
    window.onmessage = this.onSyncMessage.bind(this);
  }
  
  onSyncMessage(event) {
    let editor = this;

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
  }
}
