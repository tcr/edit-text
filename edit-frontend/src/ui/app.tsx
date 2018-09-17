// Global CSS
import '../../styles/edit.scss';

// import * as Clipboard from 'clipboard';
import * as React from 'react';
import * as ReactDOM from 'react-dom';
import axios from 'axios';
import * as Raven from 'raven-js';

import * as commands from '../editor/commands';
import * as route from './route';
import { Editor } from '../editor/editor';
import { AppServer, ProxyClient } from './sync';
import { NullServer, ControllerImpl, ServerImpl } from '../editor/network';
import { WasmClient, convertMarkdownToHtml, convertMarkdownToDoc } from '../editor/wasm';
import * as index from '../index';
import {vm} from '../editor/vm';

import DEBUG from '../debug';

declare var CONFIG: any;

// Check page configuration.
if (!CONFIG.configured) {
  alert('The window.CONFIG variable was not configured by the server!')
}

function UiElement(
  props: {
    editor: EditorFrame,
  },
  element: any,
  i = Math.random(),
) {
  if ('Button' in element) {
    let button = element.Button;
    return (
      <button
        key={i}
        onClick={
          () => props.editor.client.sendCommand(commands.Button(button[1]))
        }
        className={button[2] ? 'active' : ''}
      >{button[0]}</button>
    )
  } else if ('ButtonGroup' in element) {
    return (
      <div className="menu-buttongroup" key={i}>
        {element.ButtonGroup.map((x: any, i: number) => UiElement(props, x, i))}
      </div>
    )
  }
  return null;
}

function NativeButtons(
  props: {
    editor: EditorFrame,
    buttons: Array<any>
  },
) {
  if (!props.buttons.length) {
    return (
      <div id="native-buttons">Loading...</div>
    );
  }
  return (
    <div id="native-buttons">{
      props.buttons.map((x, i) => UiElement(props, x, i))
    }</div>
  );
}

export function graphqlPage(id: string) {
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

export function graphqlCreatePage(id: string, markdown: string) {
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
    onModal: (modal: React.ReactNode) => void,
  };

  state = {
    markdown: this.props.markdown,
  };

  render() {
    let self = this;
    return (
      <Modal>
        <h1>Markdown Import/Export</h1>
        <p>The document is displayed as Markdown in the textarea below. Feel free to copy it, or modify it and click "Import" to overwrite the current page with your changes.</p>
        <textarea
          value={self.state.markdown}
          onChange={(e) => {
            this.setState({
              markdown: e.target.value,
            });
          }}
        />
        <div className="modal-buttons">
          <button className="dismiss" onClick={() => self.props.onModal(null)}>Back</button>
          <div style={{flex: 1}} />
          <button className="load" onClick={() => {
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
          }}>Import</button>
        </div>
      </Modal>
    );
  }
}

class LocalButtons extends React.Component {
  props: {
    editorID: string,
    editor: any,
    onModal: (modal: React.ReactNode) => void,
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
    if (!document.body.classList.contains('theme-column')) {
      localStorage.setItem('edit-text:theme-wide', '1');
    } else {
      localStorage.removeItem('edit-text:theme-wide');
    }
  }

