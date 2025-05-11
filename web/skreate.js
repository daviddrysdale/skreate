// Copyright 2024-2025 David Drysdale

// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, { initialize, generate, generate_with_positions, canonicalize, canonicalize_vert, ParseError } from './pkg/skreate_wasm.js';

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
  var result = generate_with_positions(text);
  var diagram_svg = result.svg;
  div.html(diagram_svg);
  return result.positions;
}

function set_svg_with_events(editor, div) {
  var text = editor.getValue();
  highlight_elt(null);
  highlight_text(editor, null, null);

  var result = generate_with_positions(text);
  div.html(result.svg);
  var positions = result.positions;
  var timings = result.timings;

  for (const text_pos of positions) {
    $( "[id^='"+text_pos+"']" ).each( function() {
      this.addEventListener("mouseover", () => {
        highlight_text(editor, this.id, true);
      });
      this.addEventListener("mouseout", () => {
        highlight_text(editor, this.id, false);
      });
      this.addEventListener("click", () => {
        move_cursor(editor, this.id);
      });
    });
}

  setup_playthrough(editor, positions, timings);
  return positions;
}

function setup_playthrough(editor, positions, timings) {
  var playthrough_link = document.getElementById('playthrough');
  if (playthrough_link) {
    playthrough_link.onclick = function() {
      playthrough(editor, positions, timings, 500)
    };
  }
}

export function setup_download(div, diagram_div, get_value) {
  var download_link = div.find('.download');
  var input_link = div.find('.fileInput')[0];
  download_link.click(function(ev) {
    var text = get_value();
    var xml = generate(text);

    var filename;
    if (input_link.files.length == 0) {
      filename = "diagram.svg";
    } else {
      filename = input_link.files[0].name;
      filename = filename.replace(/\.skate$/, "");
      filename = filename + ".svg";
    }

    var a = $(this);
    a.attr("download", filename);
    a.attr("href", "data:image/svg+xml," + encodeURIComponent(xml));
  });
}

export function setup_save(div, diagram_div, get_value) {
  var save_link = div.find('.save');
  var input_link = div.find('.fileInput')[0];
  save_link.click(function(ev) {
    var filename;
    if (input_link.files.length == 0) {
      filename = "diagram.skate";
    } else {
      filename = input_link.files[0].name;
    }
    var text = get_value();
    var blob = new Blob([text], {type: 'text/plain;charset=utf-8'});
    var a = $(this);
    a.attr("download", filename);
    a.attr("href", URL.createObjectURL(blob));
  });
}

export function setup_load(div, diagram_div) {
  var input_link = div.find('.fileInput')[0];
  input_link.addEventListener('change', () => {
    if (input_link.files.length == 0) {
      alert("No file selected");
    } else {
      var fileobj = input_link.files[0];
      const reader = new FileReader();
      reader.onload = function fileReadDone() {
        var text = reader.result;
        console.log("Loading:\n" + text);
        var editor_div = div.find(".editor");
        var editor = ace.edit(editor_div.get(0));
        editor.setValue(text);
      }
      console.log("Loading from " + fileobj.name);
      reader.readAsText(fileobj);
    }
  });
}

export function setup_preview(div, get_value) {
  var preview_link = div.find('.preview');
  preview_link.click(function(ev) {
    var text = get_value();
    $(this).attr("href", "preview.html?text=" + canonicalize(text));
  });
}

export function setup_edit(div, get_value) {
  var edit_link = div.find('.edit');
  edit_link.click(function(ev) {
    var text = get_value();
    $(this).attr("href", "index.html?text=" + canonicalize_vert(text));
  });
}

function parse_text_pos(text_pos) {
  // This also copes with suffixed versions (e.g. "r_0_c_0_5_n2").
  var re = /r_(\d+)_c_(\d+)_(\d+)/;
  var m = text_pos.match(re);
  if (!m) {
    return null;
  }
  var row = Number(m[1]);
  var col = Number(m[2]);
  var endcol = Number(m[3]);
  return { row: row, col: col, endcol: endcol};
}

function move_cursor(editor, text_pos) {
  var pos = parse_text_pos(text_pos);
  if (pos) {
      editor.moveCursorTo(pos.row, pos.col);
  }
}

