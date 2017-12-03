import 'bootstrap/dist/css/bootstrap.min.css';
import './mote.scss';

import $ from 'jquery';
import bootstrap from 'bootstrap';
import bootbox from 'bootbox';

// Consume bootstrap so bootbox works.
bootstrap;

function newElem(attrs) {
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


// Creates an HTML tree from a document tree.
function load (el) {
  // TODO act like doc
  // console.log(el);
  var h = newElem(el.DocGroup[0]);
  for (var i = 0; i < el.DocGroup[1].length; i++) {
    if (Object.keys(el.DocGroup[1][i])[0] == 'DocChars') {
      for (var j = 0; j < el.DocGroup[1][i].DocChars.length; j++) {
        h.append($('<span>').text(el.DocGroup[1][i].DocChars[j]));
      }
    } else {
      h.append(load(el.DocGroup[1][i]));
    }
  }
  return h;
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
      var match = serialize(this.$elem[0]);
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

  monkey() {
    let self = this;

    setTimeout(() => {
      // Serialize by default the root element
      var match = serialize(this.$elem[0]);
      // test
      var packet = [
        this.ophistory,
        match
      ];
      console.log(JSON.stringify(packet));

      $.ajax('/api/random', {
        data : JSON.stringify(packet),
        contentType : 'application/json',
        type : 'POST',
      })
      .done(function (data) {
        if (arguments[0] == '') {
          alert('Operation seemed to fail! Check console')
        } else {
          self.$elem.empty().append(load(data.doc[0]));
          self.ophistory.push(data.op);
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

function renameBlock(m: Editor) {
  let active = getActive();
  let target = getTarget();

  if (active) {
    bootbox.prompt({
      title: "Rename tag group:",
      value: "p",
      callback: (tag) => {
        if (tag) {
          let attrs = intoAttrs(tag);

          m.op(delto(active,
            {
              "DelGroup": [
                {
                  "DelSkip": active.children().length
                }
              ],
            }
          ), addto(active,
            {
              "AddGroup": [attrs, [
                {
                  "AddSkip": active.children().length
                }
              ]],
            }
          ));

          modifyElem(active, attrs);
        }
      },
    }).on("shown.bs.modal", function() {
      $(this).find('input').select();
    });
  }
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
            "AddGroup": [{"tag": tag}, [
              {
                "AddSkip": next.length
              }
            ]],
          }
        ])];

        clearActive();
        clearTarget();

        console.log(prev);
        prev.wrapAll(newElem({tag: parent.data('tag')}));
        next.wrapAll(newElem({tag: tag}).addClass('active').addClass('target'));
        parent.contents().unwrap();

        m.op(operation[0], operation[1]);
      }
    },
  }).on("shown.bs.modal", function() {
    $(this).find('input').select();
  });
}

function init ($elem) {
  const m = new Editor($elem);

  // switching button
  $('<button>Style Switch</button>')
    .appendTo($elem.prev())
    .on('click', function () {
      $elem.toggleClass('theme-mock');
      $elem.toggleClass('theme-block');    
    });
    
  // monkey button
  $('<button>Monkey</button>')
    .appendTo($elem.prev())
    .on('click', function () {
      m.monkey();
    });

  // theme
  $elem.addClass('theme-block');

  m.$elem.on('click', 'span, div', function (e) {
    const active = getActive();
    const target = getTarget();

    if (e.shiftKey) {
      if (active && active.nextAll().add(active).is(this)) {
        clearTarget();
        $(this).addClass('target');
      }
    } else {
      clearActive();
      clearTarget();
      $(this).addClass('active').addClass('target')
    }
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

    if ([13, 37, 38, 39, 40].indexOf(e.keyCode) > -1) {
      return;
    }

    if (e.metaKey) {
      return;
    }

    let txt = String.fromCharCode(e.charCode);
    let span = $('<span>').text(txt).addClass('active').addClass('target');
    if (isBlock(active)) {
      clearActive();
      clearTarget();
      active.prepend(span);

      m.op([], addto(span,
        {
          "AddChars": txt
        }
      ));
    } else if (isChar(active)) {
      clearActive();
      clearTarget();
      span.insertAfter(active);

      m.op([], addto(span,
        {
          "AddChars": txt
        }
      ));
    }
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

    console.log('KEY:', e.keyCode);

    // command+,
    if (e.keyCode == 188 && e.metaKey) {
      e.preventDefault();
      wrapContent(m);
      return false;
    }

    // command+.
    if (e.keyCode == 190 && e.metaKey) {
      e.preventDefault();
      renameBlock(m);
      return false;
    }
    if (e.keyCode == 8) {
      e.preventDefault();
      if (e.shiftKey) {
        deleteBlockPreservingContent(m);
      } else if (e.metaKey) {
        deleteBlock(m);
      } else {
        deleteChars(m);
      }
      return false;
    }

    // <enter>
    if (e.keyCode == 13) {
      e.preventDefault();
      if (e.shiftKey && isBlock(active)) {
        addBlockAfter(m);
      } else if (!e.shiftKey) {
        splitBlock(m);
      }
      return false;
    }

    // Arrow keys
    if ([37, 38, 39, 40].indexOf(e.keyCode) > -1) {
      e.preventDefault();

      const keyCode = e.keyCode;

      // left
      if (keyCode == 37) {
        let last = active.prevAll().find('span').addBack('span').last();
        if (!last[0] && serializeAttrs(active.parent()).tag == 'span') {
          last = active.parent().prevAll().find('span').addBack('span').last();
        }
        if (last[0]) {
          let prev = last[0];
          clearActive();
          clearTarget();
          $(prev).addClass('active').addClass('target')
        }
      }
      // right
      if (keyCode == 39) {
        let next = active.nextAll().find('span').addBack('span').first();
        if (!next[0] && serializeAttrs(active.parent()).tag == 'span') {
          next = active.parent().nextAll().find('span').addBack('span').first();
        }
        if (next[0]) {
          let prev = next[0];
          clearActive();
          clearTarget();
          $(prev).addClass('active').addClass('target')
        }
      }

      // up
      if (keyCode == 38) {
        if (active.parent()[0] && !active.parent().hasClass('mote')) {
          let prev = active.parent()[0];
          clearActive();
          clearTarget();
          $(prev).addClass('active').addClass('target')
        }
      }
      // down
      if (keyCode == 40) {
        if (active.children()[0]) {
          let prev = active.children()[0];
          clearActive();
          clearTarget();
          $(prev).addClass('active').addClass('target')
        }
      }
    }
  })

  return m.ophistory;
}

var m1 = $('#mote-1');
var m2 = $('#mote-2');

var ops_a = init(m1);
var ops_b = init(m2);

$.get('/api/hello', data => {
    m1.empty().append(load(data));
    m2.empty().append(load(data));
});

$('#action-reset').on('click', () => {
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
});

$('#action-sync').on('click', () => {
  let packet = [
    ops_a,
    ops_b
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
})
