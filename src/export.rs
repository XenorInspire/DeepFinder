// Internal crates.
use crate::{
    cli::{CliOutput, FindingConfig},
    error::{DeepFinderError, SystemError},
    search_engine::DuplicateFile,
};

// External crates.
use csv::WriterBuilder;
use serde::Serialize;
use std::{collections::{HashMap, HashSet}, fs};

/// This struct is used to serialize (except for CSV format) the DuplicateFile struct without checkums.
/// The "checksums" fields is None if `include_hashes` is false or if there isn't any checksum.
///
#[derive(Serialize)]
struct DuplicateFileSerialized<'a> {
    pub paths: &'a HashSet<String>,
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<&'a HashMap<String, String>>,
    pub size: u64,
}

/// This function is the scheduler for exporting findings.
///
/// # Arguments
///
/// * `duplicates` - Array of DuplicateFile containing the findings.
/// * `config` - The FindingConfig struct with the user's configuration.
///
/// # Returns
///
/// The result of the export findings scheduler, DeepFinderError otherwise.
///
pub fn export_findings_scheduler(duplicates: &[DuplicateFile], config: &FindingConfig) -> Result<(), DeepFinderError> {
    match &config.output {
        CliOutput::Standard => { simple_display(duplicates, config.include_hashes_in_output); Ok(()) },
        CliOutput::JsonStdin => json_display(duplicates, None, config.include_hashes_in_output),
        CliOutput::CsvStdin => csv_display(duplicates, None, config.include_hashes_in_output),
        CliOutput::XmlStdin => xml_display(duplicates, None, config.include_hashes_in_output),
        CliOutput::JsonFile(path) => json_display(duplicates, Some(path), config.include_hashes_in_output),
        CliOutput::CsvFile(path) => csv_display(duplicates, Some(path), config.include_hashes_in_output),
        CliOutput::XmlFile(path) => xml_display(duplicates, Some(path), config.include_hashes_in_output),
    }
}

/// This function displays the findings in a simple text format.
///
/// # Arguments
///
/// * `duplicates` - Array of DuplicateFile structs containing the findings.
///
fn simple_display(duplicates: &[DuplicateFile], include_hashes: bool) {
    if duplicates.is_empty() {
        println!("No duplicate files found.");
        return;
    }

    println!("{} duplicate files found:", duplicates.len());
    for duplicate in duplicates {
        println!("Duplicate file found: {}", duplicate.name);
        duplicate.paths.iter().for_each(|path| println!(" - {path}"));
        
        println!("Occurrences: {}", duplicate.paths.len());
        if include_hashes && let Some(checksums) = &duplicate.checksums {
            for c in checksums { println!("Checksum ({0}) : {1}", c.0, c.1); }
        }
        println!();
    }
}

