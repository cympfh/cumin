extern crate structopt;
use structopt::StructOpt;

use combine::parser::Parser;
use cumin::eval::eval;
use cumin::parser::config::config;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "T", long = "type", default_value = "json")]
    output_type: String,

    #[structopt(name = "INPUT")]
    input_cumin: String,
}

fn cat(file_name: String) -> String {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;
    let file = File::open(&file_name).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut content = String::new();
    buf_reader.read_to_string(&mut content).unwrap();
    content
}

fn main() {
    let opt = Opt::from_args();
    let content = cat(opt.input_cumin);
    if let Ok((conf, rest)) = config().parse(content.as_str()) {
        if !rest.is_empty() {
            eprintln!("Parsing Stop with `{}`", rest);
            eprintln!("read conf: {:?}", &conf);
            return;
        }
        let json = eval(conf);
        println!("{}", json.stringify());
    } else {
        eprintln!("Parse Error");
    }
}
