var oatie = require('./')
  , tagr = require('tagr')
  , EventEmitter = require('events').EventEmitter;

var  __slice = [].slice,
  __hasProp = {}.hasOwnProperty,
  __extends = function (child, parent) { for (var key in parent) { if (__hasProp.call(parent, key)) child[key] = parent[key]; } function ctor() { this.constructor = child; } ctor.prototype = parent.prototype; child.prototype = new ctor(); child.__super__ = parent.prototype; return child; },
  __indexOf = [].indexOf || function (item) { for (var i = 0, l = this.length; i < l; i++) { if (i in this && this[i] === item) return i; } return -1; },
  __bind = function (fn, me){ return function (){ return fn.apply(me, arguments); }; };

function debugLog () {
  if (oatie.debug) {
    console.error.apply(console, arguments);
  }
}

/**
 * Transform
 *
 * Transform operatons A and B, yielding A', an operation on A that
 * that replicates the changes of client B as though they were made
 * before the changes on A were applied.
 */

// transformDeletions(delA, delB, schema)
//
// Transform two deletions delA and delB into delA' and delB'.

function transformDeletions (delA, delB, schema) {
  var iterA = new oatie.OpIterator(delA);
  var iterB = new oatie.OpIterator(delB);
  iterA.next();
  iterB.next();

  var oprA = oatie.record(), A = oatie._combiner(oprA, iterA, false, iterB, false);
  var oprB = oatie.record(), B = oatie._combiner(oprB, iterA, false, iterB, false);

  function unopenRecur (U, R, iterU, oprU, iterR, delR) {
    iterU.apply(delR).next();
    while (iterU.type != 'unclose') {
      if (iterU.isIn()) {
        unopenRecur(U, R, iterU, oprU, iterR, delR);
      } else {
        if (iterU.type == 'retain') {
          oprU[0].retain();
        }
        iterU.apply(delR).next();
      }
    }
    iterU.apply(delR).next();
  }

  function recur () {
    while (!(iterA.isExit() || iterB.isExit())) {
      console.log(iterA.type, iterB.type);
      if ((iterA.type === 'remove') && (iterB.type === 'remove')) {
        A.nextB();
        B.nextA();
      } else if ((iterA.type === 'remove') && (iterB.type === 'remove')) {
        A.nextB();
        B.nextA();
      } else if ((iterA.type === 'remove') && (iterB.type === 'enter' || iterB.type === 'unopen' || iterB.type === 'retain')) {
        A.nextB();
        B.useA();
      } else if ((iterA.type === 'enter' || iterA.type === 'unopen' || iterA.type === 'retain') && (iterB.type === 'remove')) {
        A.useB();
        B.nextA();
      } else if ((iterA.type === 'retain') && (iterB.type == 'unopen')) {
        unopenRecur(B, A, iterB, oprB, iterA, oprA[0]);
        B.nextA();
      } else if ((iterA.type === 'unopen') && (iterB.type === 'retain')) {
        unopenRecur(A, B, iterA, oprA, iterB, oprB[0]);
        A.nextB();
      } else if ((iterA.type === 'retain') && (iterB.type === 'enter' || iterB.type === 'retain')) {
        if (iterB.isIn()) {
          A.consumeB();
        } else {
          A.useB();
        }
        B.retainA();
      } else if ((iterA.type === 'enter' || iterA.type === 'retain') && (iterB.type === 'retain')) {
        A.retainB();
        if (iterA.isIn()) {
          B.consumeA();
        } else {
          B.useA();
        }
      } else if ((iterA.type === 'enter') && (iterB.type === 'enter')) {
        A.useB();
        B.useA();
        recur();
        A.useB();
        B.useA();
      } else if ((iterA.type === 'unopen') && (iterB.type === 'enter')) {
        A.nextB();
        B.useA();
        recur();
        A.nextB();
        B.useA();
      } else if ((iterA.type === 'enter') && (iterB.type === 'unopen')) {
        A.useB();
        B.nextA();
        recur();
        A.useB();
        B.nextA();
      } else if ((iterA.type === 'unopen') && (iterB.type === 'unopen')) {
        A.nextB();
        B.nextA();
        recur();
        A.nextB();
        B.nextA();
      } else {
        throw new Error('Encountered loop transforming del x del: ' + iterA.type + ' by ' + iterB.type);
      }
    }

    while (!iterA.isExit()) {
      if (iterA.isIn()) {
        B.consumeA();
      } else {
        B.useA();
      }
    }
    while (!iterB.isExit()) {
      if (iterB.isIn()) {
        A.consumeB();
      } else {
        A.useB();
      }
    }

    return [A.toJSON()[0], B.toJSON()[0]];
  }

  return recur();
}

