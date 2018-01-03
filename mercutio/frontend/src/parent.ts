export default class Parent {
  constructor() {
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
  }
}
