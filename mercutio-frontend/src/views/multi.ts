export function start() {
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
    monkey = !monkey;
    for (let i = 0; i < window.frames.length; i++) {
      window.frames[i].postMessage({
        'Monkey': monkey,
      }, '*');
    }
    $('#action-monkey').css('background', monkey ? '#0f0' : 'transparent');
  })
}
