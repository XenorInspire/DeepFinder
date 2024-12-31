// Internal crates.
use crate::{
    error::{ArgError, DeepFinderError},
    system,
};

// External crates.
use clap::{Arg, ArgAction, ArgMatches, Command};

/// This struct is built from the values/choices of the user.
///
#[derive(PartialEq, Debug)]
struct FindingConfig<> {
    pub path: String,
    pub enable_search_by_name: bool,
    pub include_hidden_files: bool,
    pub hash: Vec<String>,
    pub output: CliOutput,
}

/// This enum is used to determine the output format of the program.
/// If no output format is specified, the program will display the results in STDIN, in a raw format.
/// 
#[derive(PartialEq, Debug)]
enum CliOutput {
    Standard,
    CsvStdin,
    CsvFile(String),
    JsonStdin,
    JsonFile(String),
    XmlStdin,
    XmlFile(String),
}

/// This function is responsible for building the command context for the CLI with the clap framework.
///
/// # Returns
///
/// Command containing the different features of DeepFinder.
///
fn build_command_context() -> Command {
    Command::new("deepfinder")
        .arg_required_else_help(true)
        .disable_help_flag(true) // Keep the help handling in the run() function
        .disable_version_flag(true) // Keep the version handling in the run() function
        .arg(
            Arg::new("path")
                .index(1)
                .value_name("path")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("The path to the directory to search for duplicates"),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .help("Allow duplicate finding by the filename")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("hash_algorithm")
                .short('a')
                .long("hash-algorithm")
                .value_delimiter(',')
                .value_parser(vec![
                    "md5",
                    "sha1",
                    "sha224",
                    "sha256",
                    "sha384",
                    "sha512",
                    "sha3-224",
                    "sha3-256",
                    "sha3-384",
                    "sha3-512",
                    "blake2b",
                    "blake2s",
                    "whirlpool",
                ])
                .help("Allow duplicate finding by one or multiple hash algorithms")
                .value_name("hash"),
        )
        .arg(
            Arg::new("hidden_files")
                .short('f')
                .long("hidden-files")
                .help("Allow duplicate finding for hidden files")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("csv_display")
                .short('c')
                .long("csv-display")
                .help("Export the results to stdin in CSV format")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "xml_display",
                    "xml_output",
                    "json_display",
                    "json_output",
                    "csv_output",
                ]),
        )
        .arg(
            Arg::new("csv_output")
                .short('C')
                .long("csv-output")
                .help("Export the results in a CSV file")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .value_name("path")
                .conflicts_with_all([
                    "xml_display",
                    "xml_output",
                    "json_display",
                    "json_output",
                    "csv_display",
                ]),
        )
        .arg(
            Arg::new("json_display")
                .short('j')
                .long("json-display")
                .help("Export the results to stdin in JSON format")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "xml_display",
                    "xml_output",
                    "json_output",
                    "csv_display",
                    "csv_output",
                ]),
        )
        .arg(
            Arg::new("json_output")
                .short('J')
                .long("json-output")
                .help("Export the results in a JSON file")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .value_name("path")
                .conflicts_with_all([
                    "xml_display",
                    "xml_output",
                    "json_display",
                    "csv_display",
                    "csv_output",
                ]),
        )
        .arg(
            Arg::new("xml_display")
                .short('x')
                .long("xml-display")
                .help("Export the results to stdin in XML format")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([
                    "xml_output",
                    "json_display",
                    "json_output",
                    "csv_display",
                    "csv_output",
                ]),
        )
        .arg(
            Arg::new("xml_output")
                .short('X')
                .long("xml-output")
                .help("Export the results in a XML file")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .value_name("path")
                .conflicts_with_all([
                    "xml_display",
                    "json_display",
                    "json_output",
                    "csv_display",
                    "csv_output",
                ]),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::SetTrue),
        )
}

/// This function is resonsible for displaying the help menu with all the features of DeepFinder and their options.
///
fn display_help() {
    println!("Usage: deepfinder <path> [options]");
    println!("Options:");
    println!("  -n, --name\t\t\t\tFind the duplicates by their name.\n\t\t\t\t\tSelected by default if both -n and -a arguments are not specified.");
    println!("  -a, --hash-algorithm\t\t\tFind the duplicates from the hash.\n\t\t\t\t\tYou can choose between: md5, sha1, sha224, sha256, sha384, sha512,\n\t\t\t\t\tsha3-224, sha3-256, sha3-384, sha3-512, blake2b-512, blake2s-256 and whirlpool.");
    println!("  -f, --hidden-files\t\t\tEnable search for hidden files.");
    println!("  -c <path>, --csv-display\t\tExport the results to stdin in a CSV format.");
    println!("  -C <path>, --csv-output <path>\tExport the results in a CSV file.");
    println!("  -j <path>, --json-display\t\tExport the results to stdin in a JSON format.");
    println!("  -J <path>, --json-output <path>\tExport the results in a JSON file.");
    println!("  -x <path>, --xml-display\t\tExport the results to stdin in a XML format.");
    println!("  -X <path>, --xml-output <path>\tExport the results in a XML file.");
    println!("  -v, --version\t\t\t\tDisplay the version of DeepFinder.");
    println!("  -h, --help\t\t\t\tDisplay this help message.\n\n");
}

