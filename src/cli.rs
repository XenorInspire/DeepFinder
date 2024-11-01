// External crates.
use clap::{Arg, ArgAction, Command};

/// This function is charged to build the command context for the CLI with the clap framework.
///
/// # Returns
///
/// Command containing the different features of WorgenX.
///
fn build_command_context() -> Command {
    Command::new("deepfinder")
        .arg_required_else_help(true)
        .disable_help_flag(true) // Keep the help handling in the run() function
        .disable_version_flag(true) // Keep the version handling in the run() function
        .disable_help_subcommand(true) // Keep the help handling in the run() function
        .arg(Arg::new("version").short('v').long("version").action(ArgAction::SetTrue))
        .arg(Arg::new("help").short('h').long("help").action(ArgAction::SetTrue))
}

/// This function is charged to display the help menu with all the features of DeepFinder and their options.
///
fn display_help() {
    println!("Usage: deepfinder <path> [options]");
    println!("Options:");
    println!("  -n, --name\t\t\t\tFind the duplicates by their name.\n\t\t\t\t\tSelected by default if both -n and -a arguments are not specified.");
    println!("  -a, --hash-algorithm\t\t\tFind the duplicates from the hash.\n\t\t\t\t\tYou can choose between: md5, sha1, sha224, sha256, sha384, sha512,\n\t\t\t\t\tsha3-224, sha3-256, sha3-384, sha3-512, blake2b-512, blake2s-256 and whirlpool");
    println!("  -f, --hidden-files\t\t\tEnable search for hidden files.");
    println!("  -c <path>, --csv-display <path>\tExport the results to stdin in a CSV format.");
    println!("  -C <path>, --csv-output <path>\tExport the results in a CSV fie.");
    println!("  -j <path>, --json-display <path>\tExport the results to stdin in a JSON format.");
    println!("  -J <path>, --json-output <path>\tExport the results in a JSON fie.");
    println!("  -x <path>, --xml-display <path>\tExport the results to stdin in a XML format.");
    println!("  -X <path>, --xml-output <path>\tExport the results in a XML fie.");
    println!("  -v, --version\t\t\t\tDisplay the version of DeepFinder");
    println!("  -h, --help\t\t\t\tDisplay this help message\n\n");
}

/// This function is charged to schedule the execution of the different features of the program according to the user's choices.
///
/// # Returns
///
/// Ok if the program has been executed, DeepFinderError otherwise.
///
pub fn run() -> Result<(), ()> {
    let mut command_context: Command = build_command_context();
    if let Ok(matches) = command_context.clone().try_get_matches() {
        // Call display_help() instead of clap help with the -h or --help arguments (better control of the help message)
        if matches.get_flag("help") {
            display_help();
            return Ok(());
        }
        // Call println!() instead of clap version with the -v or --version arguments (better control of the version message)
        if matches.get_flag("version") {
            println!("DeepFinder v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
    }

    command_context.build();
    Ok(())
}