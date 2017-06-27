#[macro_use]
extern crate clap;
extern crate rustache;

use std::io::Write;
use std::env;
use std::fs::{self, File};
use std::path::Path;

use clap::App;

use rustache::{HashBuilder, Render};

const GIT_REV: &'static str = include_str!("../.git/refs/heads/master");

const FILES: [(&'static str, &'static str); 2] = [
    (include_str!("../templates/travis.d/build.sh"), ".travis.d/build.sh"),
    (include_str!("../templates/travis.yml"), ".travis.yml"),
];

fn main() {
    // get command line args
    let args_config = load_yaml!("args.yaml");
    let matches = App::from_yaml(args_config).get_matches();

    match matches.subcommand() {
        ("init", Some(a)) => init(a.value_of("PATH")),

        _ => {
            writeln!(&mut std::io::stderr(),
                     "No such command, try using --help.").ok();
            ::std::process::exit(64);
        },

    }
}

fn init(path: Option<&str>) {
    let path = match path {
        Some(p) => Path::new(p).to_path_buf(),
        None => env::current_dir().unwrap(),
    };

    // check if path exists, if not create it.
    match fs::metadata(&path) {
        Ok(ref m) if m.is_dir() => {},
        Ok(_) => {
            writeln!(&mut std::io::stderr(),
                     "{:?} exists and it not a directory!", path).ok();
            ::std::process::exit(64);
        },
        Err(_) => {
            println!("{:?} does not exist, creating it.", path);
            fs::create_dir_all(&path).unwrap();
        },
    }

    // easier if we just change to this dir
    env::set_current_dir(&path).unwrap();

    // add all the files:
    let data = HashBuilder::new()
        .insert("source_rev", GIT_REV);

    for &(text, path) in FILES.iter() {
        println!("Writing '{}'.", path);
        let path = Path::new(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = File::create(path).unwrap();
        data.render(text, &mut file).unwrap();
    }

    println!("\n\
        You're almost up and running!\n\
        Head over to https://travis-ci.org/profile/HackGT\n\
        and enable travis for this repository to get started!");
}
