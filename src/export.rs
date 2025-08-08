// Internal crates.
use crate::{
    cli::{CliOutput, FindingConfig},
    error::{DeepFinderError, SystemError},
    search_engine::DuplicateFile,
};

// External crates.
use csv::WriterBuilder;
use serde::Serialize;
use std::fs;

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
    match &config.output {
        CliOutput::Standard => { simple_display(duplicates); Ok(()) },
        CliOutput::JsonStdin => json_display(duplicates, None),
        CliOutput::CsvStdin => csv_display(duplicates, None),
        CliOutput::XmlStdin => xml_display(duplicates, None),
        CliOutput::JsonFile(path) => json_display(duplicates, Some(path)),
        CliOutput::CsvFile(path) => csv_display(duplicates, Some(path)),
        CliOutput::XmlFile(path) => xml_display(duplicates, Some(path)),
    }
}

/// This function displays the findings in a simple text format.
///
/// # Arguments
///
/// * `duplicates` - A vector of DuplicateFile containing the findings.
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

/// This function displays the findings in JSON format.
///
/// # Arguments
///
/// * `duplicates` - A vector of DuplicateFile containing the findings.
/// * `path` - An optional path to save the JSON output.
///
/// # Returns
///
/// Result<(), DeepFinderError> - Returns Ok if the display (and saving if a path was specified) is successful, DeepFinderError otherwise.
///
fn json_display(duplicates: Vec<DuplicateFile>, path: Option<&str>) -> Result<(), DeepFinderError> {
    let json_data: String = serde_json::to_string(&duplicates)
        .map_err(|e| DeepFinderError::SystemError(SystemError::UnableToSerialize("json".to_string(), e.to_string())))?;
    
    if let Some(file_path) = path {
        fs::write(file_path, json_data)
            .map_err(|e| DeepFinderError::SystemError(SystemError::UnableToCreateFile(file_path.to_string(), e.to_string())))?;
    } else {
        println!("{}", json_data);
    }

    Ok(())
}

