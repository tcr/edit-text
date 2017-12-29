import 'bootstrap/dist/css/bootstrap.min.css';
import './mote.scss';

import $ from 'jquery';
import bootstrap from 'bootstrap';
import bootbox from 'bootbox';

// Consume bootstrap so bootbox works.
bootstrap;

// Hashtag state

class HashState {
  static get(): Set<String> {
    return new Set((location.hash || '')
      .replace(/^#/, '')
      .split(',')
      .map(x => x.replace(/^\s+|\s+$/g, ''))
      .filter(x => x.length));
  }

  static set(input: Set<String>) {
    location.hash = Array.from(input).join(',');
  }
}

// Elements

function newElem(attrs): JQuery {
  return modifyElem($('<div>'), attrs);
}

function modifyElem(elem, attrs) {
  return elem
    .attr('data-tag', attrs.tag)
    .attr('data-client', attrs.client)
    .attr('class', attrs.class || '');
}

function serializeAttrs(elem: JQuery) {
  return {
    "tag": String(elem.attr('data-tag') || ''),
  };
}

function intoAttrs(str: string) {
  if (str == 'i') {
    return {
      "tag": "span",
      "class": "italic",
    }
  } else if (str == 'b') {
    return {
      "tag": "span",
      "class": "bold",
    }
  } else {
    return {
      "tag": str,
    };
  }
}



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
function load(vec): Array<JQuery> {
  // TODO act like doc
  // console.log(el);
  // var h = newElem(el.DocGroup[0]);
  let ret = [];
  for (var g = 0; g < vec.length; g++) {
    const el = vec[g];
    if (el.DocGroup) {
      var h = newElem(el.DocGroup[0]);
      h.append(load(el.DocGroup[1]));
      ret.push(h);
    } else if (el.DocChars) {
      for (var j = 0; j < el.DocChars.length; j++) {
        ret.push($('<span>').text(el.DocChars[j]));
      }
    } else {
      throw new Error('unknown');
    }
  }
  return ret;
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

function serialize (parent) {
  var out = []
  $(parent).children().each(function () {
    if ($(this).is('div')) {
      out.push({
        "DocGroup": [
          serializeAttrs($(this)),
          serialize(this),
        ],
      });
    } else {
      var txt = this.innerText
      if (Object.keys(out[out.length - 1] || {})[0] == 'DocChars') {
        txt = out.pop().DocChars + txt;
      }
      out.push({
        "DocChars": txt
      });
    }
  })
  return out;
}










let KEY_WHITELIST = [];

function promptString(title, value, callback) {
  bootbox.prompt({
    title,
    value,
    callback,
  }).on("shown.bs.modal", function() {
    $(this).find('input').select();
  });
}

function init ($elem, editorID: string) {
  // monkey button
  let monkey = false;
  $('<button>Monkey</button>')
    .appendTo($('#local-buttons'))
    .on('click', function () {
      monkey = !monkey;
      nativeCommand(MonkeyCommand(monkey));
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
      nativeCommand(TargetCommand(curto($(this))));
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

    nativeCommand(CharacterCommand(e.charCode,));

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
    if (!KEY_WHITELIST.some(x => Object.keys(x).every(key => e[key] == x[key]))) {
      return;
    }

    nativeCommand(KeypressCommand(
      e.keyCode,
      e.metaKey,
      e.shiftKey,
    ));
    
    e.preventDefault();
  })

  // todo
  return [];
}

// Reset button
$('#action-reset').on('click', () => {
  actionReset();
});

$('#action-monkey').on('click', () => {
  for (let i = 0; i < window.frames.length; i++) {
    window.frames[i].postMessage({
      'Monkey': {}
    }, '*');
  }
})

function actionHello(m1, data) {
  m1.empty().append(load(data));
}

function actionReset() {
  $.ajax('/api/reset', {
    contentType : 'application/json',
    type : 'POST',
  })
  .done(function (data, _2, obj) {
    if (obj.status == 200 && data != '') {
      window.location.reload();
    } else {
      alert('Error in resetting. Check the console.')
      window.stop();
    }
    //
  })
}

function actionSync(ops_a, ops_b) {
  let packet = [
    ops_a,
    ops_b,
  ];

  console.log('PACKET', packet)

  $.ajax('/api/sync', {
    data : JSON.stringify(packet),
    contentType : 'application/json',
    type : 'POST',
  })
  .done(function (data, _2, obj) {
    console.log('success', arguments);
    if (obj.status == 200 && data != '') {
      
      // window.location.reload();

      // Get the new document state and update the two clients
      for (let i = 0; i < window.frames.length; i++) {
        window.frames[i].postMessage({
          'Sync': data.doc
        }, '*');
      }
    } else {
      alert('Error in syncing. Check the command line.')
    }
    //
  })
  .fail(function () {
    console.log('failure', arguments);
    alert('Error in syncing. Check the command line.')
  });
}


// Commands
type RenameGroupCommand = {RenameGroup: any};

function RenameGroupCommand(tag: string, curspan): RenameGroupCommand {
  return {
    'RenameGroup': [tag, curspan],
  }
}

type KeypressCommand = {Keypress: [number, boolean, boolean]};

function KeypressCommand(
  keyCode: number,
  metaKey: boolean,
  shiftKey: boolean,
): KeypressCommand {
  return {
    'Keypress': [keyCode, metaKey, shiftKey],
  }
}

type CharacterCommand = {Character: number};

function CharacterCommand(
  charCode: number,
): CharacterCommand {
  return {
    'Character': charCode,
  }
}

type TargetCommand = {Target: [any]};

function TargetCommand(
  curspan,
): TargetCommand {
  return {
    'Target': curspan,
  }
}

type ButtonCommand = {Button: number};

function ButtonCommand(
  button: number,
): ButtonCommand {
  return {
    'Button': button,
  }
}

type LoadCommand = {Load: any};

function LoadCommand(
  load: any,
): LoadCommand {
  return {
    'Load': load,
  }
}

type MonkeyCommand = {Monkey: boolean};

function MonkeyCommand(
  enabled: boolean,
): MonkeyCommand {
  return {
    'Monkey': enabled,
  }
}

type Command
  = MonkeyCommand
  | RenameGroupCommand
  | KeypressCommand
  | CharacterCommand
  | TargetCommand
  | ButtonCommand
  | LoadCommand;

function nativeCommand(command: Command) {
  exampleSocket.send(JSON.stringify(command));
}

function onmessage (m1, ops_a, event) {
  let parse = JSON.parse(event.data);

  if (parse.Update) {
    m1.empty().append(load(parse.Update[0]));

    if (parse.Update[1] == null) {
      ops_a.splice(0, ops_a.length);
    } else {
      ops_a.push(parse.Update[1]);
    }

    window.parent.postMessage({
      Update: {
        doc: parse.Update[0], 
        ops: ops_a,
        name: window.name,
        version: parse.Update[2],
      },
    }, '*');
  }
  
  else if (parse.PromptString) {
    promptString(parse.PromptString[0], parse.PromptString[1], (value) => {
      // Lookup actual key
      let key = Object.keys(parse.PromptString[2])[0];
      parse.PromptString[2][key][0] = value;
      nativeCommand(parse.PromptString[2]);
    });
  }
  
  else if (parse.Setup) {
    console.log('SETUP', parse.Setup);
    KEY_WHITELIST = parse.Setup.keys.map(x => ({keyCode: x[0], metaKey: x[1], shiftKey: x[2]}));

    $('#native-buttons').each((_, x) => {
      parse.Setup.buttons.forEach(btn => {
        $('<button>').text(btn[1]).appendTo(x).click(_ => {
          nativeCommand(ButtonCommand(btn[0]));
        });
      })
    });
  }
}

let counter = 0;
setInterval(() => {
  $('#timer').each(function () {
    $(this).text(counter++ + 's');
  })
}, 1000);


if ((<any>window).MOTE_ENTRY == 'index') {
  document.body.style.background = '#eee';

  let cache = (<any>{});

  // TODO get this from the initial load
  let curversion = 101;

  window.onmessage = function (data) {
    let name = data.data.Update.name;
    cache[name] = data.data.Update;
  };

  // Sync action
  // $('#action-sync').on('click', () => {
  //   console.log('click', cache);
  //   if (!cache.left || !cache.right) {
  //     return;
  //   }
  //   actionSync(cache.left, cache.right);
  //   delete cache.left;
  //   delete cache.right;
  //   curversion += 1;
  // })

  setInterval(function () {
    if ((!cache.left || cache.left.version != curversion) ||
      (!cache.right || cache.right.version != curversion)) {
      console.log('outdated, skipping:', cache.left, cache.right);
      return;
    }
    curversion += 1;
    actionSync(cache.left.ops, cache.right.ops);
  }, 250)
}
else if ((<any>window).MOTE_ENTRY == 'client') {
  var m1 = $('#mote');

  // Receive messages from parent window.
  window.onmessage = function (event) {
    if ('Sync' in event.data) {
      // Push to native
      nativeCommand(LoadCommand(event.data.Sync))
    }
    if ('Monkey' in event.data) {
      // TODO reflect this in the app
      nativeCommand(MonkeyCommand(true));
    }
  };

  $(window).on('focus', () => $(document.body).addClass('focused'));
  $(window).on('blur', () => $(document.body).removeClass('focused'));
  
  var ops_a = init(m1, window.name);
  
  // Initial load
  var exampleSocket;
  $.get('/api/hello', data => {
    actionHello(m1, data);

    // Initial load.
    window.parent.postMessage({
      Update: {
        doc: data,
        ops: ops_a,
        name: window.name
      },
    }, '*');
  
    exampleSocket = new WebSocket(window.name == 'left' ? "ws://127.0.0.1:3012" : 'ws://127.0.0.1:3013');
    exampleSocket.onopen = function (event) {
      nativeCommand(LoadCommand(data));
    };
    exampleSocket.onmessage = onmessage.bind(exampleSocket, m1, ops_a);
    exampleSocket.onclose = function () {
      $('body').css('background', 'red');
    }
  });  
}