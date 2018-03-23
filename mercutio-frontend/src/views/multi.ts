export function start() {
  // Timer component.
  let counter = Date.now();
  setInterval(() => {
    requestAnimationFrame(() => {
      let timer = document.querySelector('#timer');
      if (timer !== null) {
        (timer as any).innerText = ((((Date.now() - counter)/1000)|0) + 's');
      }
    });
  }, 1000);

  // Monkey global click button.
  let monkey = false;
  document.querySelector('#action-monkey')!
  .addEventListener('click', () => {
    monkey = !monkey;
    for (let i = 0; i < window.frames.length; i++) {
      window.frames[i].postMessage({
        'Monkey': monkey,
      }, '*');
    }
    (document.querySelector('#action-monkey') as HTMLElement).style.background = monkey ? '#0f0' : 'transparent';
  })
}