// delAfterIns(insA, delB, schema)
//
// Transforms a insertion preceding a deletion into a deletion preceding an insertion.
// After this, sequential deletions and insertions can be composed together in one operation.

function delAfterIns (insA, delB, schema) {
  var c, delr, insr, iterA, iterB, _ref;
  _ref = oatie.record(), delr = _ref[0], insr = _ref[1];
  iterA = new oatie.OpIterator(insA);
  iterB = new oatie.OpIterator(delB);
  iterA.next();
  iterB.next();
  c = oatie._combiner([delr, insr], iterA, true, iterB, false);
  c.delAfterIns = function () {
    var _ref1, _ref2, _ref3, _ref4, _ref5, _ref6, _ref7, _ref8, _ref9;
    while (!(iterA.isExit() || iterB.isExit())) {
      if ((_ref1 = iterA.type) === 'text') {
        c.useA();
        delr.retain();
      }
      if ((_ref2 = iterA.type) === 'open') {
        delr.enter(iterA.tag, iterA.attrs);
        c.useA();
      }
      if (iterA.type === 'retain' && (iterB.type === 'remove' || iterB.type === 'retain')) {
        c.pickB();
      } else if (iterA.type === 'retain' && (iterB.type === 'unopen' || iterB.type === 'enter')) {
        c.nextA().consumeB();
      } else if (((_ref6 = iterA.type) === 'enter' || _ref6 === 'attrs') && (iterB.type === 'remove')) {
        c.retainA();
      } else if (((_ref6 = iterA.type) === 'enter' || _ref6 === 'attrs') && (iterB.type === 'retain')) {
        c.retainA().nextB();
      } else if (((_ref8 = iterA.type) === 'enter' || _ref8 === 'attrs') && ((_ref9 = iterB.type) === 'enter' || _ref9 === 'unopen')) {
        c.pickB().delAfterIns().pickB();
      }
    }

    while (!iterA.isExit()) {
      c.retainA();
    }
    // Catch .close() ending.
    if (iterA.type == 'close') {
      delr.leave();
      c.useA();
      return this.delAfterIns();
    }
    while (!iterB.isExit()) {
      if (iterB.type === 'retain') {
        c.useB();
      } else {
        c.consumeB();
      }
    }
    return this;
  };
  return c.delAfterIns().toJSON();
}

// transformInsertions(insA, insB, schema)
//
// Transform two deletions insA and insB into A' and B'. After transformation,
// insertions may need to delete content on the client that originated it; thus,
// we return two full operations A' and B' from this method.

