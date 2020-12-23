pub mod lib;
use clap::{App, Arg};
use lib::commands::{init,cat_file};

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
        .subcommand(
            App::new("cat-file").arg(
            Arg::with_name("type")
            .index(1)
            .value_name("TYPE")
            .help("Specify the type")
            .required(true)
            .possible_values(&["blob", "commit","tag", "tree"])
        ).arg(
            Arg::with_name("object")
            .index(2)
            .value_name("OBJECT")
            .help("The object to display")
            .required(true)
        ))
}

fn main() {
    let matches = make_parser().get_matches();
    if matches.is_present("init") {
        let sub_matches = matches.subcommand_matches("init").unwrap();
        if let Some(path) = sub_matches.value_of("path") {
            match init(path) {
                Ok(_) => (),
                Err(err) => println!("Error initializing: {}", err),
            }
        } else {
            println!("No value given for path")
        }
    } else if matches.is_present("cat-file"){
        let sub_matches = matches.subcommand_matches("cat-file").unwrap();
        if let Some(type_str) = sub_matches.value_of("type") {
            if let Some(object) = sub_matches.value_of("object") {
                //TODO handle unwrap here better
                match cat_file(std::env::current_dir().unwrap(), type_str, object) {
                    Ok(_) => (),
                    Err(err) => println!("Error: {}", err),
                }
            } else {
                println!("No value given for object");
            }
        } else {
            println!("No value given for type");
        }
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
            if let Some(_) = matches.subcommand_matches("init").unwrap().value_of("path") {
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }
    }
}
