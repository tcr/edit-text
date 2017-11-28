
function load (el) {
  // TODO act like doc
  // console.log(el);
  var h = $('<div>').attr('data-tag', el.DocGroup[0].tag);
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

function init (m) {

  function serialize (parent) {
    if (!parent) {
      parent = m[0];
    }
    var out = []
    $(parent).children().each(function () {
      if ($(this).is('div')) {
        out.push({
          "DocGroup": [{"tag": $(this).attr('data-tag')}, serialize(this)],
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

  function getActive() {
    var a = $('.active')
    return a[0] ? a : null;
  }

  function getTarget() {
    var a = $('.active')
    return a[0] ? a : null;
  }

  function isBlock ($active) {
    return $active && $active[0].tagName == 'DIV'
  }

  function isChar ($active) {
    return $active && $active[0].tagName == 'SPAN'
  }

  function isInline ($active) {
    return $active && ($active.data('tag') == 'b' || $active.data('tag') == 'i')
  }

  function clearActive () {
    $(document).find('.active').removeClass('active');
  }

  function clearTarget () {
    $(document).find('.target').removeClass('target');
  }

  m.on('click', 'span, div', function (e) {
    var active = getActive();
    var target = getTarget();

    if (e.shiftKey) {
      if (active && active.nextAll().add(active).is(this)) {
        clearTarget();
        $(this).addClass('target');
      }
    } else {
      clearActive();
      clearTarget();
      $(this).addClass('active').addClass('target')
      active = $(this);
    }
    return false;
  })

  $(document).on('keypress', function (e) {
    if ($(e.target).closest('.modal').length) {
      return;
    }

    var active = getActive();
    var target = getTarget();

    if (active && !active.parents('.mote').is(m)) {
      return
    }

    if (e.keyCode == 13) {
      return;
    }

    var txt = String.fromCharCode(e.charCode);
    var span = $('<span>').text(txt).addClass('active').addClass('target');
    if (isBlock(active)) {
      clearActive();
      clearTarget();
      active.prepend(span);
      active = span;

      op([], addto(active,
        {
          "AddChars": txt
        }
      ));
    } else if (isChar(active)) {
      clearActive();
      clearTarget();
      span.insertAfter(active);
      active = span;

      op([], addto(active,
        {
          "AddChars": txt
        }
      ));
    }
  })

  var ophistory = []

  // Confirm that the operation is a valid one.
  function op (d, a) {
    console.log(JSON.stringify([d, a]));
    ophistory.push([d, a]);
    setTimeout(function () {
      var match = serialize();
      // test
      var packet = [
        ophistory,
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

  $(document).on('keydown', function (e) {
    if ($(e.target).closest('.modal').length) {
      return;
    }

    var active = getActive();
    var target = getTarget();

    if (active && !active.parents('.mote').is(m)) {
      return
    }

    console.log(e.keyCode);

    // enter
    // if (e.keyCode == 13) {
    //   if (active) {
    //     var tag = prm("Tag name:", "p");
    //     if (tag) {
    //       var out = active.nextAll().add(active);
    //       var parent = active.parent();
    //       var newparent = $('<div>').attr('data-tag', tag).insertAfter(parent);
    //       newparent.append(out);
    //       // TODO what in OP form
    //     }
    //   }
    //   return false;
    // }

    // command+,
    if (e.keyCode == 188 && e.metaKey) {
      // if (isBlock(active)) {
        bootbox.prompt({
          title: "Tag name:",
          value: "p",
          callback: function (tag) {
            if (tag) {
              var out = active.nextAll().add(active).not($('.target').nextAll());
              clearActive();
              clearTarget();
              out.wrapAll($('<div>').attr('data-tag', tag));
              active = out.parent().addClass('active').addClass('target');
              op([], addto(active,
                {
                  "AddGroup": [{"tag": tag}, [
                    {
                      "AddSkip": out.length
                    }
                  ]],
                }
              ));
            }
          },
        });
      // } else {
      //   alert('Not implemented?')
      // }
      return false;
    }

    // command+.
    if (e.keyCode == 190 && e.metaKey) {
      if (active) {
        bootbox.prompt({
          title: "Tag name:",
          value: "p",
          callback: function (tag) {
            if (tag) {
              op(delto(active,
                {
                  "DelGroup": [
                    {
                      "DelSkip": active.children().length
                    }
                  ],
                }
              ), addto(active,
                {
                  "AddGroup": [{"tag": tag}, [
                    {
                      "AddSkip": active.children().length
                    }
                  ]],
                }
              ));

              console.log('hi');
              active.attr('data-tag', tag);
              console.log('uh');

              //
              // var out = active.nextAll().add(active).not($('.target').nextAll());
              // clearActive();
              // clearTarget();
              // out.wrapAll($('<div>').attr('data-tag', tag));
              // active = out.parent().addClass('active').addClass('target');
              // op([], addto(active,
              //   {
              //     "variant": "AddGroup",
              //     "fields": [{"tag": tag}, [
              //       {
              //         "variant": "AddSkip",
              //         "fields": [out.length]
              //       }
              //     ]],
              //   }
              // ));
            }
          },
        });
      }
      return false;
    }
    if (e.keyCode == 8) {
      if (e.shiftKey) {
        // Delete group while saving contents.
        if (isBlock(active)) {
          clearActive();
          clearTarget();

          op(delto(active,
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
      } else if (e.metaKey) {
        // Delete whole block.
        if (isBlock(active)) {
          op(delto(active,
            {
              "DelGroupAll": null,
            }
          ), []);

          active.remove();
          active = null;
          clearActive();
          clearTarget();
        }
      } else {
        // Delete characters.
        if (isChar(active)) {
          clearActive();
          clearTarget();

          op(delto(active,
            {
              "DelChars": 1,
            }
          ), []);

          var prev = active.prev();
          var dad = active.parent();
          active.remove();
          active = prev[0] ? prev : dad[0] ? dad : null;
          active.addClass('active').addClass('target');
        }
      }

      return false;
    }

    // <enter>
    if (e.keyCode == 13) {
      if (e.shiftKey && isBlock(active)) {
        bootbox.prompt({
          title: "Tag name:",
          value: active.data('tag').toLowerCase(),
          callback: function (tag) {
            if (tag) {
              clearActive();
              clearTarget();

              $('<div data-tag="' + tag + '">').insertAfter(active).addClass('active').addClass('target');

              op([], addto(active.next(),
                {
                  "AddGroup": [{"tag": tag}, []],
                }
              ));
            }
          }
        });
      } else if (!e.shiftKey) {
        bootbox.prompt({
          title: "Tag name:",
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
              prev.wrapAll('<div data-tag="' + parent.data('tag') + '">');
              next.wrapAll('<div data-tag="' + tag + '" class="active target">');
              parent.contents().unwrap();

              op(operation[0], operation[1]);
            }
          },
        });
      }
    }
  })

  return ophistory;
}

var m1 = $('#mote-1');
var m2 = $('#mote-2');

var ops_a = init(m1);
var ops_b = init(m2);

$.get('/api/hello', function (data) {
  m1.empty().append(load(data));
  m2.empty().append(load(data));
})

$('#sync').on('click', function () {
  var packet = [
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
      alert('Error in syncing. Check the console.')
    }
    //
  })
  .fail(function () {
    console.log('failure', arguments);
    alert('Error in syncing. Check the console.')
  });
})
