import { WasmClientController as WasmClientModule } from './bindgen/edit_client';

declare var window: any;

let globalClientBindings: WasmClientModule | null = null;

let POEM = `The Telegraphers Valentine, by J.C. Maxwell, 1860

The tendrils of my soul are twined
With thine, though many a mile apart.
And thine in close coiled circuits wind
Around the needle of my heart.

Constant as Daniel, strong as Grove.
Ebullient throughout its depths like Smee,
My heart puts forth its tide of love,
And all its circuits close in thee.`;

let POEMCACHE = POEM;

let int: any = null;
let delay = 0;
let delcount = POEMCACHE.length - 3;
const DEBUG = {
    startWriter: () => {
        if (!int) {
            int = setInterval(() => {
                if (delay > 0) {
                    delay--;
                    return;
                }

                if (POEM.match(/^\n\n/)) {
                    POEM = POEM.slice(2);
                    delay = 20;

                    let KeyboardEventAny = KeyboardEvent as any;
                    let evt = new KeyboardEventAny("keydown", {
                        bubbles: true,
                        cancelable: true,
                        keyCode: 13,
                    });
                    document.dispatchEvent(evt);
                }
                else if (POEM.length > 0) {
                    let char = POEM[0];
                    POEM = POEM.slice(1);

                    let KeyboardEventAny = KeyboardEvent as any;
                    let evt = new KeyboardEventAny("keypress", {
                        bubbles: true,
                        cancelable: true,
                        charCode: char.charCodeAt(0),
                    });
                    document.dispatchEvent(evt);

                    if (POEM.match(/^\n/)) {
                        delay = 15;
                    }
                } else if (delcount > 0) {
                    delcount--;
                    let KeyboardEventAny = KeyboardEvent as any;
                    let evt = new KeyboardEventAny("keydown", {
                        bubbles: true,
                        cancelable: true,
                        keyCode: 8,
                    });
                    document.dispatchEvent(evt);
                } else if (delcount == 0) {
                    delcount = POEMCACHE.length - 3;
                    POEM = POEMCACHE;
                    delay = 3;
                }
            }, 50);
        }
    },

    stopWriter: () => {
        console.log('stop');
        if (int !== null) {
            clearInterval(int);
        }
        int = null;
    },

    asMarkdown: () => {
        if (globalClientBindings == null) {
            throw new Error('Bindings not assigned');
        }

        return globalClientBindings.asMarkdown();
    },

    asJSON: () => {
        if (globalClientBindings == null) {
            throw new Error('Bindings not assigned');
        }

        return globalClientBindings.asJSON();
    },

    typeChar: (charCode: number) => {
        let event = new (KeyboardEvent as any)("keypress", {
            bubbles: true,
            cancelable: true,
            charCode: charCode,
        });
        document.dispatchEvent(event);
    },

    root: (): HTMLElement => {
        return document.querySelector('.edit-text')! as HTMLElement;
    },

    userCarets: (): Array<Attr> => {
        return Array.from(document.querySelectorAll('.edit-text [data-tag=caret]'))
            .map(x => x as HTMLElement)
            .filter(x => x.getAttribute('data-client') == DEBUG.clientID())
            .map((x): Attr => x.getAttributeNode('data-focus')!);
    },

    clientID: (): String => {
        return globalClientBindings!.clientID();
    },

    mousedown: (x: number, y: number) => {
        let evt = new MouseEvent("mousedown", {
            bubbles: true,
            cancelable: true,
            clientX: x,
            clientY: y,
        });
        document.querySelector('.edit-text')!.dispatchEvent(evt);
    },

    mouseup: (x: number, y: number) => {
        let evt = new MouseEvent("mouseup", {
            bubbles: true,
            cancelable: true,
            clientX: x,
            clientY: y,
        });
        document.querySelector('.edit-text')!.dispatchEvent(evt);
    },

    caretToStartOfLine: () => {
        let caret = document.querySelector(`.edit-text [data-tag=caret][data-client=${JSON.stringify(DEBUG.clientID())}][data-focus=true]`);
        if (caret) {
            let edit = document.querySelector('.edit-text')!;
            let clientY = (caret.getBoundingClientRect().top + caret.getBoundingClientRect().bottom) / 2;
            let clientX = edit.getBoundingClientRect().left;
            DEBUG.mousedown(clientX, clientY);
            DEBUG.mouseup(clientX, clientY);
        } else {
            throw new Error('No caret found.');
        }
    },

    caretToEndOfLine: () => {
        let caret = document.querySelector(`.edit-text [data-tag=caret][data-client=${JSON.stringify(DEBUG.clientID())}][data-focus=true]`);
        if (caret) {
            let edit = document.querySelector('.edit-text')!;
            let clientY = (caret.getBoundingClientRect().top + caret.getBoundingClientRect().bottom) / 2;
            let clientX = edit.getBoundingClientRect().right - 1;
            DEBUG.mousedown(clientX, clientY);
            DEBUG.mouseup(clientX, clientY);
        } else {
            throw new Error('No caret found.');
        }
    },

    drawMarker(x: number, y: number): Element {
        let div = document.createElement('div');
        document.body.appendChild(div);
        div.style.cssText = `
          display: block;
          position: absolute;
          width: 20px;
          height: 20px;
          border: 2px solid red;
          border-right: 0px;
          border-bottom: 0px;
          box-sizing: border-box;
          background: transparent;
          box-shadow: 2px 2px 2px rgba(0, 0, 0, 0.3);
          top: ${y}px;
          left: ${x}px;
        `;
        return div;
    },

    randomCaretPosition: () => {
        globalClientBindings!.command(JSON.stringify({
            ControllerCommand: {
                "tag" : "RandomTarget" ,
                "fields" : { "position" : Math.random() , } ,
            },
        }));
    },

    // Bindings to global ref for client module
    // NOTE: only for debugging! bindings should not be referenced globally.

    setGlobalClientBindings: (
        bindings: WasmClientModule,
    ) => {
        globalClientBindings = bindings;
    },

    // Timings

    times: ({} as any),

    measureTime(key: string) {
        if (DEBUG.times[key]) {
            // console.warn('Duplicate time measurement being recorded:', key);
        } else {
            DEBUG.times[key] = Date.now() - (DEBUG.times['start'] as any);
            console.info('Time measurement %s:', key, DEBUG.times[key]);
        }
    },
};

DEBUG.times['start'] = Date.now();
console.info('DEBUG start.');

window.DEBUG = DEBUG;

export default DEBUG;
