use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("pack")
        .about("Package manager for vim")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("list")
                .about("List installed packages")
                .arg(
                    Arg::new("start")
                        .long("start")
                        .short('s')
                        .conflicts_with("opt")
                        .action(ArgAction::SetTrue)
                        .help("List start packages"),
                )
                .arg(
                    Arg::new("opt")
                        .long("opt")
                        .short('o')
                        .conflicts_with("start")
                        .action(ArgAction::SetTrue)
                        .help("List optional packages"),
                )
                .arg(
                    Arg::new("detached")
                        .long("detached")
                        .short('d')
                        .action(ArgAction::SetTrue)
                        .help("List detached(untracked) packages"),
                )
                .arg(
                    Arg::new("category")
                        .long("category")
                        .short('c')
                        .help("List packages under this category")
                        .value_name("CATEGORY"),
                ),
        )
        .subcommand(
            Command::new("install")
                .about("Install new packages/plugins")
                .arg(
                    Arg::new("opt")
                        .short('o')
                        .long("opt")
                        .action(ArgAction::SetTrue)
                        .help("Install plugins as opt(ional)"),
                )
                .arg(
                    Arg::new("category")
                        .long("category")
                        .short('c')
                        .help("Install package under provided category")
                        .default_value("default")
                        .value_name("CATEGORY"),
                )
                .arg(
                    Arg::new("local")
                        .short('l')
                        .long("local")
                        .action(ArgAction::SetTrue)
                        .help("Install local plugins"),
                )
                .arg(
                    Arg::new("on")
                        .long("on")
                        .help("Command for loading the plugins")
                        .value_name("LOAD_CMD"),
                )
                .arg(
                    Arg::new("for")
                        .long("for")
                        .help("Load this plugins for specific types")
                        .value_name("TYPES"),
                )
                .arg(
                    Arg::new("build")
                        .long("build")
                        .help("Build command for build package")
                        .value_name("BUILD_CMD"),
                )
                .arg(
                    Arg::new("threads")
                        .short('j')
                        .long("threads")
                        .help("Installing packages concurrently")
                        .value_name("THREADS")
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(Arg::new("package").action(ArgAction::Append).num_args(1..)),
        )
        .subcommand(
            Command::new("uninstall")
                .about("Uninstall packages/plugins")
                .arg(
                    Arg::new("all")
                        .short('a')
                        .long("all")
                        .action(ArgAction::SetTrue)
                        .help("remove all package related configuration as well"),
                )
                .arg(Arg::new("package").required(true).action(ArgAction::Append).num_args(1..)),
        )
        .subcommand(
            Command::new("config")
                .about("Configure/edit the package specific configuration")
                .arg(
                    Arg::new("delete")
                        .short('d')
                        .long("delete")
                        .action(ArgAction::SetTrue)
                        .help("Delete package configuration file"),
                )
                .arg(Arg::new("package").required(true)),
        )
        .subcommand(
            Command::new("move")
                .about("Move a package to a different category or make it optional.")
                .arg(
                    Arg::new("opt")
                        .conflicts_with("category")
                        .long("opt")
                        .short('o')
                        .action(ArgAction::SetTrue)
                        .help("Make package optional"),
                )
                .arg(Arg::new("package").help("Package to move").required(true))
                .arg(
                    Arg::new("category")
                        .conflicts_with("opt")
                        .help("Category to move the package to"),
                ),
        )
        .subcommand(
            Command::new("update")
                .about("Update packages")
                .arg(
                    Arg::new("skip")
                        .short('s')
                        .long("skip")
                        .action(ArgAction::Append)
                        .help("Skip packages"),
                )
                .arg(
                    Arg::new("packfile")
                        .short('p')
                        .long("packfile")
                        .action(ArgAction::SetTrue)
                        .help("Regenerate the '_pack' file (combine all package configurations)"),
                )
                .arg(
                    Arg::new("threads")
                        .short('j')
                        .long("threads")
                        .help("Updating packages concurrently")
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(
                    Arg::new("package")
                        .help("Packages to update, default all")
                        .action(ArgAction::Append)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("generate")
                .about("Generate the pack package file")
        )
        .subcommand(
            Command::new("completions")
                .about("Generates completion scripts for your shell")
                .hide(true)
                .arg(
                    Arg::new("SHELL")
                        .required(true)
                        .value_parser(["bash", "fish", "zsh"])
                        .help("The shell to generate the script for"),
                ),
        )
}
