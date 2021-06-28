use clap::{crate_authors, crate_version, App, Arg, ArgMatches, SubCommand};
use thiserror::Error;
use zombiesplit::{config, model::game::category::ShortDescriptor, Zombie};

fn main() {
    run().unwrap()
}

fn run() -> anyhow::Result<()> {
    env_logger::try_init()?;

    let matches = app().get_matches();
    let cfg = config::System::load(matches.value_of("config").unwrap())?;
    let zombie = Zombie::new(cfg)?;

    match matches.subcommand() {
        ("init", Some(sub_m)) => run_init(zombie, sub_m),
        ("add-game", Some(sub_m)) => run_add_game(zombie, sub_m),
        ("add-run", Some(sub_m)) => run_add_run(zombie, sub_m),
        ("list-runs", Some(sub_m)) => run_list_runs(zombie, sub_m),
        ("run", Some(sub_m)) => run_run(zombie, sub_m),
        _ => Ok(()),
    }
}

fn run_init(zombie: Zombie, _matches: &ArgMatches) -> anyhow::Result<()> {
    zombie.init_db()?;
    Ok(())
}

fn run_add_game(mut zombie: Zombie, matches: &ArgMatches) -> anyhow::Result<()> {
    let path = matches.value_of("game").ok_or(Error::Game)?;
    zombie.add_game(path)?;
    Ok(())
}

fn run_list_runs(zombie: Zombie, matches: &ArgMatches) -> anyhow::Result<()> {
    zombie.list_runs(&get_short_descriptor(matches)?)?;
    Ok(())
}

fn run_add_run(mut zombie: Zombie, matches: &ArgMatches) -> anyhow::Result<()> {
    let path = matches.value_of("run").ok_or(Error::Run)?;
    zombie.add_run(path)?;
    Ok(())
}

fn run_run(zombie: Zombie, matches: &ArgMatches) -> anyhow::Result<()> {
    zombie.run(&get_short_descriptor(matches)?)?;
    Ok(())
}

fn get_short_descriptor(matches: &ArgMatches) -> Result<ShortDescriptor, Error> {
    let game = matches.value_of("game").ok_or(Error::Game)?;
    let category = matches.value_of("category").ok_or(Error::Category)?;
    Ok(ShortDescriptor::new(game, category))
}

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("zombiesplit")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("config")
                .help("use this system config file")
                .long("config")
                .default_value("sys.toml"),
        )
        .subcommand(init_subcommand())
        .subcommand(add_game_subcommand())
        .subcommand(add_run_subcommand())
        .subcommand(list_runs_subcommand())
        .subcommand(run_subcommand())
}

fn init_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init").about("Initialises zombiesplit's database")
}

fn list_runs_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("list-runs")
        .about("lists all runs stored for a category")
        .arg(Arg::with_name("game").help("The game to query").index(1))
        .arg(
            Arg::with_name("category")
                .help("The category to query")
                .index(2),
        )
}

fn run_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("run")
        .about("starts a zombiesplit session")
        .arg(Arg::with_name("game").help("The game to run").index(1))
        .arg(
            Arg::with_name("category")
                .help("The category to run")
                .index(2),
        )
}

fn add_game_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("add-game")
        .about("adds a game from its TOML description")
        .arg(
            Arg::with_name("game")
                .help("Path to game file to load")
                .index(1),
        )
}

fn add_run_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("add-run")
        .about("adds a run from its TOML description")
        .arg(
            Arg::with_name("run")
                .help("Path to run file to load")
                .index(1),
        )
}

#[derive(Debug, Error)]
enum Error {
    /// Error getting a category from the command line.
    #[error("no category provided")]
    Category,
    /// Error getting a game from the command line.
    #[error("no game provided")]
    Game,
    /// Error getting a run from the command line.
    #[error("no run provided")]
    Run,
}