  render(): React.ReactNode {
    return (
      <div className="menu-buttongroup" style={{marginRight: 0}}>
        <button onClick={() => this.onMarkdownClick()}>Load/Save</button>

        <button id="width" onClick={() => this.toggleWidth()}>Page Width</button>

        <b style={{marginLeft: 10, whiteSpace: 'nowrap'}}>
          Client: <kbd tabIndex={0}>{this.props.editorID}</kbd>
        </b>
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


export type NoticeProps = {
  element: React.ReactNode,
  level: 'notice' | 'error',
};

function FooterNotice(props: {
  onDismiss: () => void,
  children: React.ReactNode,
  level: string,
}) {
  return (
    <div className={`footer-bar ${props.level}`}>
      <div>{props.children}</div>
      <span onClick={props.onDismiss}>&times;</span>
    </div>
  );
}

type EditorFrameProps = {
  network: ServerImpl,
  client: ControllerImpl,
  body: string,
};

// Initialize child editor.
export class EditorFrame extends React.Component {
  props: EditorFrameProps;

  state: {
    body: string,
    buttons: any,
    editorID: string,
    modal: React.ReactNode,
    notices: Array<NoticeProps>,
  };

  KEY_WHITELIST: any;
  network: ServerImpl;
  client: ControllerImpl;
  markdown: string;

  editor: Editor | null;

  constructor(
    props: EditorFrameProps,
  ) {
    super(props);

    this.KEY_WHITELIST = [];

    this.network = props.network;
    this.client = props.client;

    this.client.onMessage = this.onClientToFrontendCommand.bind(this);

    // Background colors.
    // TODO make these actionable on this object right?
    this.client.onClose = function () {
      document.body.style.background = 'red';
      console.error('!!! client close');
    };
    this.network.onClose = function () {
      document.body.style.background = 'red';
      console.error('!!! server close');
    };

    this.state = {
      body: this.props.body,
      buttons: [],
      editorID: '$$$$$$',
      modal: null,
      notices: [],
    };
  }

  showNotification(notice: NoticeProps) {
    this.setState({
      notices: this.state.notices.slice().concat([notice]),
    })
  }

  render(): React.ReactNode {
    let modalClass = this.state.modal == null ? '' : 'modal-active';
    let editBoundary = null;
    return (
      <div className="fullpage">
        {this.state.modal}
        <div id="root-layout" className={modalClass}>
          <div id="toolbar">
            <a href="/" id="logo">{CONFIG.title}</a>
            <NativeButtons
              editor={this}
              buttons={this.state.buttons} 
            />
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

          <div id="edit-layout">
            <div
              id="edit-outer"
              ref={r => editBoundary = r}
              onMouseDown={e => {
                this.editor!.onMouseDown(e as any);
              }}
              onMouseUp={e => {
                this.editor!.onMouseUp(e as any);
              }}
              onMouseMove={e => {
                this.editor!.onMouseMove(e as any);
              }}
            >
              <Editor 
                controller={this.props.client} 
                KEY_WHITELIST={this.KEY_WHITELIST}
                content={this.state.body}
                editorID={this.state.editorID}
                disabled={!!this.state.modal}
                ref={r => this.editor = r}
              />
            </div>
          </div>
        </div>
        <div id="footer">{
          this.state.notices.map((x, key) => {
            return (
              <FooterNotice 
                key={key}
                onDismiss={() => {
                  let notices = this.state.notices.slice();
                  notices.splice(key, 1);
                  this.setState({
                    notices,
                  });
                }}
                level={x.level}
              >
                {x.element}
              </FooterNotice>
            );
          })
        }</div>
      </div>
    );
  }

  // Received message on native socket
  onClientToFrontendCommand(parse: any) {
    const editor = this;

    if (parse.Init) {
      let editorID = parse.Init;

      this.setState({
        editorID,
      });

      console.info('Editor "%s" connected.', editorID);

      // Log the editor ID.
      Raven.setExtraContext({
        editor_id: editorID,
      });
    }

    else if (parse.Update) {
      // Update page content
      // console.groupCollapsed('Parse Update');
      // console.log(parse.Update);
      let programs = JSON.parse(parse.Update[0]);
      programs.forEach((program: any) => {
        // console.log('🚗🚗🚗🚗', program, '\n');
        this.editor!._runProgram(program);

        // Corrections
        // while (true) {
        //   let unjoinedSpans = document.querySelector('.edit-text span.Normie + span.Normie');
        //   if (unjoinedSpans === null) {
        //     break;
        //   }
        //   let right = unjoinedSpans;
        //   let left = right.previousSibling;
        //   while (right.childNodes.length) {
        //     left!.appendChild(right.firstChild!);
        //   }
        //   right!.parentNode!.removeChild(right);
        //   left!.normalize();
        // }

        // console.log('👿👿👿👿👿👿', document.querySelector('.edit-text')!.innerHTML);
      });
      // console.log(parse.Update[0]);
      // console.log(document.querySelector('.edit-text')!.innerHTML);
      // console.groupEnd();
      // this.setState({
      //   body: JSON.stringify(parse.Update[0], null, '  '),
      // });
    }

    else if (parse.UpdateFull) {
      DEBUG.measureTime('first-update');

      this.editor!._setHTML(parse.UpdateFull);
      // Update page content
      // this.setState({
      //   body: parse.UpdateFull[0],
      // });
    }

    else if (parse.Controls) {
      // console.log('SETUP CONTROLS', parse.Controls);

      // Update the key list in-place.
      editor.KEY_WHITELIST.splice.apply(editor.KEY_WHITELIST,
        [0, 0].concat(parse.Controls.keys.map((x: any) => ({
          keyCode: x[0],
          metaKey: x[1],
          shiftKey: x[2],
        })))
      );

      // Update buttons view
      // this.setState({
      //   buttons: parse.Controls.buttons,
      // });

      DEBUG.measureTime('interactive');
    }

    else {
      console.error('Unknown packet:', parse);
    }
  }
}


function multiConnect(client: ControllerImpl) {
  // Blur/Focus classes.
  window.addEventListener('focus', () => {
    document.body.classList.remove('blurred');
  });
  window.addEventListener('blur', () => {
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
      client.sendCommand(commands.Monkey(msg.Monkey));
    }
  };
}


class EditText extends React.Component {
  props: {
    client: WasmClient,
    markdown: string,
    onChange: Function | null,
  };

  state = {
    content: convertMarkdownToHtml(this.props.markdown),
    whitelist: [],
  };

  render() {
    return (
      <Editor
        editorID={'$local'}
        disabled={false}
        controller={this.props.client}
        content={this.state.content}
        KEY_WHITELIST={this.state.whitelist}
      />
    );
  }

  componentDidMount() {
    this.props.client.onMessage = (parse: any) => {
      if (parse.Init) {
        // let editorID = parse.Init;
  
        // this.setState({
        //   editorID,
        // });
  
        // console.info('Editor "%s" connected.', editorID);
  
        // // Log the editor ID.
        // Raven.setExtraContext({
        //   editor_id: editorID,
        // });
      }
  
      else if (parse.Update) {
        // Update page content
        this.setState({
          content: parse.Update[0],
        });

        // TODO generate markdown from client now
        // if (this.props.onChange !== null) {
        //   this.props.onChange(parse.Update[1]);
        // }
      }
  
      else if (parse.UpdateFull) {
        // Update page content
        this.setState({
          content: parse.UpdateFull,
        });

        // TODO generate markdown from client now
        // if (this.props.onChange !== null) {
        //   this.props.onChange(parse.UpdateFull[1]);
        // }
      }

      else if (parse.Controls) {
        // console.log('SETUP CONTROLS', parse.Controls);
  
        // Update the key list in-place.
        this.setState({
          whitelist: parse.Controls.keys.map((x: any) => ({
            keyCode: x[0],
            metaKey: x[1],
            shiftKey: x[2],
          })),
        });
      }
    };

    this.props.client
      .connect(() => {})
      .then(() => {
        console.log('Loading static editor.');
        this.props.client.clientBindings.command(JSON.stringify({
          ClientCommand: {
            Init: ["$local", convertMarkdownToDoc(this.props.markdown), 100],
          } 
        }));
      });
  }
}

// export function start() {
export function start_standalone() {
  index.getWasmModule()
  .then(() => {
    let client = new WasmClient();

    // Create the editor frame.
    ReactDOM.render(
      <div style={{display: 'flex', height: '100%', width: '100%'}}>
        <div style={{flex: 1}}>
          <EditText
            client={client}
            markdown={"# Most of all\n\nThe world is a place where parts of wholes are perscribred"}
            onChange={(markdown: string) => {
              // TODO not visible until styles are encapsulated.
              // TODO edit-text needs a markdown viewer split pane :P.
              document.getElementById('mdpreview')!.innerText = markdown;
            }}
          />
        </div>
        <div style={{background: '#fef', flex: 1, padding: '20px'}}>
          <pre id="mdpreview"></pre>
        </div>
      </div>,
      document.querySelector('#content')!,
    );
  });
}

export function start() {
// export function start_app() {
  let server: ServerImpl;
  let client: ControllerImpl;

  // Wasm and Proxy implementations
  if (CONFIG.wasm) {
    let wasmClient = new WasmClient();
    let wasmServer = new AppServer();

    // Link them.
    wasmClient.server = wasmServer;
    wasmServer.client = wasmClient;

    client = wasmClient;
    server = wasmServer;
  } else {
    client = new ProxyClient();
    server = new NullServer();
  }

  // Connect to parent window (if exists).
  if (window.parent != window) {
    multiConnect(client);
  }

  // TODO move this to a better logical location and manage local storage better
  if (localStorage.getItem('edit-text:theme-wide')) {
    document.body.classList.remove('theme-column');
  }

  // TODO fix the adding of editing-blurred to the bdy
  // document.addEventListener('focus', () => {
  //   // console.log('(page focus)');
  //   document.body.classList.remove('editing-blurred');
  // });
  // document.addEventListener('blur', () => {
  //   // console.log('(page blur)');
  //   document.body.classList.add('editing-blurred');
  // });
  // document.body.classList.add('editing-blurred');

  // Create the editor frame.
  let editorFrame: EditorFrame | null;
  ReactDOM.render(
    <EditorFrame
      network={server}
      client={client}
      body={document.querySelector('.edit-text')!.innerHTML}
      ref={c => editorFrame = c}
    />,
    document.querySelector('#content')!,
    () => {
      // Default notification
      if (!sessionStorage.getItem("its-only-funny-once")) {
        editorFrame!.showNotification({
          element: (<div>
            Check out <a href="http://github.com/tcr/edit-text">edit-text</a> on Github for more information.
          </div>),
          level: 'notice',
        });
        sessionStorage.setItem("its-only-funny-once", 'true');
      }

      // Connect to remote sockets.
      // TODO why is nativeConnect an error?
      DEBUG.measureTime('connect-client');
      client
        .connect(() => {
          // TODO
        })
        .then(() => {
          server.connect((message: React.ReactNode) => {
            editorFrame!.showNotification({
              element: message,
              level: 'error',
            });
          });
        });
    }
  );
}
