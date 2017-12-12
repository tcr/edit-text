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














function addto (el, then) {
  var p = el.parents('.mote');
  if (Array.isArray(then)) {
    var cur = then;
  } else {
    var cur = [then];
  }
  while (!el.is(p)) {
    if (el.prevAll().length > 0) {
      cur.unshift({
        "AddSkip": el.prevAll().length,
      });
    }
    el = el.parent();
    if (el.is(p)) {
      break;
    }
    cur = [{
      "AddWithGroup": cur,
    }];
  }
  return cur;
}

function delto (el, then) {
  var p = el.parents('.mote');
  if (Array.isArray(then)) {
    var cur = then;
  } else {
    var cur = [then];
  }
  while (!el.is(p)) {
    if (el.prevAll().length > 0) {
      cur.unshift({
        "DelSkip": el.prevAll().length,
      });
    }
    el = el.parent();
    if (el.is(p)) {
      break;
    }
    cur = [{
      "DelWithGroup": cur,
    }];
  }
  return cur;
}

class Editor {
  $elem: JQuery;
  ophistory;

  constructor($elem: JQuery) {
    this.$elem = $elem;
    this.ophistory = [];
  }

  op(d, a) {
    console.log(JSON.stringify([d, a]));
    this.ophistory.push([d, a]);
    setTimeout(() => {
      // Serialize by default the root element
      var match = serialize(this.$elem);
      // test
      var packet = [
        this.ophistory,
        match
      ];
      console.log(JSON.stringify(packet));

      $.ajax('/api/confirm', {
        data : JSON.stringify(packet),
        contentType : 'application/json',
        type : 'POST',
      })
      .done(function () {
        console.log('success', arguments);
        if (arguments[0] == '') {
          alert('Operation seemed to fail! Check console')
        }
      })
      .fail(function () {
        console.log('failure', arguments);
      });
    })
  }
}


function wrapContent(m: Editor) {
  let active = getActive();
  let target = getTarget();

  // if (isBlock(active)) {
    bootbox.prompt({
      title: "Wrap selected in tag:",
      value: "p",
      callback: (tag) => {
        if (tag) {
          var attrs = intoAttrs(tag);

          var out = active.nextAll().add(active).not($('.target').nextAll());
          clearActive();
          clearTarget();
          out.wrapAll(newElem(attrs));
          active = out.parent().addClass('active').addClass('target');
          m.op([], addto(active,
            {
              "AddGroup": [attrs, [
                {
                  "AddSkip": out.length
                }
              ]],
            }
          ));
        }
      },
    }).on("shown.bs.modal", function() {
      $(this).find('input').select();
    });
  // } else {
  //   alert('Not implemented?')
  // }
}

function deleteBlockPreservingContent(m: Editor) {
  let active = getActive();
  let target = getTarget();

  // Delete group while saving contents.
  if (isBlock(active)) {
    clearActive();
    clearTarget();

    m.op(delto(active,
      {
        "DelGroup": [
          {
            "DelSkip": active.children().length
          }
        ],
      }
    ), []);

    var first = active.children().first();
    var last = active.children().last();
    if (active.contents().length) {
      active.contents().unwrap();
    } else {
      active.remove();
    }
    active = first.addClass('active')[0] ? first : null;
    last.addClass('target');
  }
}

function deleteBlock(m: Editor) {
  const active = getActive();
  const target = getTarget();

  // Delete whole block.
  if (isBlock(active)) {
    m.op(delto(active,
      {
        "DelGroupAll": null,
      }
    ), []);

    active.remove();
    clearActive();
    clearTarget();
  }
}

function deleteChars(m: Editor) {
  const active = getActive();
  const target = getTarget();

  // Delete characters.
  if (isChar(active)) {
    clearActive();
    clearTarget();

    m.op(delto(active,
      {
        "DelChars": 1,
      }
    ), []);

    var prev = active.prev();
    var dad = active.parent();
    active.remove();
    $(prev[0] ? prev : dad[0] ? dad : null)
      .addClass('active')
      .addClass('target');
  }
}

function addBlockAfter(m: Editor) {
  const active = getActive();
  const target = getTarget();

  bootbox.prompt({
    title: "New tag to add after:",
    value: active.data('tag').toLowerCase(),
    callback: function (tag) {
      if (tag) {
        clearActive();
        clearTarget();

        newElem({tag: tag})
          .insertAfter(active)
          .addClass('active')
          .addClass('target');

        m.op([], addto(active.next(),
          {
            "AddGroup": [{"tag": tag}, []],
          }
        ));
      }
    }
  }).on("shown.bs.modal", function() {
    $(this).find('input').select();
  });
}