function transformInsertions (insA, insB, schema) {
  var iterA = new oatie.OpIterator(insA)
    , iterB = new oatie.OpIterator(insB);
  
  iterA.next();
  iterB.next();

  var oprA = oatie.record(), delrA = oprA[0], insrA = oprA[1]
    , oprB = oatie.record(), delrB = oprB[0], insrB = oprB[1];


  // Transformer functions.
  var tran = {

    // tracks = [tagA, realTag, tagB, isOriginalayerA, isOriginatypeB];
    // TODO rename states
    tracks: [],

    // Return the deepest open track.
    current: function () {
      return tran.tracks[tran.tracks.length - 1];
    },
    currentA: function () {
      return tran.tracks.filter(function (track) {
        return track[0];
      }).reverse()[0];
    },
    currentB: function () {
      return tran.tracks.filter(function (track) {
        return track[2];
      }).reverse()[0];
    },

    // Return deepest open track for this schema layer.
    top: function (type) {
      return tran.tracks.filter(function (track) {
        return schema.findType(track[0] || track[2]) == type;
      }).reverse()[0];
    },
    topA: function (type) {
      return tran.tracks.filter(function (track) {
        return track[0] && schema.findType(track[0]) == type;
      }).reverse()[0];
    },
    topB: function (type) {
      return tran.tracks.filter(function (track) {
        return track[2] && schema.findType(track[2]) == type;
      }).reverse()[0];
    },

    // Indicate a new open track.
    push: function (a, r, b, aOrig, bOrig) {
      for (var i = 0; i < tran.tracks.length; i++) {
        if (!tran.tracks[i][1]) {
          break;
        }
      }
      tran.tracks.splice(i, 0, [a, r, b, !!aOrig, !!bOrig]);
    },

    // Return the lowest track that the client is NOT on.
    lowestA: function (tag) {
      // lowest top element that does not include B + those that include R.
      var type = schema.findType(tag);
      return tran.tracks.filter(function (track) {
        return !track[0] && track[2];
      }).filter(function (track) {
        return type.tags.indexOf(track[2]) > -1;
      }).reverse()[0];
      // return [].concat(
      //   tran.tracks.filter(function (track) {
      //     return track[1] && !track[0]
      //   }).reverse().slice(0, 1),
      //   tran.tracks.filter(function (track) {
      //     return !track[1] && !track[0];
      //   })
      // ).filter(function (track) {
      //   return type.tags.indexOf(track[2]) > -1;
      // })[0];
    },
    lowestB: function (tag) {
      var type = schema.findType(tag);
      return tran.tracks.filter(function (track) {
        return track[0] && !track[2];
      }).filter(function (track) {
        return type.tags.indexOf(track[0]) > -1;
      }).reverse()[0];
    },

    // Stop tracking an element depending on its originating client.
    pop: function () {
      return tran.tracks.pop();
    },
    popA: function (preserve) {
      for (var i = tran.tracks.length - 1; i >= 0; i--) {
        var track = tran.tracks[i];
        if (track[0]) {
          if (!track[2]) {
            tran.tracks.splice(i, 1);
            return;
          } else {
            track.splice(0, track.length, null, preserve ? track[1] : null, track[2], null, preserve ? track[4] : null);
            return;
          }
        }
      }
    },
    popB: function (preserve) {
      for (var i = tran.tracks.length - 1; i >= 0; i--) {
        var track = tran.tracks[i];
        if (track[2]) {
          if (!track[0]) {
            tran.tracks.splice(i, 1);
            return;
          } else {
            track.splice(0, track.length, track[0], preserve ? track[1] : null, null, preserve ? track[3] : null, null);
            return;
          }
        }
      }
    },

    // Consider a client's open atom as part of an already opened element.
    //TODO needs to take a type, not a tag, to pass to lowestA.
    trackA: function (a, isOrig) {
      var track = tran.lowestA(a);
      track[0] = a;
      track[3] = !!isOrig;
    },
    trackB: function (b, isOrig) {
      var track = tran.lowestB(b);
      track[2] = b;
      track[4] = !!isOrig;
    },

    // Use an insertion or an enter and start a new output element.
    useA: function () {
      var a = iterA.tag;
      iterA.apply(insrA);
      iterA.apply(insrB);
      delrA.enter();
      iterA.next();
      tran.push(a, a, null, true, false);
    },
    useB: function () {
      var b = iterB.tag;
      iterB.apply(insrA);
      iterB.apply(insrB);
      delrB.enter();
      iterB.next();
      tran.push(null, b, b, false, true);
    },
    use: function () {
      var a = iterA.tag;
      iterA.apply(insrA);
      iterA.apply(insrB);
      delrA.enter();
      delrB.enter();
      iterA.next();
      iterB.next();
      tran.push(a, a, a, true, true);
    },

    // Pick client A over client B.
    pickA: function () {
      tran.useA();
      tran.silenceB();
    },

    // Replace both open clients with a combined element.
    substitute: function (r) {
      tran.push(null, r, null);
      insrA.open(r, {});
      insrB.open(r, {});
      tran.silenceA();
      tran.silenceB();
    },

    // Remove an element that was opened from the originating client.
    // The element is considered having been merged with the lowest unbound state for this client.
    silenceA: function () {
      debugLog('  silenceA:', iterA.tag, tran.lowestA(iterA.tag));
      delrA.unopen(iterA.tag);
      tran.trackA(iterA.tag);
      iterA.next();
    },
    silenceB: function () {
      debugLog('  silenceB:', iterB.tag, tran.lowestB(iterB.tag));
      delrB.unopen(iterB.tag);
      tran.trackB(iterB.tag);
      iterB.next();
    },

    // Close a client intelligently.
    closeA: function (like, unlike) {
      var layer = schema.findType(tran.currentA()[0]);
      var like = layer.like, unlike = layer.unlike;
      var strategy = a == b ? like : unlike;

      var track = tran.currentA(), a = track[0], r = track[1], b = track[2], origA = track[3], origB = track[4];

      debugLog('  closeA():', a, r, b, like, unlike, origA, origB);

      tran.popA(like != 'split');

      // If client B is not open, or client B is open but we split along element bounds, close.
      if ((!b && r) || (b && like == 'split')) {
        if (origA) {
          // Preserve unmodified insertions.
          insrA.alter('', null);
        }
        insrA.leave();
      }

      // If the other client is still open, we must switch to a close statement.
      if ((!b && r) || (b && like == 'split')) {
        // Should this match the other one...? alter as (b, {}) instead?
        insrB.alter(r, {}).close();
      }

      if (!origA || (b && like == 'combine')) {
        delrA.alter(a, {});
      }
      delrA.leave();
      
      iterA.next();


      // if (tran.top() && !tran.top()[0] && !tran.top()[2] && tran.top()[1]) {
      //   insrA.close(); insrB.close(); tran.pop();
      // }
    },
    closeB: function (layer, demote) {
      var layer = schema.findType(tran.currentB()[2]);
      var like = layer.like, unlike = layer.unlike;
      var strategy = a == b ? like : unlike;

      var track = tran.currentB(), a = track[0], r = track[1], b = track[2], origA = track[3], origB = track[4];

      debugLog('  closeB():', a, r, b, like, unlike, origA, origB, demote);

      tran.popB(like != 'split' || demote);

      // If client A is not open, or client A is open but we split along element bounds, close.
      if (((!a && r) || (a && like == 'split')) && !demote) {
        if (origB) {
          // Preserve unmodified insertions.
          insrB.alter('', null);
        }
        insrB.leave();
      }

      // If the other client is still open, we must switch to a close statement.
      if ((a && like == 'split') && !demote) {
        insrA.alter(a, {}).close();
      } else if ((!a && r) && !demote) {
        insrA.alter(r, {}).close();
      }

      if (!origB || (a && like == 'combine') || demote) {
        delrB.alter(b, {});
      }
      delrB.leave();
      
      iterB.next();

      // if (demote) {
      //   tran.push(null, r, null);
      // }
      // if (tran.top() && !tran.top()[0] && !tran.top()[2] && tran.top()[1]) {
      //   insrA.close(); insrB.close(); tran.pop();
      // }
    },
    close: function () {
      var track = tran.pop(), a = track[0], r = track[1], b = track[2], origA = track[3], origB = track[4];

      debugLog('  close():', a, b, null, origA, origB);
      
      if (origA) {
        insrA.alter('', null)
      }
      insrA.leave();
      if (origB) {
        insrB.alter('', null)
      }
      insrB.leave();

      if (origA) {
        delrA.leave();
      } else {
        delrA.alter(a, {}).leave();
      }
      if (origB) {
        delrB.leave();
      } else {
        delrB.alter(b, {}).leave();
      }
      
      iterA.next();
      iterB.next();
    },

    // Combines client A into open element by creating a new element.
    combineA: function (r) {
      var track = tran.abort(), r = track[1], b = track[2];

      debugLog('  combineA:', null, r, b);

      tran.push(null, r, null);
      insrA.open(r, {});
      insrB.open(r, {});
      tran.silenceA();
      tran.trackB(b);
    },
    combineB: function (r) {
      var track = tran.abort(), a = track[0], r = track[1];

      debugLog('  combineB:', a, r, null);

      tran.push(null, r, null);
      insrA.open(r, {});
      insrB.open(r, {});
      tran.silenceB();
      tran.trackA(a);
    },

    // Abort open element, replace with A.
    switchA: function (a) {
      var track = tran.abort(), b = track[2];
      tran.useA(a);
      tran.trackB(b);
    },

    // Close the topmost element.
    abort: function () {
      var track = tran.pop(), a = track[0], r = track[1], b = track[2];
      if (r) {
        debugLog('ABORTING', a, r, b);
        if (a) {
          insrA.alter(r, {}).close();
        } else {
          insrA.close();
        }
        if (b) {
          insrB.alter(r, {}).close();
        } else {
          insrB.close();
        }
      }
      return [a, r, b];
    },

    // Interrupt all children of a type.
    interrupt: function (type) {
      var regen = [];
      while (tran.current() && tran.current()[1] && schema.findType(tran.current()[1]) != type && schema.getAncestors(type).indexOf(schema.findType(tran.current()[1])) == -1) {
        regen.push(tran.abort());
      }

      regen.forEach(function (r) {
        tran.push(r[0], null, r[2], false, false);
      });
    },

    // Regenerate elements which have been closed on output
    // but are still open on one or both clients.
    regenerate: function (type) {
      tran.tracks.filter(function (track) {
        // Filter for types that are ancestors of the current type.
        var tracktype = schema.findType(track[0] || track[2]);
        return !type || (schema.getAncestors(type).indexOf(tracktype) > -1 && tracktype != type);
      }).forEach(function (track) {
        if (!track[1]) {
          debugLog('REGENERATE', track);
          if (track[0] || track[2]) {
            var a = track[0], b = track[2], origA = track[3], origB = track[4];
            track.splice(0, 5, a, a || b, b, origA, origB);

            if (origA) {
              insrA.enter();
            } else {
              insrA.open(a || b, {});
            }
            
            if (origB) {
              insrB.enter();
            } else {
              insrB.open(a || b, {});
            }
          } else {
            throw new Error('Inconsistent track state :(');
          }
        }
      });
    },
  };

  // Main loop.

  while (!(iterA.type === 'end' && iterB.type === 'end')) {
    debugLog(['>>>', iterA.type, iterB.type].join(' ').green);

    // Closing tags.

    if (iterA.type === 'close' || iterB.type === 'close') {
      while (iterA.type === 'close' || iterB.type === 'close') {
        // Identify the transformation layers for these elements.
        var typeA = iterA.type === 'close' && schema.findType(tran.currentA()[0]);
        var typeB = iterB.type === 'close' && schema.findType(tran.currentB()[2]);
        
        debugLog('>>> closing:', iterA.type === 'close' ? typeA && tran.topA(typeA) || '???' : '...', iterB.type === 'close' ? typeB && tran.topB(typeB) || '???' : '...');

        if (!(typeA || typeB)) {
          throw new Error('Could not transform close atoms, none exist.');
        }

        // Greedy matching,
        if (iterA.type == 'open' && schema.getAncestors(schema.findType(iterA.tag)).indexOf(typeB) > -1
        && !(tran.top(schema.findType(iterA.tag)) && !tran.top(schema.findType(iterA.tag))[0])) {
          debugLog('  Greed matching B');
          tran.closeB(null, true);
        }
        // if (iterB.type == 'open' && schema.getAncestors(schema.findType(iterB.tag)).indexOf(typeA) > -1) {
        // }

        // ...close both elements at once,
        else if (typeA && typeB && tran.currentA() === tran.currentB()) {
          tran.interrupt(typeA || typeB);
          tran.close();
        }

        // ...close client A,
        else if (typeA && (!typeB || (typeA != typeB && schema.getAncestors(typeB).indexOf(typeA) == -1))) {
          tran.interrupt(typeA);
          tran.closeA();
        }

        // ...or close client B.
        else if (typeB) {
          tran.interrupt(typeB);
          tran.closeB();
        }

        debugLog('  States:', tran.tracks);
        debugLog('  Op for A:', insrA.stack);
        debugLog('  Op for B:', insrB.stack);
      }
    }

    // Opening tags.

    else if (iterA.type === 'open' || iterB.type === 'open') {
      while (iterA.type === 'open' || iterB.type === 'open') {
        var typeA = iterA.type === 'open' && schema.findType(iterA.tag);
        var typeB = iterB.type === 'open' && schema.findType(iterB.tag);

        debugLog('>>> opening:', iterA.type === 'open' ? iterA.tag : '..', iterB.type === 'open' ? iterB.tag : '...');

        if (!(typeA || typeB)) {
          throw new Error('Tag does not exist in transformation schema.');
        }

        // Silence client A elements that match existing output elements,
        if (typeA && typeA.like == 'stack' && tran.top(typeA) && tran.top(typeA) != tran.topA(typeA) && tran.top(typeA)[1] == iterA.tag) {
          tran.regenerate(typeA);
          tran.silenceA();
        }
        
        // ...silence client B elements that match existing output elements (ignoring priority),
        else if (typeB && typeB.like == 'stack' && tran.top(typeB) && tran.top(typeB) != tran.topB(typeB) && tran.top(typeB)[1] == iterB.tag) {
          tran.regenerate(typeB);
          tran.silenceB();
        }

        // ...simultaneous open,
        else if (typeA && typeB && typeA === typeB) {
          tran.regenerate(typeA);
          if (typeA.void) {
            if (iterB.tag === iterA.tag) {
              iterA.next();
              iterB.next();
              insrA.retain();
              insrB.retain();
              delrA.retain();
              delrB.retain();
            } else {
              iterA.apply(insrB).next().apply(insrB).next();
              insrB.retain();
              iterB.apply(insrA).next().apply(insrA).next();
              insrA.retain();
              delrA.retain();
              delrB.retain();
            }
          } else {
            tran.interrupt(typeA);
            if (iterB.tag === iterA.tag) {
              if (typeA.like === 'split') {
                tran.use();
              } else if (typeA.like === 'combine') {
                tran.use();
              }
            } else {
              if (typeA.unlike === 'priority') {
                tran.pickA();
              } else if (typeA.unlike === 'combine') {
                tran.substitute(typeA.combine({
                  tag: iterA.tag
                }, {
                  tag: iterB.tag
                }));
              }
            }
          }
        }

        // ...open A,
        else if (typeA && (!typeB || (typeA == typeB || schema.getAncestors(typeB).indexOf(typeA) > -1))) {
          tran.regenerate(typeA);

          // Void elements are transformed immediately.
          if (typeA.void) {
            iterA.apply(insrB).next().apply(insrB).next();
            insrA.retain();
            delrA.retain();
          } else {
            // Interrupt child layers from either client.
            tran.interrupt(typeA);
            // If there is nothing to transform with or we're just stacking elements, open away.
            if (!tran.top(typeA) || (tran.top(typeA) == tran.topA(typeA))) {
              tran.useA();
            } else {
              // Check if there is an existing element with this tag.
              if (tran.lowestA(iterA.tag)) {
                if (typeA.like === 'split') {
                  tran.switchA();
                } else if (typeA.like === 'combine') {
                  tran.silenceA();
                }
              } else {
                if (typeA.unlike === 'priority') {
                  tran.useA();
                } else if (typeA.unlike === 'combine') {
                  // TODO actually use a merge feature for this tag.
                  tran.combineA('combined');
                }
              }
            }
          }
        }

        // ...or open B.
        else if (typeB) {
          tran.regenerate(typeB);

          // Void elements are transformed immediately.
          if (typeB.void) {
            iterB.apply(insrA).next().apply(insrA).next();
            insrB.retain();
            delrB.retain();
          } else {
            // Interrupt child layers from either client.
            tran.interrupt(typeB);
            // If there is nothing to transform with or we're just stacking elements, open away.
            if (!tran.top(typeB) || (tran.top(typeB) == tran.topB(typeB))) {
              tran.useB();
            } else {
              // Check if there is an existing element with this tag.
              if (tran.lowestB(iterB.tag)) {
                if (typeB.like === 'split') {
                  tran.silenceB();
                } else if (typeB.like === 'combine') {
                  tran.silenceB();
                }
              } else {
                if (typeB.unlike === 'priority') {
                  tran.silenceB();
                } else if (typeB.unlike === 'combine') {
                  tran.combineB('combine');
                }
              }
            }
          }
        }

        debugLog('  States:', tran.tracks);
        debugLog('  Op for A:', insrA.stack);
        debugLog('  Op for B:', insrB.stack);
      }
    }

    // Remainder.

    else {
      // Regenerate layers.
      tran.regenerate();

      function consume (iter, insr) {
        iter.apply(insr).next();
        while (!iter.isExit()) {
          if (iter.isIn()) {
            consume(iter, insr);
          } else {
            iter.apply(insr).next();
          }
        }
        iter.apply(insr).next();
      }

      if (iterA.type === 'text') {
        iterA.apply(insrB).next();
        insrA.retain();
        delrA.retain();
      } else if (iterB.type === 'text') {
        iterB.apply(insrA).next();
        insrB.retain();
        delrB.retain();
      } else if ((iterA.type === 'enter' || iterA.type === 'leave') && iterA.type === iterB.type) {
        iterA.apply(insrA).apply(insrB).apply(delrA).apply(delrB).next();
        iterB.next();
      } else if (iterA.type === 'enter' && iterB.type === 'retain') {
        consume(iterA, insrB);
        iterB.apply(delrB).apply(insrA).apply(delrA).next();
      } else if (iterA.type === 'retain' && iterB.type === 'enter') {
        consume(iterB, insrA);
        iterA.apply(delrA).apply(insrB).apply(delrB).next();
      } else if (iterA.type === 'retain' || iterB.type === 'retain') {
        insrA.retain();
        insrB.retain();
        delrA.retain();
        delrB.retain();
        if (iterA.type === 'retain') {
          iterA.next();
        }
        if (iterB.type === 'retain') {
          iterB.next();
        }
      } else {
        if (iterA.type !== 'end' && iterB.type !== 'end' && iterA.type == iterB.type) {
          iterA.apply(insrA).apply(insrB).apply(delrA).apply(delrB).next();
          iterB.next();
        } else if (iterA.type == 'enter') {
          consume(iterA, insrB);
          insrA.retain();
        } else if (iterB.type == 'enter') {
          consume(iterB, insrA);
          insrB.retain();
        } else {
          throw new Error('Cannot transform ' + iterA.type + ' x ' + iterB.type);
        }
      }
    }
  }
  return [oatie.op(delrA, insrA), oatie.op(delrB, insrB)];
};

