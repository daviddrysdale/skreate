// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, { initialize, generate, ParseError } from '../pkg/skreate_wasm.js';

async function run() {
  // First up we need to actually load the wasm file, so we use the
  // default export to inform it where the wasm file is located on the
  // server, and then we wait on the returned promise to wait for the
  // wasm to be loaded.
  await init();

  // And afterwards we can use all the functionality defined in wasm.
  initialize();
}
await run();

export function set_svg(text, div) {
  var diagram_svg = generate(text);
  div.html(diagram_svg);
}

export function setup_download(div, diagram_div, get_value) {
  var download_link = div.find('.download');
  download_link.click(function(ev) {
    var svg = diagram_div.find('svg')[0];
    var width = parseInt(svg.width.baseVal.value);
    var height = parseInt(svg.height.baseVal.value);
    var data = get_value();
    var xml = '<?xml version="1.0" encoding="utf-8" standalone="no"?><!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 20010904//EN" "http://www.w3.org/TR/2001/REC-SVG-20010904/DTD/svg10.dtd"><svg xmlns="http://www.w3.org/2000/svg" width="' + width + '" height="' + height + '" xmlns:xlink="http://www.w3.org/1999/xlink"><source><![CDATA[' + data + ']]></source>' + svg.innerHTML + '</svg>';

    var a = $(this);
    a.attr("download", "diagram.svg");
    a.attr("href", "data:image/svg+xml," + encodeURIComponent(xml));
  });
}

export function setup_editor(div, autofocus, text) {
  var editor_div = div.find(".editor");
  editor_div.html(text);
  var editor = ace.edit(editor_div.get(0));
  editor.getSession().on('change', debounce(on_change, 100));
  if (autofocus) {
      editor.focus();
  }
  function getValue() {
    return editor.getValue();
  }
  var diagram_div = div.find(".diagram");

  setup_download(div, diagram_div, getValue);

  on_change();

  function on_change() {
    try {
      // Clear out old diagram and editor annotations.
      editor.getSession().setAnnotations([]);
      diagram_div.html('');

      var options = { scale: 1 };

      set_svg(editor.getValue(), diagram_div);
    } catch(err) {
      var annotation = {
        type: "error", // also warning and information
        column: 0,
        row: 0,
        text: err.message
      };
      if (err instanceof ParseError) {
        annotation.row    = err.row;
        annotation.column = err.col;
        annotation.text   = err.msg;
      }
      editor.getSession().setAnnotations([annotation]);
    }
  }
}