function change_elt_colour(text_pos, colour, skip_reps=false) {
  $( "[id^='"+text_pos+"']" ).each( function() {
    if (skip_reps && this.id.includes("_rep")) {
      return;
    }
    const cur_style = this.getAttribute("style");
    let stroke_regexp = /stroke: *[^;]+;/;
    let red_stroke = cur_style.replace(stroke_regexp, "stroke:" + colour + ";");
    let fill_regexp = /fill: *[^;]+;/;
    let red_fill = red_stroke.replace(fill_regexp, "fill:" + colour + ";");
    this.setAttribute("style", red_fill);
  })
}

var currently_highlighted;
function highlight_elt(text_pos, skip_reps=false) {
  if (text_pos == currently_highlighted) {
    return;
  }
  if (currently_highlighted) {
    console.log("clear highlight for " + currently_highlighted);
    change_elt_colour(currently_highlighted, "black");
    currently_highlighted = null;
  }
  currently_highlighted = text_pos;
  if (text_pos) {
    console.log("highlight " + text_pos);
    change_elt_colour(text_pos, "red", skip_reps);
  }
}

var AceRange = ace.require('ace/range').Range;

var current_text_marker;
var marked_text_position;
function highlight_text(editor, text_pos, enabled) {
  if (enabled && text_pos == marked_text_position) {
      return;
  }
  if (current_text_marker) {
    console.log("clear text marker_id=" + current_text_marker + " at " + marked_text_position);
    editor.getSession().removeMarker(current_text_marker);
    current_text_marker = null;
    marked_text_position = null;
  }

  if (enabled) {
    var pos = parse_text_pos(text_pos);
    var range = new AceRange(pos.row, pos.col, pos.row, pos.endcol);
    current_text_marker = editor.getSession().addMarker(range, "ace_selected_word", "text");
    console.log("set text marker at " + text_pos + " == " + range + " => marker_id=" + current_text_marker);
    marked_text_position = text_pos;
  }
}

function playthrough(editor, positions, timings, timeout) {
  if (!positions || positions.length === 0) {
    highlight_elt(null);
    highlight_text(editor, null, false);
    return;
  }
  let text_pos = positions[0];
  let time = timings[0];
  let rest = positions.slice(1);
  let rest_timings = timings.slice(1);
  if (time > 0) {
    console.log("highlight " + text_pos + " for " + time);
  }
  if (text_pos.includes("_rep")) {
    highlight_elt(text_pos);
  } else {
    highlight_elt(text_pos, /* skip_reps= */ true);
  }
  highlight_text(editor, text_pos, true);
  setTimeout(() => {
    playthrough(editor, rest, rest_timings, timeout)
  }, time * timeout);
}

export function setup_editor(div, autofocus, text) {
  var editor_div = div.find(".editor");
  editor_div.html(text);
  var editor = ace.edit(editor_div.get(0));
  editor.getSession().on('change', debounce(on_change, 400));
  if (autofocus) {
      editor.focus();
  }
  function getValue() {
    return editor.getValue();
  }
  var diagram_div = div.find(".diagram");

  setup_save(div, diagram_div, getValue);
  setup_load(div, diagram_div);
  setup_download(div, diagram_div, getValue);
  setup_preview(div, getValue);
  setup_edit(div, getValue);

  on_change();

  function on_change() {
    try {
      // Clear out old editor annotations.
      editor.getSession().setAnnotations([]);
      var options = { scale: 1 };

      var positions = set_svg_with_events(editor, diagram_div);

      editor.session.selection.on('changeCursor', function(e) {
        var cursor = editor.selection.getCursor();
        var to_highlight = null;
        for (const text_pos of positions) {
          var pos = parse_text_pos(text_pos);
          if (!pos) {
            continue;
          }
          if ((cursor.row == pos.row) &&
              (cursor.column >= pos.col) && (cursor.column <= pos.endcol)) {
            to_highlight = text_pos;
            break;
          }
        }
        highlight_elt(to_highlight);
      });
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

// Return the text content of the URL or an error.  Loads the URL synchronously.
export function load_skate(url) {
  var result;
  function listener() {
    result = this.responseText;
  }
  var xhr = new XMLHttpRequest();
  xhr.addEventListener("load", listener);
  xhr.open("GET", url, /* async= */false);
  xhr.setRequestHeader("Cache-Control", "no-cache, no-store, max-age=0");
  xhr.send();

  return result;
}
