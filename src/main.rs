#[macro_use]
extern crate clap;
extern crate rustache;

use std::io::Write;
use std::env;
use std::fs::{self, File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{App, ArgMatches};
use rustache::{HashBuilder, Render};

type Template = (&'static str, &'static str, u32, bool);

const REPO: &'static str = "hackgt";
const GIT_REV: &'static str = include_str!("../.git/refs/heads/master");
const ROOT_DOMAIN: &'static str = "hack.gt";

const TRAVIS_BUILD: Template =
    (include_str!("../templates/travis.d/build.sh"), ".travis.d/build.sh", 0o775, true);

const TRAVIS_META: Template =
    (include_str!("../templates/travis.yml"), ".travis.yml", 0o664, true);

const GITIGNORE: Template =
    (include_str!("../templates/gitignore"), ".gitignore", 0o664, false);

const LICENSE: Template =
    (include_str!("../templates/LICENSE"), "LICENSE", 0o664, false);

const README: Template =
    (include_str!("../templates/README.md"), "README.md", 0o664, false);

const CNAME: Template =
    (include_str!("../templates/CNAME"), "CNAME", 0o664, false);

const INDEX_HTML: Template =
    (include_str!("../templates/index.html"), "index.html", 0o664, false);

fn main() {
    // get command line args
    let args_config = load_yaml!("args.yaml");
    let matches = App::from_yaml(args_config).get_matches();

    match matches.subcommand() {
        ("init", Some(a)) => init(a, a.value_of("PATH")),
        ("test", Some(_)) => test(),

        _ => {
            writeln!(&mut std::io::stderr(),
                     "No such command, try using --help.").ok();
            ::std::process::exit(64);
        },
    }
}

fn get_root(marker: &str) -> Option<PathBuf> {
    fn check(path: &PathBuf, marker: &str) -> bool {
        let paths = fs::read_dir(path).unwrap();
        for path in paths {
            if path.unwrap().file_name().to_str().unwrap() == marker {
                return true;
            }
        }
        return false;
    }

    let mut current_dir = env::current_dir().unwrap();
    while !check(&current_dir, marker) {
        if !current_dir.pop() {
            return None;
        }
    }
    Some(current_dir)
}

fn init(matches: &ArgMatches, path: Option<&str>) {
    let path = match path {
        Some(p) => Path::new(p).to_path_buf(),
        None => env::current_dir().unwrap(),
    };

    // check if path exists, if not create it.
    let created = match fs::metadata(&path) {
        Ok(ref m) if m.is_dir() => false,
        Ok(_) => {
            writeln!(&mut std::io::stderr(),
                     "{:?} exists and it not a directory!", path).ok();
            ::std::process::exit(64);
        },
        Err(_) => {
            println!("{:?} does not exist, creating it.", path);
            fs::create_dir_all(&path).unwrap();
            true
        },
    };

    // easier if we just change to this dir
    env::set_current_dir(&path).unwrap();

    // do a quit `git init`
    if created || get_root(".git").is_none() {
        Command::new("git")
            .args(&["init"])
            .spawn()
            .expect("Failed to run `git init`!")
            .wait()
            .unwrap();
    }

    // what do we want to make?
    if matches.is_present("static") {
        init_static();
    } else if matches.is_present("jekyll") {
        unimplemented!();
    } else if matches.is_present("node") {
        unimplemented!();
    } else {
        init_deployment();
    }

    println!("\n\
              ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓\n\
              ┃ You're almost up and running! Just a few more steps: ┃\n\
              ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\n\
              \n\
              1. Create this repo on GitHub: https://github.com/HackGT/\
              \n\
              2. Hit 'Restart Build' to get your project set up with all of\n\
              HackGT's infra: https://travis-ci.org/HackGT/travis-secrets-setter\n\
              ");
}

fn init_static() {
    println!("Creating a static HTML project!");
    let files = [
        TRAVIS_BUILD,
        TRAVIS_META,
        GITIGNORE,
        LICENSE,
        README,
        CNAME,
        INDEX_HTML,
    ];

    let absolute_path = env::current_dir().unwrap();
    let basename = absolute_path.components()
        .last().unwrap().as_os_str().to_str().unwrap();

    // add all the files:
    let data = HashBuilder::new()
        .insert("project_type", "static")
        .insert("use_docker", false)
        .insert("org_name", REPO)
        .insert("namespace", "dev")
        .insert("root_domain", ROOT_DOMAIN)
        .insert("source_rev", GIT_REV.trim())
        .insert("app_name", format!("{}", basename))
        .insert("app_repo", format!("{}/{}", REPO, basename));

    gen_files(&files, &data);

    Command::new("git")
        .args(&["checkout", "-B", "gh-pages"])
        .spawn()
        .expect("Failed to switch to gh-pages branch!")
        .wait()
        .unwrap();
}

fn init_deployment() {
    println!("Creating a Docker deployment project!");
    let files = [
        TRAVIS_BUILD,
        TRAVIS_META,
        GITIGNORE,
        LICENSE,
        README,
    ];
    let absolute_path = env::current_dir().unwrap();
    let basename = absolute_path.components()
        .last().unwrap().as_os_str().to_str().unwrap();

    // add all the files:
    let data = HashBuilder::new()
        .insert("project_type", "deployment")
        .insert("use_docker", true)
        .insert("org_name", REPO)
        .insert("namespace", "static")
        .insert("root_domain", ROOT_DOMAIN)
        .insert("source_rev", GIT_REV.trim())
        .insert("app_name", format!("{}", basename))
        .insert("app_repo", format!("{}/{}", REPO, basename));

    gen_files(&files, &data);
}

fn gen_files(files: &[Template], data: &HashBuilder) {
    for &(text, path, perm, overwrite) in files.iter() {
        if !overwrite && fs::metadata(&path).is_ok() {
            continue;
        }
        println!("Writing '{}'.", path);
        let path = Path::new(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = File::create(path).unwrap();
        data.render(text, &mut file).unwrap();
        fs::set_permissions(path, Permissions::from_mode(perm)).unwrap();
    }
}

fn get_build_script() -> PathBuf {
    let mut root = match get_root(".travis.d") {
        Some(r) => r,
        None => {
            println!("Could not find build files.\n\
                      Are you sure you ran `hackgproject init`?");
            ::std::process::exit(64);
        }
    };

    root.push(".travis.d/build.sh");

    // check if path exists, if not create it.
    if fs::metadata(&root).is_err() {
        println!("Could not find build files.\n\
                  Are you sure you ran `hackgproject init`?");
        ::std::process::exit(64);
    }

    return root;
}

fn test() {
    let build_script = get_build_script();
    let root = get_root(".travis.d").unwrap();
    env::set_current_dir(&root).unwrap();

    let status = Command::new(build_script.to_str().unwrap())
        .spawn()
        .expect("Failed to run the build script.")
        .wait()
        .unwrap();

    ::std::process::exit(status.code().unwrap());
}
