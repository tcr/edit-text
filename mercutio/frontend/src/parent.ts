export default class Parent {
  cache: any;
  version: number;
  alive: number;
  syncSocket: WebSocket;
  docState: any | null;
  syncWait: boolean;

  constructor() {
    this.cache = {};
    // TODO get this from the initial load
    this.version = 101;
    this.alive = 0;
    this.syncWait = false;

    // Timer component.
    let counter = 0;
    setInterval(() => {
      requestAnimationFrame(() => {
        $('#timer').each(function () {
          $(this).text(counter++ + 's');
        })
      });
    }, 1000);

    // Monkey global click button.
    $('#action-monkey').on('click', () => {
      for (let i = 0; i < window.frames.length; i++) {
        window.frames[i].postMessage({
          'Monkey': {}
        }, '*');
      }
    })

    let parent = this;
    this.syncSocket = new WebSocket('ws://127.0.0.1:3010');
    this.syncSocket.onopen = function (event) {
      console.log('(!) SyncSocket connected.');
    };
    this.syncSocket.onmessage = this.onSyncClientMessage.bind(this);
    this.syncSocket.onclose = function () {
      $('body').css('background', 'red');
      alert('Websocket closed, error?');

      // TODO just in case?
      window.stop();
    }
  }

  onSyncClientMessage(msg) {
    let data = JSON.parse(msg.data);

    if ('Update' in data) {
      console.log('updating', data);
      this.docState = data.Update;
      this.syncChildren(data.Update);
      this.syncWait = false;
    }
  }

  initialize() {
    // let parent = this;
    // $.get('/api/hello', data => {
    //   parent.syncChildren(data);
    // });
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
        if (parent.docState !== null && parent.docState && parent.alive == 2) {
          parent.syncChildren(parent.docState);
        }
      }
    };
  }

  sync() {
    let parent = this;

    if (this.syncWait) {
      return;
    }

    if (this.alive != 2) {
      return;
    }

    if ((!this.cache.left || this.cache.left.version != this.version) ||
      (!this.cache.right || this.cache.right.version != this.version)) {
      console.log('outdated, skipping:', this.cache.left, this.cache.right);
      return;
    }

    // Drop no-ops
    if (!this.cache.left.ops.length && !this.cache.right.ops.length) {
      return;
    }

    this.syncWait = true;
    this.version += 1;

    let packet = [this.cache.left.ops, this.cache.right.ops];

    this.syncSocket.send(JSON.stringify({
      Sync: packet,
    }));
  }

  syncChildren(data) {
    for (let i = 0; i < window.frames.length; i++) {
      window.frames[i].postMessage({
        'Sync': data
      }, '*');
    }
  }
}