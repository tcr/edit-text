import * as commands from './commands.ts';
import HashState from './hashstate.ts';

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


// Creates an HTML tree from a document tree.
function docToStrings(ret: Array<string>, vec: Array<any>) {
  // TODO act like doc
  // console.log(el);
  // var h = newElem(el.DocGroup[0]);
  for (var g = 0; g < vec.length; g++) {
    const el = vec[g];
    if (el.DocGroup) {
      const attrs = el.DocGroup[0];
      ret.push(`<div
        data-tag=${JSON.stringify(String(attrs.tag))}
        data-client=${JSON.stringify(String(attrs.client))}
        class=${JSON.stringify(String(attrs.class || ''))}
      >`);
      docToStrings(ret, el.DocGroup[1]);
      ret.push('</div>');
    } else if (el.DocChars) {
      for (var j = 0; j < el.DocChars.length; j++) {
        ret.push('<span>');
        ret.push(String(el.DocChars[j]));
        ret.push('</span>');
      }
    } else {
      throw new Error('unknown');
    }
  }
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
  KEY_WHITELIST: any;

  constructor($elem, editorID: string) {
    this.$elem = $elem;
    this.editorID = editorID;
    this.ops = [];
    this.KEY_WHITELIST = [];

    let editor = this;

    $('<b style="width: 200px; display: block">Client: ' + editorID + '</b>')
      .appendTo($('#local-buttons'));

    // monkey button
    let monkey = false;
    $('<button>Monkey</button>')
      .appendTo($('#local-buttons'))
      .on('click', function () {
        monkey = !monkey;
        editor.nativeCommand(commands.MonkeyCommand(monkey));
        $(this).css('font-weight') == '700'
          ? $(this).css('font-weight', 'normal')
          : $(this).css('font-weight', 'bold');
      })

    // switching button
    $('<button>Toggle Element View</button>')
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
        editor.nativeCommand(commands.TargetCommand(curto($(this))));
      }

      // TODO this bubbles if i use preventDEfault?
      window.focus();
      return false;
    })

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

  load(data: string) {
    let elem = this.$elem[0];
    requestAnimationFrame(() => {
      elem.innerHTML = data;
    });
  }

  nativeCommand(command: commands.Command) {
    this.nativeSocket.send(JSON.stringify(command));
  }

  nativeConnect() {
    let editor = this;
    this.nativeSocket = new WebSocket('ws://127.0.0.1:3012/' + editor.editorID);
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

    console.log(parse);
  
    if (parse.Update) {
      editor.load(parse.Update[0]);
  
      if (parse.Update[1] == null) {
        editor.ops.splice(0, this.ops.length);
      } else {
        editor.ops.push(parse.Update[1]);
      }
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
      editor.KEY_WHITELIST = parse.Setup.keys.map(x => ({keyCode: x[0], metaKey: x[1], shiftKey: x[2]}));
  
      $('#native-buttons').each((_, x) => {
        parse.Setup.buttons.forEach(btn => {
          $('<button>').text(btn[1]).appendTo(x).click(_ => {
            editor.nativeCommand(commands.ButtonCommand(btn[0]));
          });
        })
      });
    }
  }

  syncConnect() {
    window.onmessage = this.onSyncMessage.bind(this);
  }
  
  onSyncMessage(event) {
    let editor = this;

    // if ('Sync' in event.data) {
    //   // Push to native
    //   editor.nativeCommand(commands.LoadCommand(event.data.Sync))
    // }
    if ('Monkey' in event.data) {
      // TODO reflect this in the app
      editor.nativeCommand(commands.MonkeyCommand(true));
    }
  }
}
