import 'bootstrap/dist/css/bootstrap.min.css';
import './mote.scss';
import * as commands from './commands.ts';
import Editor from './editor.ts';
import Parent from './parent.ts';

import $ from 'jquery';
import bootstrap from 'bootstrap';
import bootbox from 'bootbox';

// Consume bootstrap so bootbox works.
bootstrap;

// Blur/Focus classes.
$(window).on('focus', () => $(document.body).addClass('focused'));
$(window).on('blur', () => $(document.body).removeClass('focused'));

// Entry.
if ((<any>window).MOTE_ENTRY == 'index') {
  new Parent();
}
else if ((<any>window).MOTE_ENTRY == 'client') {
  let editor = new Editor($('#mote'), (location.search || '').substr(1));

  editor.syncConnect();
  editor.nativeConnect();
}