/// This function is responsible for scheduling the execution of the different features of the program according to the user's choices.
///
/// # Returns
///
/// Ok if the program has been executed, DeepFinderError otherwise.
///
pub fn run() -> Result<(), DeepFinderError> {
    let command_context: Command = build_command_context();
    let matches: ArgMatches = command_context.try_get_matches().map_err(|_| DeepFinderError::ArgError(ArgError::NoPathSpecified))?;

    // Call display_help() instead of clap help with the -h or --help arguments (better control of the help message).
    if matches.get_flag("help") {
        display_help();
        return Ok(());
    }
    // Call println!() instead of clap version with the -v or --version arguments (better control of the version message).
    if matches.get_flag("version") {
        println!("DeepFinder v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    parse_user_choices(matches)?;
    Ok(())
}

/// This function is responsible for parsing the user's choices and building the FindingConfig struct.
///
/// # Arguments
///
/// * `matches` - The ArgMatches struct containing the user's choices.
///
/// # Returns
///
/// Ok(FindingConfig) if the user's choices are valid, DeepFinderError otherwise.
///
fn parse_user_choices(matches: ArgMatches) -> Result<FindingConfig, DeepFinderError> {
    let path: String = matches
        .get_one::<String>("path")
        .ok_or(DeepFinderError::ArgError(ArgError::NoPathSpecified))
        .and_then(|path| system::is_valid_folder_path(path).map_err(DeepFinderError::SystemError))?;

    let mut config: FindingConfig = FindingConfig {
        path,
        enable_search_by_name: matches.get_flag("name"),
        include_hidden_files: matches.get_flag("hidden_files"),
        hash: matches
            .get_many::<String>("hash_algorithm")
            .unwrap_or_default()
            .cloned()
            .collect(),
        output: CliOutput::Standard,
    };

    config.output = match (
        matches.get_flag("csv_display"),
        matches.get_one::<String>("csv_output"),
        matches.get_flag("json_display"),
        matches.get_one::<String>("json_output"),
        matches.get_flag("xml_display"),
        matches.get_one::<String>("xml_output"),
    ) {
        (true, _, _, _, _, _) => CliOutput::CsvStdin,
        (_, Some(path), _, _, _, _) => {
            let temp_path: String = check_output_arg(path)?;
            CliOutput::CsvFile(temp_path.to_string())
        }
        (_, _, true, _, _, _) => CliOutput::JsonStdin,
        (_, _, _, Some(path), _, _) => {
            let temp_path: String = check_output_arg(path)?;
            CliOutput::JsonFile(temp_path.to_string())
        }
        (_, _, _, _, true, _) => CliOutput::XmlStdin,
        (_, _, _, _, _, Some(path)) => {
            let temp_path: String = check_output_arg(path)?;
            CliOutput::XmlFile(temp_path.to_string())
        }
        _ => CliOutput::Standard,
    };

    Ok(config)
}

/// This function is responsible for checking the path for the 'output' arguments, if it's a valid path on the filesystem.
///
/// # Arguments
///
/// * `path` - The path to check.
///
/// # Returns
///
/// Ok(String) if the path is valid, DeepFinderError otherwise.
///
fn check_output_arg(path: &str) -> Result<String, DeepFinderError> {
    match system::is_valid_file_path(path) {
        Ok(full_path) => Ok(full_path),
        Err(e) => Err(DeepFinderError::SystemError(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user_choices() {
        let command_context: Command = build_command_context();
        let matches: ArgMatches = command_context.clone().get_matches_from(vec!["deepfinder", "/tmp", "-a", "md5,sha256", "-f", "-n", "-C", "/tmp/output.csv"]);
        let expected: FindingConfig = FindingConfig {
            path: "/tmp".to_string(),
            enable_search_by_name: true,
            include_hidden_files: true,
            hash: vec!["md5".to_string(), "sha256".to_string()],
            output: CliOutput::CsvFile("/tmp/output.csv".to_string()),
        };
        assert_eq!(parse_user_choices(matches).unwrap(), expected);

        let matches_error1: ArgMatches = command_context.get_matches_from(vec!["deepfinder", "-a", "md5,sha256", "-f", "-n", "-J", "./output.json"]); // Missing searching path.
        assert_eq!(parse_user_choices(matches_error1).unwrap_err(), DeepFinderError::ArgError(ArgError::NoPathSpecified));
    }
}