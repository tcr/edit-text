export default class Parent {
  cache: any;
  version: number;
  alive: number;

  constructor() {
    this.cache = {};
    // TODO get this from the initial load
    this.version = 101;
    this.alive = 0;

    // Timer component.
    let counter = 0;
    setInterval(() => {
      $('#timer').each(function () {
        $(this).text(counter++ + 's');
      })
    }, 1000);

    // Monkey global click button.
    $('#action-monkey').on('click', () => {
      for (let i = 0; i < window.frames.length; i++) {
        window.frames[i].postMessage({
          'Monkey': {}
        }, '*');
      }
    })

    // Reset button
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
          window.stop();
        }
        //
      })
    });
  }

  initialize() {
    let parent = this;
    $.get('/api/hello', data => {
      parent.syncChildren(data);
    });
  }

  childConnect() {
    let parent = this;
    window.onmessage = function (event) {
      if ('Update' in event.data) {
        let name = event.data.Update.name;
        parent.cache[name] = event.data.Update;
      }
      if ('Live' in event.data) {
        parent.alive += 1;
        if (parent.alive == 2) {
          parent.initialize();
        }
      }
    };
  }

  sync() {
    let parent = this;

    if (this.alive != 2) {
      return;
    }

    if ((!this.cache.left || this.cache.left.version != this.version) ||
      (!this.cache.right || this.cache.right.version != this.version)) {
      console.log('outdated, skipping:', this.cache.left, this.cache.right);
      return;
    }
    this.version += 1;

    let packet = [this.cache.left.ops, this.cache.right.ops];

    console.log('PACKET', packet)

    $.ajax('/api/sync', {
      data : JSON.stringify(packet),
      contentType : 'application/json',
      type : 'POST',
    })
    .done(function (data, _2, obj) {
      console.log('success', arguments);
      if (obj.status == 200 && data != '') {
        // Get the new document state and update the two clients
        parent.syncChildren(data.doc);
      } else {
        alert('Error in syncing. Check the command line.')
        window.stop();
      }
      //
    })
    .fail(function () {
      console.log('failure', arguments);
      alert('HTTP error in syncing. Check the command line.')
      window.stop();
    });
  }

  syncChildren(data) {
    for (let i = 0; i < window.frames.length; i++) {
      window.frames[i].postMessage({
        'Sync': data
      }, '*');
    }
  }
}