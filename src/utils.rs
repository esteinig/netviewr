use std::{fs::File, io::Write, path::PathBuf};
use needletail::{parse_fastx_file, parser::LineEnding};
use crate::error::NetviewError;


/// Write a FASTA record
pub fn write_fasta(
    id: &[u8],
    seq: &[u8],
    writer: &mut dyn Write,
    line_ending: LineEnding,
) -> Result<(), NetviewError> {
    let ending = line_ending.to_bytes();
    writer.write_all(b">")?;
    writer.write_all(id)?;
    writer.write_all(&ending)?;
    writer.write_all(seq)?;
    writer.write_all(&ending)?;
    Ok(())
}


/// Concatenates multiple Fasta files into a single file.
///
/// The function takes a base Fasta file and a list of Fasta files to append to the base file.
/// It writes the output to a new file specified by `output_path`.
///
/// # Arguments
///
/// * `base_file` - A `PathBuf` to the base Fasta file.
/// * `files_to_append` - A vector of `PathBuf` references to the Fasta files to append.
/// * `output_path` - A `PathBuf` to the output Fasta file.
///
/// # Returns
///
/// This function returns a `Result<(), ConcatError>`, which is `Ok` if the files were
/// successfully concatenated, or an `Err` with a `ConcatError` detailing what went wrong.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use netview::utils::concatenate_fasta_files;
///
/// let base_file = PathBuf::from("base.fasta");
/// let files_to_append = vec![PathBuf::from("append1.fasta"), PathBuf::from("append2.fasta")];
/// let output_path = PathBuf::from("output.fasta");
///
/// if let Err(e) = concatenate_fasta_files(base_file, &files_to_append, output_path) {
///     println!("An error occurred: {}", e);
/// }
/// ```
pub fn concatenate_fasta_files(base_file: &PathBuf, files_to_append: &Vec<PathBuf>, output_path: &PathBuf) -> Result<(), NetviewError> {
    let mut output_file = File::create(&output_path)?;

    // Append base file content to the output file.
    let base_content = std::fs::read(&base_file)?;
    output_file.write_all(&base_content)?;

    // Iterate over files to append and write their content to the output file.
    for file_path in files_to_append {
        let content = std::fs::read(file_path)?;
        output_file.write_all(&content)?;
    }

    Ok(())
}

pub fn get_ids_from_fasta_files(fasta: &Vec<PathBuf>) -> Result<Vec<String>, NetviewError> {
    
    let mut ids = Vec::new();

    for file in fasta {
        let mut reader = parse_fastx_file(&file)?;

        // Iterate through the FASTA file and store sequences by ID
        while let Some(record) = reader.next() {
            let record = record?;
            let id = std::str::from_utf8(record.id())?
                .split_whitespace()
                .collect::<Vec<_>>()[0]
                .to_string();
            ids.push(id)
        }
    }
    Ok(ids)
}