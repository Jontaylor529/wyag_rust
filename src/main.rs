pub mod lib;
use lib::*;
use clap::{App, Arg, SubCommand};

fn make_parser() -> App<'static, 'static> {
    App::new("wyag")
        .version("0.1")
        .author("Jonathan Taylor")
        .about("Learner's git implementation (not fully featured)")
        .subcommand(
            App::new("init").arg(
                Arg::with_name("path")
                    .index(1)
                    .value_name("PATH")
                    .default_value(".")
                    .help("Directory in which to initialize new repo")
                    .required(true),
            ),
        )
}

fn main() {
    let matches = make_parser().get_matches();
    if matches.is_present("init") {
        let sub_matches = matches.subcommand_matches("init").unwrap();
        if let Some(path) = sub_matches.value_of("path") {
            match GitRepository::init(path) {
                Ok(_) => (),
                Err(err) => println!("Error initializing: {}", err),
            }
        } else {
            println!("No value given for path")
        }
    }
    else {
        println!("not  a valid command")
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_init() {
        let parser = make_parser();
        let res_path = ["C:\\", "Users", "username", "repos", "repo_name"]
            .iter()
            .collect::<PathBuf>();
        let res_path_str = res_path.to_str().unwrap();
        let args = ["wyag", "init", res_path_str];

        let matches = parser.get_matches_from(args.iter());

        if matches.is_present("init") {
            if let Some(path) = matches.subcommand_matches("init").unwrap().value_of("path") {
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }
    }
}