/// This function displays the findings in CSV format.
///
/// # Arguments
///
/// * `duplicates` - A vector of DuplicateFile containing the findings.
/// * `path` - An optional path to save the CSV output.
///
/// # Returns
///
/// Result<(), DeepFinderError> - Returns Ok if the display (and saving if a path was specified) is successful, DeepFinderError otherwise.
///
fn csv_display(duplicates: Vec<DuplicateFile>, path: Option<&str>) -> Result<(), DeepFinderError> {
    let mut wtr: csv::Writer<Vec<u8>> = WriterBuilder::new().delimiter(b';').from_writer(vec![]);
    wtr.write_record(["Name", "Paths", "Occurrences", "Size", "Checksums"])
        .map_err(|e| DeepFinderError::SystemError(SystemError::UnableToSerialize("csv".to_string(), e.to_string())))?;
    for file in duplicates {
        wtr.write_record(&[
            file.name,
            file.paths.iter().cloned().collect::<Vec<String>>().join("\n"),
            file.nb_occurrences.to_string(),
            file.size.to_string(),
            file.checksums.as_ref().map_or("N/A".to_string(), |checksums| {
                checksums.iter()
                    .map(|(algo, checksum)| format!("{algo}:{checksum}"))
                    .collect::<Vec<String>>()
                    .join("\n")
            }),
        ]).map_err(|e| DeepFinderError::SystemError(SystemError::UnableToSerialize("csv".to_string(), e.to_string())))?;
    }
    
    let csv_data: String = String::from_utf8(wtr.into_inner().unwrap_or_default())
        .map_err(|e| DeepFinderError::SystemError(SystemError::UnableToSerialize("csv".to_string(), e.to_string())))?;

    if let Some(file_path) = path {
        fs::write(file_path, csv_data)
            .map_err(|e| DeepFinderError::SystemError(SystemError::UnableToCreateFile(file_path.to_string(), e.to_string())))?;
    } else {
        println!("{}", csv_data);
    }

    Ok(())
}
/// This function displays the findings in XML format.
///
/// # Arguments
///
/// * `duplicates` - A vector of DuplicateFile containing the findings.
/// * `path` - An optional path to save the XML output.
///
/// # Returns
///
/// Result<(), DeepFinderError> - Returns Ok if the display (and saving if a path was specified) is successful, DeepFinderError otherwise.
///
fn xml_display(duplicates: Vec<DuplicateFile>, path: Option<&str>) -> Result<(), DeepFinderError> {
    #[derive(Serialize)]
    #[serde(rename = "duplicate_files")]
    struct DuplicateFilesWrapper {
        #[serde(rename = "duplicate_file")]
        files: Vec<DuplicateFile>,
    }

    let wrapper: DuplicateFilesWrapper = DuplicateFilesWrapper { files: duplicates };
    let xml_data: String = serde_xml_rs::to_string(&wrapper)
        .map_err(|e| DeepFinderError::SystemError(SystemError::UnableToSerialize("xml".to_string(), e.to_string())))?;
    
    if let Some(file_path) = path {
        fs::write(file_path, xml_data)
            .map_err(|e| DeepFinderError::SystemError(SystemError::UnableToCreateFile(file_path.to_string(), e.to_string())))?;
    } else {
        println!("{}", xml_data);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_json_display_to_stdout() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file1.txt".to_string(),
                paths: ["path1".to_string(), "path2".to_string()].into_iter().collect(),
                nb_occurrences: 2,
                size: 123,
                checksums: None,
            }
        ];
        assert!(json_display(duplicates.clone(), None).is_ok());
    }

    #[test]
    fn test_json_display_to_file() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file2.txt".to_string(),
                paths: ["pathA".to_string(), "pathB".to_string()].into_iter().collect(),
                nb_occurrences: 2,
                size: 456,
                checksums: None,
            }
        ];
        let test_path: &'static str = "test_output.json";
        assert!(json_display(duplicates.clone(), Some(test_path)).is_ok());

        let content: String = fs::read_to_string(test_path).expect("File should exist");
        assert!(content.contains("file2.txt"));
        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_csv_display_to_stdout() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file1.txt".to_string(),
                paths: ["path1".to_string(), "path2".to_string()].into_iter().collect(),
                nb_occurrences: 2,
                size: 123,
                checksums: None,
            }
        ];
        assert!(csv_display(duplicates.clone(), None).is_ok());
    }

    #[test]
    fn test_csv_display_to_file() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file2.txt".to_string(),
                paths: ["pathA".to_string(), "pathB".to_string()].into_iter().collect(),
                nb_occurrences: 2,
                size: 456,
                checksums: None,
            }
        ];
        
        let test_path: &'static str = "test_output.csv";
        assert!(csv_display(duplicates.clone(), Some(test_path)).is_ok());
        
        let content: String = fs::read_to_string(test_path).expect("File should exist");
        assert!(content.contains("file2.txt"));
        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_xml_display_to_stdout() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file1.txt".to_string(),
                paths: ["path1".to_string(), "path2".to_string()].into_iter().collect(),
                nb_occurrences: 2,
                size: 123,
                checksums: None,
            }
        ];
        assert!(xml_display(duplicates.clone(), None).is_ok());
    }

    #[test]
    fn test_xml_display_to_file() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file2.txt".to_string(),
                paths: ["pathA".to_string(), "pathB".to_string()].into_iter().collect(),
                nb_occurrences: 2,
                size: 456,
                checksums: None,
            }
        ];
        
        let test_path: &'static str = "test_output.xml";
        assert!(xml_display(duplicates.clone(), Some(test_path)).is_ok());
        
        let content: String = fs::read_to_string(test_path).expect("File should exist");
        assert!(content.contains("file2.txt"));
        let _ = fs::remove_file(test_path);
    }
}