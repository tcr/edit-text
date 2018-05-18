// Global CSS
import '../styles/edit.scss';

// import * as Clipboard from 'clipboard';
import * as commands from './commands';
import * as util from './util';
import * as React from 'react';

import * as ReactDOM from 'react-dom';
import axios from 'axios';
import * as route from './route';
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

function graphqlPage(id: string) {
  return axios.post(
    route.graphqlUrl(),
    {
      query:
`
query ($id: String!) { page(id: $id) { markdown }}
`,
      variables: {
         id,
      },
    }
  );
}

function graphqlCreatePage(id: string, markdown: string) {
  return axios.post(
    route.graphqlUrl(),
    {
      query:
`
mutation ($id: String!, $markdown: String!) { createPage(id: $id, markdown: $markdown) { __typename } }
`,
      variables: {
        id,
        markdown,
      },
    }
  );
}

class MarkdownModal extends React.Component {
  props: {
    markdown: string,
    onModal: Function,
  };

  state = {
    markdown: this.props.markdown,
  };

  render() {
    let self = this;
    return (
      <Modal>
        <h1>Markdown</h1>
        <p>Copy the text below. Or edit it and click load to overwrite the current page.</p>
        <textarea
          value={self.state.markdown}
          onChange={(e) => {
            this.setState({
              markdown: e.target.value,
            });
          }}
        />
        <button onClick={() => self.props.onModal(null)}>Dismiss</button>
        <button onClick={() => {
          graphqlCreatePage(route.pageId(), self.state.markdown)
          .then(req => {
            if (req.data.errors && req.data.errors.length) {
              console.log(req.data.errors);
              console.error('Error in markdown save.');
            } else {
              console.log('Update success, reloading...');
              setTimeout(() => {
                window.location.reload();
              }, 500);
            }
          })
          .catch(err => console.error(err));
        }}>Load</button>
      </Modal>
    );
  }
}

class LocalButtons extends React.Component {
  props: {
    editorID: string,
    editor: any,
    onModal: Function,
  };

  state = {};

  onMarkdownClick() {
    let self = this;
    graphqlPage(route.pageId())
    .then(res => {
      let graphql = res.data;
      let markdown = graphql.data.page.markdown;
      
      self.props.onModal(
        <MarkdownModal
          markdown={markdown}
          onModal={self.props.onModal}
        />
      );
    })
    .catch(err => {
      console.error('onSaveMarkdown:', err);
    })
  }

  toggleWidth() {
    document.body.classList.toggle('theme-column');
    if (document.body.classList.contains('theme-column')) {
      localStorage.setItem('edit-text:theme-column', '1');
    } else {
      localStorage.removeItem('edit-text:theme-column');
    }
  }

  render(): React.ReactNode {
    return (
      <div>
        <button onClick={() => this.onMarkdownClick()}>View as Markdown</button>

        <button id="width" onClick={() => this.toggleWidth()}>Toggle Page Width</button>

        <b>Client: <kbd>{this.props.editorID}</kbd></b>
      </div>
    );
  }
}

function Modal(props: any) {
  return (
    <div id="modal">
      <div id="modal-dialog">
        {props.children}
      </div>
    </div>
  );
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
    modal: React.ReactNode,
  };

  KEY_WHITELIST: any;
  network: Network;
  markdown: string;

  constructor(
    props,
  ) {
    super(props);

    this.KEY_WHITELIST = [];

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

    this.state = {
      body: this.props.body,
      buttons: [],
      editorID: '$$$$$$',
      modal: null,
    };
  }

  render(): React.ReactNode {
    return (
      <div>
        {this.state.modal}
        <div className={this.state.modal == null ? '' : 'modal-active'}>
          <div id="toolbar">
            <a href="https://github.com/tcr/edit-text" id="logo">edit-text</a>
            <NativeButtons 
              editor={this}
              buttons={this.state.buttons} 
            />,
            <LocalButtons
              editor={this}
              editorID={this.state.editorID}
              onModal={(modal) => {
                this.setState({
                  modal
                });
              }}
            />
          </div>

          <Editor 
            network={this.props.network} 
            KEY_WHITELIST={this.KEY_WHITELIST}
            content={this.state.body}
            editorID={this.state.editorID}
            disabled={!!this.state.modal}
          />

          <div id="footer">
            See more <a href="http://github.com/tcr/edit-text">on Github</a>.
          </div>
        </div>
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

  // TODO move this to a better logical location and manage local storage better
  if (localStorage.getItem('edit-text:theme-column')) {
    document.body.classList.add('theme-column');
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