function splitBlock(m: Editor) {
  const active = getActive();
  const target = getTarget();

  bootbox.prompt({
    title: "New tag to split this into:",
    value: active.parent().data('tag').toLowerCase(),
    callback: function (tag) {
      if (tag) {
        var prev = active.prevAll().add(active);
        var next = active.nextAll();

        var parent = active.parent();

        var operation = [delto(parent, [
          {
            "DelGroup": [
              {
                "DelSkip": prev.length + next.length
              }
            ],
          },
        ]), addto(parent, [
          {
            "AddGroup": [{"tag": parent.data('tag')}, [
              {
                "AddSkip": prev.length
              }
            ]],
          },
          {
            "AddGroup": [{"tag": tag},
              next.length ?
                [{
                  "AddSkip": next.length
                }]
                : []
            ],
          }
        ])];

        clearActive();
        clearTarget();

        let newPrev = newElem({tag: parent.data('tag')});
        let newNext = newElem({tag: tag}).addClass('active').addClass('target');
        prev.wrapAll(newPrev);
        if (next.length) {
          next.wrapAll(newNext);
        } else {
          newNext.insertAfter(parent);
        }
        parent.contents().unwrap();

        m.op(operation[0], operation[1]);
      }
    },
  }).on("shown.bs.modal", function() {
    $(this).find('input').select();
  });
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
  const m = new Editor($elem);

  // switching button
  $('<button>Style Switch</button>')
    .appendTo($elem.prev())
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
  
  // Button shelf
  $('<div class="button-shelf">')
    .appendTo($elem.prev())

  // theme
  if (HashState.get().has(`${editorID}-theme-block`)) {
    $elem.addClass('theme-block');
  } else {
    $elem.addClass('theme-mock');
  }

  m.$elem.on('mousedown', 'span, div', function (e) {
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
      $(this).addClass('active').addClass('target')

      nativeCommand(TargetCommand(curto($(this))));
    }

    // TODO this bubbles if i use preventDEfault?
    return false;
  })

  $(document).on('keypress', (e) => {
    if ($(e.target).closest('.modal').length) {
      return;
    }

    const active = getActive();
    const target = getTarget();

    if (active && !active.parents('.mote').is(m.$elem)) {
      return
    }

    if (e.metaKey) {
      return;
    }

    nativeCommand(CharacterCommand(
      e.charCode,
      curto(active),
    ));

    e.preventDefault();
  });

  $(document).on('keydown', (e) => {
    if ($(e.target).closest('.modal').length) {
      return;
    }

    const active = getActive();
    const target = getTarget();

    if (active && !active.parents('.mote').is(m.$elem)) {
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

  return m.ophistory;
}

var m1 = $('#mote-1');

var ops_a = init(m1, 'left');

// Initial load
var exampleSocket;
$.get('/api/hello', data => {
  actionHello(data);

  exampleSocket = new WebSocket("ws://127.0.0.1:3012");
  exampleSocket.onopen = function (event) { 
    nativeCommand(LoadCommand(data));
  };
  exampleSocket.onmessage = onmessage;
  
});

// Reset button
$('#action-reset').on('click', () => {
  actionReset();
});

// Sync action
$('#action-sync').on('click', () => {
  actionSync();
})

function actionHello(data) {
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
    }
    //
  })
}

function actionSync() {
  // TODO fix this as ops_a, ops_b
  let packet = [
    ops_a,
    ops_a
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
      window.location.reload();
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

type CharacterCommand = {Character: [number, any]};

function CharacterCommand(
  charCode: number,
  curspan,
): CharacterCommand {
  return {
    'Character': [charCode, curspan],
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

type Command = RenameGroupCommand | KeypressCommand | CharacterCommand | TargetCommand | ButtonCommand | LoadCommand;

function nativeCommand(command: Command) {
  exampleSocket.send(JSON.stringify(command));
}

function onmessage (event) {
  let parse = JSON.parse(event.data);

  if (parse.Update) {
    console.log('update:', parse.Update);
    m1.empty().append(load(parse.Update[0]));
    // Load new op
    ops_a.push(parse.Update[1]);
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

    $('.button-shelf').each((_, x) => {
      parse.Setup.buttons.forEach(btn => {
        $('<button>').text(btn[1]).appendTo(x).click(_ => {
          nativeCommand(ButtonCommand(btn[0]));
        });
      })
    });
  }
}


