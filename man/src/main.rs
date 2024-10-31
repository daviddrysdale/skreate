use skreate::{moves, params::Abbrev};

fn main() {
    let infos = moves::info();
    for info in infos {
        println!("{}: {}", info.name, info.summary);
        if !info.params.is_empty() {
            println!("Parameters:");
        }
        for param in info.params {
            print!("  {}: {} (default {}", param.name, param.doc, param.default);
            if let Some(short) = param.short {
                match short {
                    Abbrev::GreaterLess(_detents) => {
                        print!(", supported as > or < (repeated) suffix");
                    }
                    Abbrev::PlusMinus(_detents) => {
                        print!(", supported as + or - (repeated) suffix");
                    }
                }
            }
            println!(")");
        }
    }
}
