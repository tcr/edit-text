export default class Parent {
  constructor() {
    // Timer component.
    let counter = Date.now();
    setInterval(() => {
      requestAnimationFrame(() => {
        $('#timer').each(function () {
          $(this).text((((Date.now() - counter)/1000)|0) + 's');
        })
      });
    }, 1000);

    // Monkey global click button.
    let monkey = false;
    $('#action-monkey').on('click', () => {
      for (let i = 0; i < window.frames.length; i++) {
        window.frames[i].postMessage({
          'Monkey': !monkey,
        }, '*');
      }
      monkey = !monkey;
    })
  }
}