/// This function displays the findings in JSON format.
///
/// # Arguments
///
/// * `duplicates` - Array of DuplicateFile structs containing the findings.
/// * `path` - An optional path to save the JSON output.
///
/// # Returns
///
/// Result<(), DeepFinderError> - Returns Ok if the display (and saving if a path was specified) is successful, DeepFinderError otherwise.
///
fn json_display(duplicates: &[DuplicateFile], path: Option<&str>, include_hashes: bool) -> Result<(), DeepFinderError> {
    let json_values: Vec<DuplicateFileSerialized> = duplicates.iter().map(|d| {
        DuplicateFileSerialized {
            paths: &d.paths,
            name: &d.name,
            checksums: if include_hashes { d.checksums.as_ref() } else { None },
            size: d.size,
        }
    }).collect();

    let json_data: String = serde_json::to_string(&json_values)
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
/// * `duplicates` - Array of DuplicateFile structs containing the findings.
/// * `path` - An optional path to save the CSV output.
///
/// # Returns
///
/// Result<(), DeepFinderError> - Returns Ok if the display (and saving if a path was specified) is successful, DeepFinderError otherwise.
///
fn csv_display(duplicates: &[DuplicateFile], path: Option<&str>, include_hashes: bool) -> Result<(), DeepFinderError> {
    let header: Vec<&str> = if include_hashes {
        ["Name", "Paths", "Occurrences", "Size", "Checksums"].to_vec()
    } else {
        ["Name", "Paths", "Occurrences", "Size"].to_vec()
    };

    let mut wtr: csv::Writer<Vec<u8>> = WriterBuilder::new().delimiter(b';').from_writer(vec![]);
    wtr.write_record(&header)
        .map_err(|e| DeepFinderError::SystemError(SystemError::UnableToSerialize("csv".to_string(), e.to_string())))?;

    for file in duplicates {
        if include_hashes {
            let checksums_str: String =  file.checksums.as_ref().map_or_else(
                || "N/A".to_string(),
                |checksums| {
                    checksums.iter()
                        .map(|(algo, checksum)| format!("{}:{}", algo, checksum))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            );
            
            wtr.write_record(
                [
                    &file.name,
                    &file.paths.iter().cloned().collect::<Vec<String>>().join("\n"),
                    &file.paths.len().to_string(),
                    &file.size.to_string(),
                    &checksums_str,
                ]
            ).map_err(|e| DeepFinderError::SystemError(SystemError::UnableToSerialize("csv".to_string(), e.to_string())))?;
        } else {
            wtr.write_record(
                [
                    &file.name,
                    &file.paths.iter().cloned().collect::<Vec<String>>().join("\n"),
                    &file.paths.len().to_string(),
                    &file.size.to_string(),
                ]
            ).map_err(|e| DeepFinderError::SystemError(SystemError::UnableToSerialize("csv".to_string(), e.to_string())))?;
        }
        
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
/// * `duplicates` - Array of DuplicateFile structs containing the findings.
/// * `path` - An optional path to save the XML output.
///
/// # Returns
///
/// Result<(), DeepFinderError> - Returns Ok if the display (and saving if a path was specified) is successful, DeepFinderError otherwise.
///
fn xml_display(duplicates: &[DuplicateFile], path: Option<&str>, include_hashes: bool) -> Result<(), DeepFinderError> {
    #[derive(Serialize)]
    #[serde(rename = "duplicate_files")]
    struct DuplicateFilesWrapper<'a> {
        #[serde(rename = "duplicate_file")]
        files: Vec<DuplicateFileSerialized<'a>>,
    }
    
    let xml_values: Vec<DuplicateFileSerialized> = duplicates.iter().map(|d| {
        DuplicateFileSerialized {
            paths: &d.paths,
            name: &d.name,
            checksums: if include_hashes { d.checksums.as_ref() } else { None },
            size: d.size,
        }
    }).collect();
    let wrapper: DuplicateFilesWrapper = DuplicateFilesWrapper { files: xml_values };
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
                size: 123,
                checksums: None,
            }
        ];
        assert!(json_display(&duplicates.clone(), None, true).is_ok());
    }

    #[test]
    fn test_json_display_to_file() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file2.txt".to_string(),
                paths: ["pathA".to_string(), "pathB".to_string()].into_iter().collect(),
                size: 456,
                checksums: None,
            }
        ];
        let test_path: &'static str = "test_output.json";
        assert!(json_display(&duplicates.clone(), Some(test_path), false).is_ok());

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
                size: 123,
                checksums: None,
            }
        ];
        assert!(csv_display(&duplicates.clone(), None, false).is_ok());
    }

    #[test]
    fn test_csv_display_to_file() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file2.txt".to_string(),
                paths: ["pathA".to_string(), "pathB".to_string()].into_iter().collect(),
                size: 456,
                checksums: None,
            }
        ];
        
        let test_path: &'static str = "test_output.csv";
        assert!(csv_display(&duplicates.clone(), Some(test_path), true).is_ok());
        
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
                size: 123,
                checksums: None,
            }
        ];
        assert!(xml_display(&duplicates.clone(), None, false).is_ok());
    }

    #[test]
    fn test_xml_display_to_file() {
        let duplicates: Vec<DuplicateFile> = vec![
            DuplicateFile {
                name: "file2.txt".to_string(),
                paths: ["pathA".to_string(), "pathB".to_string()].into_iter().collect(),
                size: 456,
                checksums: None,
            }
        ];
        
        let test_path: &'static str = "test_output.xml";
        assert!(xml_display(&duplicates.clone(), Some(test_path), true).is_ok());
        
        let content: String = fs::read_to_string(test_path).expect("File should exist");
        assert!(content.contains("file2.txt"));
        let _ = fs::remove_file(test_path);
    }
}