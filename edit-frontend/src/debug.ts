declare var window: any;

let POEM = `
The Telegraphers Valentine, by J.C. Maxwell, 1860

The tendrils of my soul are twined
With thine, though many a mile apart.
And thine in close coiled circuits wind
Around the needle of my heart.

Constant as Daniel, strong as Grove.
Ebullient throughout its depths like Smee,
My heart puts forth its tide of love,
And all its circuits close in thee.

O tell me, when along the line
From my full heart the message flows,
What currents are induced in thine?
One click from thee will end my woes.

Through many a volt the weber flew,
And clicked this answer back to me;
I am thy farad staunch and true,
Charged to a volt with love for thee
`;

let int: any = null;
let delay = 0;
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
                    delay = 50;

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
                        delay = 22;
                    }
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
};

window.DEBUG = DEBUG;

export default DEBUG;