// oatie.transform(opA, opB, schema)
//
// Transform two operations according to a schema.

oatie.transform = function (opA, opB, schema) {
  var delA = opA[0], insA = opA[1];
  var delB = opB[0], insB = opB[1];

  // Transform deletions A and B against each other to get delA` and delB`.
debugLog();
debugLog('--- transforming: deletions'.yellow);
debugLog('delA\t', JSON.stringify(delA));
debugLog('delB\t', JSON.stringify(delB));
  var _ = transformDeletions(delA, delB, schema)
    , delA_0 = _[0], delB_0 = _[1];
debugLog('delA`0\t', JSON.stringify(delA_0));
debugLog('delB`0\t', JSON.stringify(delB_0));

  // The result will be applied after the client's insert operations had already been performed.
  // Reverse the impact of insA with delA` to not affect already newly added elements or text.
debugLog('--- transforming: deletions after insertions (A)'.yellow);
debugLog('insA\t', JSON.stringify(insA));
debugLog('delA`0\t', JSON.stringify(delA_0));
  var _ = delAfterIns(insA, delA_0, schema)
    , delA_1 = _[0];
debugLog('delA`1\t', JSON.stringify(delA_1));

debugLog('--- transforming: deletions after insertions (B)'.yellow);
debugLog('insB\t', JSON.stringify(insB));
debugLog('delB_0\t', JSON.stringify(delB_0));
  var _ = delAfterIns(insB, delB_0)
    , delB_1 = _[0];
debugLog('delB`1\t', JSON.stringify(delB_1));

debugLog('--- composing: reducing insertions by deletions'.yellow);
  // Insertions from both clients must be composed as though they happened against delA` and delB`
  // so that we don't have phantom elements.
  var _ = oatie._composer(insA, true, delA_1, false).compose().toJSON()
    , insA1 = _[1];
  var _ = oatie._composer(insB, true, delB_1, false).compose().toJSON()
    , insB1 = _[1];

  // Transform insert operations together.
debugLog('insA\t', JSON.stringify(insA), '\ninsB\t', JSON.stringify(insB));
debugLog('delA_1\t', JSON.stringify(delA_1), '\ndelB_1\t', JSON.stringify(delB_1));
debugLog('insA1\t', JSON.stringify(insA1), '\ninsB1\t', JSON.stringify(insB1));

debugLog('--- insertions'.yellow);
  var _ = transformInsertions(insA1, insB1, schema),
    delA_2 = _[0][0], insA_1 = _[0][1],
    delB_2 = _[1][0], insB_1 = _[1][1];
debugLog('delA_2\t', JSON.stringify(delA_2), '\ndelB_2\t', JSON.stringify(delB_2));
debugLog('insA_1\t', JSON.stringify(insA_1), '\ninsB_1\t', JSON.stringify(insB_1));

debugLog('--- compose deletions'.yellow);
  // Our delete operations are now subsequent operations, and so can be composed.
  var _ = oatie._composer(delA_1, false, delA_2, false).compose().toJSON()
    , delA_3 = _[0], _ = _[1];
  var _ = oatie._composer(delB_1, false, delB_2, false).compose().toJSON()
    , delB_3 = _[0], _ = _[1];
debugLog('delA_3\t', JSON.stringify(delA_3), '\ndelA_3', JSON.stringify(delA_3));

debugLog('--- transform result'.yellow);
debugLog('delA_3\t', JSON.stringify(delA_3), '\ninsA_1\t', JSON.stringify(insA_1));
debugLog('delB_3\t', JSON.stringify(delB_3), '\ninsB_1\t', JSON.stringify(insB_1));
debugLog('');

  // Return operations A` and B`.
  return [[delA_3, insA_1], [delB_3, insB_1]];
};