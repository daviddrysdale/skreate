// Copyright 2024-2025 David Drysdale

//! Documentation generator.

use clap::Parser;
use handlebars::JsonValue;
use regex::Regex;
use serde_json::json;
use skreate::moves;
use std::ffi::OsStr;
use std::fs::{read_dir, File};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

const TEMPLATE: &str = "template";

#[derive(Parser, Debug)]
struct Options {
    /// Input file.
    #[arg(short, long)]
    in_file: String,
    /// Output file.
    #[arg(short, long)]
    out_file: String,
    /// Directory holding examples.
    #[arg(short, long)]
    eg_dir: Option<String>,
}

fn check_dir(dir: &str) -> &Path {
    let path = Path::new(dir);
    if !path.exists() {
        eprintln!("Directory {dir} does not exist.");
        std::process::exit(1)
    }
    if !path.is_dir() {
        eprintln!("Location {dir} is not a directory.");
        std::process::exit(1)
    }
    path
}

use handlebars::JsonRender;

fn edit_helper(
    h: &handlebars::Helper,
    _hbs: &handlebars::Handlebars,
    _ctx: &handlebars::Context,
    _rc: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).unwrap();

    out.write("<a href=\"./?text=")?;
    out.write(urlencoding::encode(param.value().render().as_ref()).as_ref())?;
    out.write("\"><b><code>")?;
    out.write(param.value().render().as_ref())?;
    out.write("</code></b></a>")?;
    Ok(())
}

static NEXT: Mutex<u32> = Mutex::new(1);

fn next() -> u32 {
    let mut next = NEXT.lock().unwrap();
    let val = *next;
    *next += 1;
    val
}

fn example_helper(
    h: &handlebars::Helper,
    _hbs: &handlebars::Handlebars,
    _ctx: &handlebars::Context,
    _rc: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let semicolon_re = Regex::new(r"\s*;\s*").unwrap();

    let text: String = h.param(0).unwrap().value().render();
    // Use ';;' for actual semicolon, so temporarily make it something else.
    let text = str::replace(&text, r#";;"#, r#"@"#);
    // Convert ';' to newline (and remove whitespace).
    let text = semicolon_re.replace_all(&text, "\n");
    // Restore desired semicolons.
    let text = str::replace(&text, r#"@"#, r#";"#);
    // Now that the semicolon shenanigans are done, it's OK to put the HTML entity for double quotes (which uses a
    // semi-colon) in.
    let text = str::replace(&text, r#"""#, r#"&quot;"#);
    let num = next();

    out.write(&format!(
        r##"
<section class="skreate" id="example_{num}" data-skreate="{text}">
  <table class="inner">
    <tr>
      <td>
        <div class="editor-wrapper"><div class="editor"></div></div>
        <a href="#" class="edit">Edit</a> |
        <a href="#" class="preview">Preview</a>
      </td>
      <td><div class="diagram"></div></td>
    </tr>
  </table>
</section>
"##
    ))?;

    Ok(())
}

fn main() {
    let extension = OsStr::new("skate");
    let opts = Options::parse();

    let mut hbs = handlebars::Handlebars::new();
    hbs.register_helper("edit", Box::new(edit_helper));
    hbs.register_helper("example", Box::new(example_helper));
    hbs.register_template_file(TEMPLATE, &opts.in_file)
        .unwrap_or_else(|e| panic!("failed to load template at {}: {e:?}", opts.in_file));

    let mut examples: Vec<String> = Vec::new();
    if let Some(eg_dir) = &opts.eg_dir {
        let eg_path = check_dir(eg_dir);
        for entry in read_dir(eg_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && Some(extension) == path.extension() {
                let example = entry.file_name().into_string().unwrap();
                println!("Add '{example}' to examples list");
                examples.push(example);
            }
        }
    }
    examples.sort();

    let infos = moves::INFO;
    let mut json = json!({"infos": &infos, "examples": &examples});

    // Add an explicit discriminant for `infos.params.default` so the template can spot
    // variants that have falsy values (e.g. `default: Value::Bool(false)`).
    let jinfos: &mut JsonValue = json.as_object_mut().unwrap().get_mut("infos").unwrap();
    for jinfo in jinfos.as_array_mut().unwrap() {
        let jinfo = jinfo.as_object_mut().unwrap();
        let jparams: &mut JsonValue = jinfo.get_mut("params").unwrap();
        for jparam in jparams.as_array_mut().unwrap() {
            let jparam = jparam.as_object_mut().unwrap();
            let jdflt = jparam.get_mut("default").unwrap().as_object_mut().unwrap();
            if jdflt.contains_key("Number") {
                jdflt.insert("isNumber".to_string(), true.into());
            } else if jdflt.contains_key("Boolean") {
                jdflt.insert("isBoolean".to_string(), true.into());
            } else if jdflt.contains_key("Text") {
                jdflt.insert("isText".to_string(), true.into());
            }
        }
    }

    let filename = Path::new(&opts.out_file);
    println!("Render to {filename:?}");
    let mut outfile = File::create(filename).expect("failed to create {filename:?}");
    outfile
        .write_all(
            hbs.render(TEMPLATE, &json)
                .expect("failed to render")
                .as_bytes(),
        )
        .expect("failed to write rendered manual");
}
