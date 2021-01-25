use std::path::Path;

#[macro_use]
extern crate anyhow;
use anyhow::Result;

extern crate structopt;
use structopt::StructOpt;

extern crate serde_json;
extern crate serde_yaml;

use cumin::eval::eval;
use cumin::parser::cumin::cumin;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "T", long = "type", default_value = "json")]
    output_type: String,

    #[structopt(name = "INPUT", default_value = "-")]
    input_cumin: String,
}

fn cat(file_name: &String) -> String {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::{self, Read};

    let mut content = String::new();
    if file_name == "-" {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut content).unwrap();
    } else {
        let file = File::open(&file_name).unwrap();
        let mut buf_reader = BufReader::new(file);
        buf_reader.read_to_string(&mut content).unwrap();
    }
    content
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let content = cat(&opt.input_cumin);
    if let Ok((rest, cumin)) = cumin(content.as_str()) {
        if !rest.is_empty() {
            eprintln!("Parsing Stop with `{}`", rest);
            eprintln!("read conf: {:?}", &cumin);
            bail!("Parsing Failed.");
        }
        let cd = Path::new(&opt.input_cumin)
            .parent()
            .map(|path| String::from(path.to_str().unwrap()));
        let json = eval(cumin, cd)?;
        match opt.output_type.as_str() {
            "json" | "JSON" | "Json" => {
                println!("{}", json.stringify());
            }
            "yaml" | "YAML" | "Yaml" => {
                let value: serde_json::Value = serde_json::from_str(&json.stringify())?;
                let yaml_str = serde_yaml::to_string(&value)?;
                println!("{}", yaml_str);
            }
            _ => {
                bail!("Unknown format `{}`", opt.output_type);
            }
        }
    } else {
        bail!("Parse Error");
    }
    Ok(())
}
