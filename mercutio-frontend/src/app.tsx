// Global CSS
import '../styles/mercutio.scss';

import * as Clipboard from 'clipboard';
import * as commands from './commands';
import * as util from './util';
import * as React from 'react';
import * as ReactDOM from 'react-dom';
import { Editor } from './editor';
import { Network, ProxyNetwork, WasmNetwork } from './network';

declare var CONFIG: any;

// Check page configuration.
if (!CONFIG.configured) {
  alert('The window.CONFIG variable was not configured by the server!')
}

const ROOT_QUERY = '.edit-text';

function NativeButtons(
  props
) {
  return (
    <div id="native-buttons">{
      props.buttons.map((btn, i) =>
        <button
          key={i}
          onClick={
            () => props.editor.network.nativeCommand(commands.Button(btn[0]))
          }
          className={btn[2] ? 'active' : ''}
        >{btn[1]}</button>
      )
    }</div>
  );
}

class LocalButtons extends React.Component {
  props: {
    editorID: string,
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
    // TODO
    // this.props.editor.$elem.toggleClass('theme-mock');
    // this.props.editor.$elem.toggleClass('theme-block');
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

        <b>Client: <kbd>{this.props.editorID}</kbd></b>
      </div>
    );
  }
}

function pollMarkdown(network: Network) {
  // Request markdown source.
  setInterval(() => {
    try {
      network.nativeCommand(commands.RequestMarkdown());
    } catch (e) {
      // Socket may not be ready yet
    }
  }, 2000);
  setTimeout(() => {
    // Early request
    try {
      network.nativeCommand(commands.RequestMarkdown());
    } catch (e) {
      // Socket may not be ready yet
    }
  }, 500);
}

// Initialize child editor.
export class EditorFrame extends React.Component {
  props: {
    network: Network,
    body: string,
  };

  state: {
    body: string,
    buttons: any,
    editorID: string,
  };

  KEY_WHITELIST: any;
  markdown: string;
  network: Network;

  constructor(
    props,
  ) {
    super(props);

    this.KEY_WHITELIST = [];
    this.markdown = '';

    this.network = props.network;
    this.network.onNativeMessage = this.onNativeMessage.bind(this);

    // Background colors.
    // TODO make these actionable on this object right?
    this.network.onNativeClose = function () {
      document.body.style.background = 'red';
    };
    this.network.onSyncClose = function () {
      document.body.style.background = 'red';
    };

    {
      new Clipboard('#save-markdown', {
        text: (trigger) => {
          return this.markdown;
        }
      });

      pollMarkdown(this.network);
    }

    this.state = {
      body: this.props.body,
      buttons: [],
      editorID: '$$$$$$',
    };
  }

  render(): React.ReactNode {
    return (
      <div>
        <div id="toolbar">
          <a href="https://github.com/tcr/edit-text" id="logo">edit-text</a>
          <NativeButtons 
            buttons={this.state.buttons}
            editor={this} 
          />,
          <LocalButtons
            editorID={this.state.editorID}
          />
        </div>

        <Editor 
          network={this.props.network} 
          KEY_WHITELIST={this.KEY_WHITELIST}
          content={this.state.body}
          editorID={this.state.editorID}
        />,
      </div>
    );
  }

  // Received message on native socket
  onNativeMessage(parse: any) {
    const editor = this;

    if (parse.Init) {
      this.setState({
        editorID: parse.Init,
      })
    }

    else if (parse.Update) {
      // Update page content
      this.setState({
        body: parse.Update[0],
      });
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

      // Update buttons view
      this.setState({
        buttons: parse.Controls.buttons,
      });
    }

    else {
      console.error('Unknown packet:', parse);
    }
  }
}


function multiConnect(network: Network) {
  // Blur/Focus classes.
  window.addEventListener('focus', () => {
    document.body.classList.remove('blurred');
  });
  window.addEventListener('blue', () => {
    document.body.classList.add('blurred');
  });
  document.body.classList.add('blurred');

  // Forward Monkey events.
  window.onmessage = (event) => {
    let editor = this;

    // Sanity check.
    if (typeof event.data != 'object') {
      return;
    }
    let msg = event.data;

    if ('Monkey' in msg) {
      // TODO reflect this in the app
      network.nativeCommand(commands.Monkey(msg.Monkey));
    }
  };
}

export function start() {
  let network = CONFIG.wasm ? new WasmNetwork() : new ProxyNetwork();

  // Connect to parent window (if exists).
  if (window.parent != window) {
    multiConnect(network);
  }

  // Create the editor frame.
  ReactDOM.render(
    <EditorFrame
      network={network}
      body={document.querySelector('.edit-text')!.innerHTML}
    />,
    document.querySelector('#content')!,
  )

  // Connect to remote sockets.
  network.nativeConnect()
    .then(() => network.syncConnect())
    .then(() => {
      console.log('edit-text initialized.');
    });
}
