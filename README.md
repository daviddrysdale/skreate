# Figure Skating Diagram Generator

This repo holds the source code for a [browser-based diagram generator](https://lurklurk.org/skreate), for creating
figure skating choreography diagrams from a text description.

The majority of the code is written in [Rust](https://www.rust-lang.org/), and compiled to [WebAssembly
(Wasm)](https://webassembly.org/) to run in the browser. There's also a small amount of JavaScript to glue things
together, which relies heavily on various open-source components:

- [ACE editor](https://github.com/ajaxorg/ace)
- [jQuery](https://jquery.com/)
- [Underscore](https://underscorejs.org/)

The `web/` directory holds the assembled components of the generator page: HTML, CSS, JavaScript and Wasm.
A web server that serves from this directory (e.g. `make serve`) will show the diagram generator page.
