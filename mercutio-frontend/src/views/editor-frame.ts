import Clipboard from 'clipboard';

import * as commands from '../commands';
import HashState from '../hashstate';
import * as util from '../util';
import * as interop from '../interop';
import {editorSetup} from '../editor';
import {Network, ProxyNetwork, WasmNetwork} from '../network';

// Initialize child editor.
export class EditorFrame {
  $elem: any;
  editorID: string;
  ops: Array<any>;
  KEY_WHITELIST: any;
  markdown: string;

  network: Network;

  constructor(elem: HTMLElement, network: Network) {
    this.$elem = $(elem);
    this.editorID = '$$$$$$'; // TODO should this autopopulate
    this.ops = [];
    this.KEY_WHITELIST = [];
    this.markdown = '';

    this.network = network;
    this.network.onNativeMessage = this.onNativeMessage.bind(this);

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
        try {
          editor.network.nativeCommand(commands.RequestMarkdown());
        } catch (e) {
          // Socket may not be ready yet
        }
      }, 2000);
      setTimeout(() => {
        // Early request
        try {
          editor.network.nativeCommand(commands.RequestMarkdown());
        } catch (e) {
          // Socket may not be ready yet
        }
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
      });

    // Client Id.
    $('<b>Client: <kbd>' + this.editorID + '</kbd></b>')
      .appendTo($('#local-buttons'));

    editorSetup(this.$elem[0], this.network, this.KEY_WHITELIST);
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

  // Received message on native socket
  onNativeMessage(parse: any) {
    const editor = this;

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
      editor.KEY_WHITELIST.splice.apply(editor.KEY_WHITELIST,
        [0, 0].concat(parse.Setup.keys.map(x => ({
          keyCode: x[0],
          metaKey: x[1],
          shiftKey: x[2],
        })))
      );
  
      $('#native-buttons').each((_, x) => {
        parse.Setup.buttons.forEach(btn => {
          $('<button>').text(btn[1]).appendTo(x).click(_ => {
            editor.network.nativeCommand(commands.ButtonCommand(btn[0]));
          });
        })
      });
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
      let msg = event.data;

      if ('Monkey' in msg) {
        // TODO reflect this in the app
        editor.network.nativeCommand(commands.MonkeyCommand(msg.Monkey));
      }
    };
  }
}

export function start(network: Network) {
  // Create the editor frame.
  let editor = new EditorFrame(document.getElementById('mote'), network);
  // Connect to parent window (if exists).
  editor.multiConnect();

  // Background colors.
  network.onNativeClose = function () {
    $('body').css('background', 'red');
  };
  network.onSyncClose = function () {
    $('body').css('background', 'red');
  };

  // Connect to remote sockets.
  network.nativeConnect()
  .then(() => network.syncConnect())
  .then(() => {
    console.log('edit-text initialized.');
  });
};
