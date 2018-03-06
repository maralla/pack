use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new("pack")
        .about("Package manager for vim")
        .author(crate_authors!())
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name("list")
                .about("List installed packages")
                .arg(
                    Arg::with_name("start")
                        .long("start")
                        .short("s")
                        .conflicts_with("opt")
                        .help("List start packages"),
                )
                .arg(
                    Arg::with_name("opt")
                        .long("opt")
                        .short("o")
                        .conflicts_with("start")
                        .help("List optional packages"),
                )
                .arg(
                    Arg::with_name("category")
                        .long("category")
                        .short("c")
                        .help("List packages under this category")
                        .value_name("CATEGORY"),
                ),
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Install new packages/plugins")
                .arg(
                    Arg::with_name("opt")
                        .short("o")
                        .long("opt")
                        .help("Install plugins as opt(ional)"),
                )
                .arg(
                    Arg::with_name("category")
                        .long("category")
                        .short("c")
                        .help("Install package under provided category")
                        .default_value("default")
                        .value_name("CATEGORY"),
                )
                .arg(
                    Arg::with_name("local")
                        .short("l")
                        .long("local")
                        .help("Install local plugins"),
                )
                .arg(
                    Arg::with_name("on")
                        .long("on")
                        .help("Command for loading the plugins")
                        .value_name("LOAD_CMD"),
                )
                .arg(
                    Arg::with_name("for")
                        .long("for")
                        .help("Load this plugins for specific types")
                        .value_name("TYPES"),
                )
                .arg(
                    Arg::with_name("build")
                        .long("build")
                        .help("Build command for build package")
                        .value_name("BUILD_CMD"),
                )
                .arg(
                    Arg::with_name("threads")
                        .short("j")
                        .long("threads")
                        .help("Installing packages concurrently")
                        .value_name("THREADS"),
                )
                .arg(Arg::with_name("package").required(true).multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("uninstall")
                .about("Uninstall packages/plugins")
                .arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("remove all package related configuration as well"),
                )
                .arg(Arg::with_name("package").required(true).multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("Configure/edit the package specific configuration")
                .arg(
                    Arg::with_name("delete")
                        .short("d")
                        .long("delete")
                        .help("Delete package configuration file"),
                )
                .arg(Arg::with_name("package").required(true)),
        )
        .subcommand(
            SubCommand::with_name("move")
                .about("Move a package to a different category or make it optional.")
                .arg(
                    Arg::with_name("opt")
                        .conflicts_with("category")
                        .long("opt")
                        .short("o")
                        .help("Make package optional"),
                )
                .arg(
                    Arg::with_name("package")
                        .help("Package to move")
                        .required(true),
                )
                .arg(
                    Arg::with_name("category")
                        .conflicts_with("opt")
                        .help("Category to move the package to"),
                ),
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("Update packages")
                .arg(
                    Arg::with_name("skip")
                        .short("s")
                        .long("skip")
                        .multiple(true)
                        .help("Skip packages"),
                )
                .arg(
                    Arg::with_name("packfile")
                        .short("p")
                        .long("packfile")
                        .help("Regenerate the '_pack' file (combine all package configurations)"),
                )
                .arg(
                    Arg::with_name("threads")
                        .short("j")
                        .long("threads")
                        .help("Updating packages concurrently"),
                )
                .arg(
                    Arg::with_name("package")
                        .help("Packages to update, default all")
                        .multiple(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("generate")
                .about("Generate the pack package file")
                .help("Generate _pack.vim file which combines all package configurations"),
        )
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generates completion scripts for your shell")
                .setting(AppSettings::Hidden)
                .arg(
                    Arg::with_name("SHELL")
                        .required(true)
                        .possible_values(&["bash", "fish", "zsh"])
                        .help("The shell to generate the script for"),
                ),
        )
}
