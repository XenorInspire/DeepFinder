// Internal crates.
use crate::{cli::{CliOutput, FindingConfig}, error::DeepFinderError, search_engine::DuplicateFile};

/// This function is the scheduler for exporting findings.
///
/// # Arguments
///
/// * `duplicates` - A vector of DuplicateFile containing the findings.
/// * `config` - The FindingConfig struct with the user's configuration.
///
/// # Returns
///
/// The result of the export findings scheduler, DeepFinderError otherwise.
///
pub fn export_findings_scheduler(duplicates: Vec<DuplicateFile>, config: &FindingConfig) -> Result<(), DeepFinderError> {
    match config.output {
        CliOutput::Standard => {
            simple_display(duplicates);
            Ok(())
        },
        // CliOutput::JsonStdin => json_display(duplicates, None),
        // CliOutput::CsvStdin => csv_display(duplicates, None),
        // CliOutput::XmlStdin => xml_display(duplicates, None),
        // CliOutput::JsonFile(path) => json_display(duplicates, Some(path)),
        // CliOutput::CsvFile(path) => csv_display(duplicates, Some(path)),
        // CliOutput::XmlFile(path) => xml_display(duplicates, Some(path)),
        _ => Ok(()),
    }
}

/// This function displays the findings in a simple text format.
///
/// # Arguments
///
/// * `duplicates` - A vector of DuplicateFile containing the findings.
///
/// # Returns
///
/// Result<(), DeepFinderError> - Returns Ok if the display is successful, DeepFinderError otherwise.
///
fn simple_display(duplicates: Vec<DuplicateFile>) {
    if duplicates.is_empty() {
        println!("No duplicate files found.");
        return;
    }

    println!("{} duplicate files found:", duplicates.len());
    for duplicate in duplicates {
        println!("Duplicate file found: {}", duplicate.name);
        for path in &duplicate.paths {
            println!(" - {path}");
        }
        println!("Occurrences: {}", duplicate.nb_occurrences);
        if let Some(checksums) = &duplicate.checksums {
            for (algo, checksum) in checksums {
                println!("Checksum ({algo}) : {checksum}");
            }
        }
        println!();
    }
}