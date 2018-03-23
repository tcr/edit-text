import Clipboard from 'clipboard';

import * as commands from '../commands';
import * as util from '../util';
import * as interop from '../interop';
import * as React from 'react';
import * as ReactDOM from 'react-dom';
import { Editor } from '../editor';
import { Network, ProxyNetwork, WasmNetwork } from '../network';

declare var CONFIG: any;

const ROOT_QUERY = '.edit-text';

function NativeButtons(props) {
  return props.buttons.map(btn =>
    <button
      onClick={
        () => props.editor.network.nativeCommand(commands.ButtonCommand(btn[0]))
      }
      className={btn[2] ? 'active' : ''}
    >{btn[1]}</button>
  );
}

class LocalButtons extends React.Component {
  props: {
    editor: EditorFrame,
  };

  state = {
    width: 'auto',
    copying: false,
  };

  onSaveMarkdown() {
    this.setState({
      ...this.state,
      copying: true,
    });
    setTimeout((() => {
      this.setState({
        ...this.state,
        copying: false,
      });
    }).bind(this), 2000);
  }

  onXray() {
    this.props.editor.$elem.toggleClass('theme-mock');
    this.props.editor.$elem.toggleClass('theme-block');
  }

  render(): React.ReactNode {
    return (
      <div>
        <button
          id="save-markdown"
          onClick={() => this.onSaveMarkdown()}
          style={{ width: this.state.copying ? this.state.width : 'auto', }}
          ref={(el) => { el && (this.state.width = el.offsetWidth + 'px'); }}
        >
          {this.state.copying ? `Copied!` : `Save Markdown`}
        </button>

        <button id="xray" onClick={() => this.onXray()}>X-Ray</button>

        <b>Client: <kbd>{this.props.editor.editorID}</kbd></b>
      </div>
    );
  }
}

// Initialize child editor.
export class EditorFrame {
  $elem: any;
  editorID: string;
  ops: Array<any>;
  KEY_WHITELIST: any;
  markdown: string;

  network: Network;

  constructor(
    elem: Element,
    network: Network,
    body: string,
  ) {
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
        text: function (trigger) {
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

    ReactDOM.render(
      <LocalButtons editor={editor} />,
      document.querySelector("#local-buttons"),
    );

    this.$elem[0].innerHTML = '';

    ReactDOM.render(
      <Editor 
        network={this.network} 
        KEY_WHITELIST={this.KEY_WHITELIST}
        content={body}
      />,
      this.$elem[0],
    );
  }

  setID(id: string) {
    this.editorID = id;

    // Update the client identifier
    $('kbd').text(id);
  }

  load(data: string) {
    // let elem = this.$elem[0];
    // requestAnimationFrame(() => {
    //   elem.innerHTML = data;

    //   // Highlight our caret.
    //   document.querySelectorAll(
    //     `div[data-tag="caret"][data-client=${JSON.stringify(this.editorID)}]`,
    //   ).forEach(caret => {
    //     caret.classList.add("current");
    //   });
    // });
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
      } else {
        editor.ops.push(parse.Update[1]);
      }
    }

    else if (parse.MarkdownUpdate) {
      editor.markdown = parse.MarkdownUpdate;
    }

    else if (parse.Controls) {
      console.log('SETUP CONTROLS', parse.Controls);

      // Update the key list in-place.
      editor.KEY_WHITELIST.splice.apply(editor.KEY_WHITELIST,
        [0, 0].concat(parse.Controls.keys.map(x => ({
          keyCode: x[0],
          metaKey: x[1],
          shiftKey: x[2],
        })))
      );

      ReactDOM.render(
        <NativeButtons buttons={parse.Controls.buttons} editor={editor} />,
        document.querySelector("#native-buttons"),
      );
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

export function start() {
  let network = CONFIG.wasm ? new WasmNetwork() : new ProxyNetwork();

  // Utility classes for Multi
  if (window.parent != window) {
    // Blur/Focus classes.
    $(window).on('focus', () => $(document.body).removeClass('blurred'));
    $(window).on('blur', () => $(document.body).addClass('blurred'));
    $(document.body).addClass('blurred');
  }

  // Create the editor frame.
  let editor = new EditorFrame(
    document.querySelector(ROOT_QUERY)!,
    network,
    document.querySelector('.edit-text')!.innerHTML,
  );
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
