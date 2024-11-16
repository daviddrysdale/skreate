use serde_json::json;
use skreate::moves;

const TEMPLATE: &str = "template";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("No argument provided");
        std::process::exit(1)
    }
    let filename = &args[1];
    let mut hbs = handlebars::Handlebars::new();
    hbs.register_template_file(TEMPLATE, filename)
        .expect(&format!("failed to load template at {filename}"));

    let infos = moves::info();
    let json = json!({"infos": &infos});
    println!(
        "{}",
        hbs.render(TEMPLATE, &json)
            .expect("failed to render template")
    );
}
