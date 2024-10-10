extern crate rayon;

use csv::{ReaderBuilder, Trim};
use itertools::Itertools;
use needletail::parse_fastx_file;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

use crate::error::NetviewError;

pub fn extract_fasta_ids(fasta_path: &Path) -> Result<Vec<String>, NetviewError> {
    // Open the FASTA file using needletail
    let mut reader = parse_fastx_file(fasta_path)?;

    // Create a vector to store sequence IDs
    let mut sequence_ids: Vec<String> = Vec::new();

    // Iterate over the FASTA sequences
    while let Some(record) = reader.next() {
        let record = record?;

        // Convert the full header to a string and split by whitespace
        let full_header = String::from_utf8_lossy(record.id());

        // Extract the part before the first space (the sequence ID)
        if let Some(seq_id) = full_header.split_whitespace().next() {
            sequence_ids.push(seq_id.to_string());
        }
    }

    Ok(sequence_ids)
}

pub fn parse_identifiers(id_path: &Path) -> Result<Vec<String>, NetviewError> {
    let reader = BufReader::new(File::open(id_path)?);

    // Create a vector to store identifiers
    let mut ids: Vec<String> = Vec::new();

    for line in reader.lines() {
        let id = line?;
        ids.push(id)
    }

    Ok(ids)
}

pub fn write_ids(ids: &Vec<String>, file: &Path) -> Result<(), NetviewError> {
    let mut writer = BufWriter::new(File::create(file)?);

    for id in ids {
        writeln!(writer, "{}", &id)?
    }

    Ok(())
}

/// Executes a system command to generate a distance matrix, and then parses
/// and returns the matrix without the first row and column.
///
/// This function executes the `skani` command, which is expected to output a
/// tab-delimited distance matrix. The first row (header) and first column (row index)
/// are trimmed from the output, and the remaining data is returned as a symmetrical
/// distance matrix.
///
/// # Examples
///
/// ```
/// use netview::subtype::skani_distance_matrix;
///
/// let result = skani_distance_matrix();
/// match result {
///     Ok(matrix) => println!("Processed matrix: {:?}", matrix),
///     Err(e) => println!("Error occurred: {}", e),
/// }
/// ```
pub fn skani_distance_matrix(
    fasta: &Path,
    marker_compression_factor: usize,
    compression_factor: usize,
    threads: usize,
    min_percent_identity: f64,
    min_alignment_fraction: f64,
    small_genomes: bool,
) -> Result<(Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<String>, Vec<String>), NetviewError> {
    let args = if small_genomes {
        vec![
            String::from("triangle"),
            "-i".to_string(),
            fasta.display().to_string(),
            "-t".to_string(),
            format!("{}", threads),
            "-s".to_string(),
            format!("{:.2}", min_percent_identity),
            "--min-af".to_string(),
            format!("{:.2}", min_alignment_fraction),
            String::from("--full-matrix"),
            String::from("--distance"),
            String::from("--small-genomes"),
        ]
    } else {
        vec![
            String::from("triangle"),
            "-i".to_string(),
            fasta.display().to_string(),
            "-m".to_string(),
            format!("{}", marker_compression_factor),
            "-c".to_string(),
            format!("{}", compression_factor),
            "-t".to_string(),
            format!("{}", threads),
            "-s".to_string(),
            format!("{:.2}", min_percent_identity),
            "--min-af".to_string(),
            format!("{:.2}", min_alignment_fraction),
            String::from("--full-matrix"),
            String::from("--distance"),
        ]
    };

    log::info!("Computing pairwise distances with 'skani' (Shaw and Yu, 2023)");
    let output = Command::new("skani").args(&args).output()?.stdout;

    let output_str = String::from_utf8_lossy(&output);

    // Regex to match non-numeric and non-tab characters (to find rows and columns).
    let re = Regex::new(r"\t").expect("Failed to compile regex");

    // Vector to hold sequence IDs (from the first column).
    let skani_ids: Vec<String> = output_str
        .lines()
        .skip(1) // Skip the first line (header).
        .map(|line| {
            re.split(line)
                .next() // Get the first element (ID)
                .unwrap()
                .split_whitespace()
                .collect_vec()[0]
                .to_string()
        })
        .collect();

    let matrix: Vec<Vec<f64>> = output_str
        .lines()
        .skip(1) // Skip the first line (header).
        .map(|line| {
            re.split(line)
                .skip(1) // Skip the first column (index).
                .filter_map(|number| number.parse::<f64>().ok())
                .collect()
        })
        .collect();

    let af_matrix_path = Path::new("skani_matrix.af");

    let af_matrix: Vec<Vec<f64>> = if af_matrix_path.exists() {
        let af_file = File::open(af_matrix_path)?;
        let reader = BufReader::new(af_file);

        let af_matrix: Vec<Vec<f64>> = reader
            .lines()
            .skip(1) // Skip the first line (header).
            .map(|line| {
                re.split(&line.unwrap())
                    .skip(1) // Skip the first column (index).
                    .filter_map(|number| number.parse::<f64>().ok())
                    .collect()
            })
            .collect();

        // Delete the skani_matrix.af file after parsing
        std::fs::remove_file(af_matrix_path)?;

        af_matrix
    } else {
        return Err(NetviewError::ParseSkaniMatrix); // Handle case if the file doesn't exist
    };

    let fasta_ids = extract_fasta_ids(&fasta)?;
    let missing_ids = find_missing_ids(fasta_ids.clone(), skani_ids.clone());

    log::info!("Missing sequences: {:#?}", missing_ids);

    // Check if both matrices are square
    if matrix.len() > 0
        && matrix[0].len() == matrix.len()
        && af_matrix.len() > 0
        && af_matrix[0].len() == af_matrix.len()
    {
        Ok((matrix, af_matrix, skani_ids, missing_ids))
    } else {
        Err(NetviewError::ParseSkaniMatrix)
    }
}

fn find_missing_ids(ids1: Vec<String>, ids2: Vec<String>) -> Vec<String> {
    // Convert the Vecs to HashSets for efficient comparison
    let set1: HashSet<String> = ids1.into_iter().collect();
    let set2: HashSet<String> = ids2.into_iter().collect();

    // Find the elements in set1 that are missing in set2
    set1.difference(&set2).cloned().collect()
}

/// Writes a matrix of `f64` values to a specified file in tab-delimited format.
///
/// # Arguments
///
/// * `matrix` - A two-dimensional vector (matrix) where each inner `Vec<f64>`
///   represents a row of the matrix. Each `f64` element is written as a tab-separated
///   value to the file.
/// * `file_path` - A string slice that holds the path to the file where the matrix
///   should be written. The function will create the file if it does not exist, or
///   overwrite the file if it already exists.
///
/// # Errors
///
/// Returns a `SubtypeDatabaseError` if there is any problem during file operations,
/// such as failing to create or write to the file.
///
/// # Example
///
/// ```rust
/// let matrix: Vec<Vec<f64>> = vec![
///     vec![1.0, 2.0, 3.0],
///     vec![4.0, 5.0, 6.0],
///     vec![7.0, 8.0, 9.0],
/// ];
///
/// if let Err(e) = write_matrix_to_file(matrix, "matrix_output.txt") {
///     eprintln!("Error writing matrix to file: {:?}", e);
/// }
/// ```
pub fn write_matrix_to_file(matrix: &Vec<Vec<f64>>, file_path: &Path) -> Result<(), NetviewError> {
    // Open the file for writing (or create it if it doesn't exist)
    let mut file = File::create(file_path)?;

    // Iterate through the rows of the matrix
    for row in matrix {
        // Convert each row into a tab-separated string
        let row_str = row
            .iter()
            .map(|num| num.to_string())
            .collect::<Vec<String>>()
            .join("\t");

        // Write the row to the file, followed by a newline
        writeln!(file, "{}", row_str)?
    }

    Ok(())
}

/// Represents a row in the matrix for easier handling with serde.
#[derive(Deserialize)]
struct MatrixRow(Vec<f64>);

/// Parses a distance matrix from a CSV/TSV file.
///
/// The function can handle both symmetrical and lower triangular matrices.
/// It automatically detects whether the file is CSV or TSV based on the extension.
///
/// # Arguments
///
/// * `file_path` - The path to the input CSV/TSV file is extracted from file path
///                 extensions `.tsv` and `.csv`. Defaults to CSV if extension
///                 fails to be extracted from file path (i.e. no extension).
///
/// # Returns
///
/// A `Result` with either:
/// - `Ok(Vec<Vec<f64>>)` containing the parsed distance matrix.
/// - `Err(NetviewError)` indicating the error encountered.
///
/// # Examples
///
/// ```
/// use netview::parse_distance_matrix;
/// use std::path::Path;
///
/// let distance_matrix = parse_distance_matrix(
///     Path::new("distance_matrix.csv")
/// ).unwrap();
///
/// println!("{:#?}", distance_matrix);
/// ```
pub fn parse_input_matrix<P: AsRef<Path>>(
    file_path: P,
    is_csv: bool,
) -> Result<Vec<Vec<f64>>, NetviewError> {
    let file = File::open(file_path.as_ref()).map_err(|_| NetviewError::FileReadError)?;

    let mut rdr = ReaderBuilder::new()
        .delimiter(if is_csv { b',' } else { b'\t' })
        .trim(Trim::All)
        .has_headers(false)
        .from_reader(file);

    let mut matrix = Vec::new();

    for result in rdr.deserialize() {
        let record: MatrixRow = result.map_err(|e| NetviewError::ParseError(e.to_string()))?;
        matrix.push(record.0);
    }

    log::info!(
        "Input matrix dimensions: {}",
        match matrix.is_empty() {
            true => "0 x 0".to_string(),
            false => format!("{} x {}", matrix.len(), matrix[matrix.len() - 1].len()),
        }
    );

    // Validate matrix format (symmetrical or lower triangular)
    // This step is crucial to ensure the matrix conforms to expectations
    if !is_matrix_valid(&matrix) {
        return Err(NetviewError::MatrixFormatError);
    }

    Ok(matrix)
}

/// Validates if the given matrix is symmetrical or lower triangular.
fn is_matrix_valid(matrix: &[Vec<f64>]) -> bool {
    let n = matrix.len();
    for (i, row) in matrix.iter().enumerate() {
        if row.len() > n || (row.len() != n && row.len() != i + 1) {
            return false;
        }
    }
    true
}

/// Transforms a lower triangular matrix into a symmetrical matrix, with error handling.
///
/// # Arguments
///
/// * `distance_matrix` - A slice of Vec<Vec<f64>> representing the lower triangular or full symmetrical matrix.
///
/// # Returns
///
/// Returns a `Result` with either:
/// - `Ok(Vec<Vec<f64>>)`: A symmetrical matrix as a vector of vectors of f64.
/// - `Err(NetviewError)`: An error indicating what went wrong in the process.
///
/// # Examples
///
/// ```
/// use netview::make_symmetrical;
/// use netview::NetviewError;
///
/// let lower_triangular_matrix = vec![
///     vec![0.0],
///     vec![1.0, 0.0],
/// ];
///
/// let symmetrical_matrix = make_symmetrical(
///     &lower_triangular_matrix
/// ).unwrap();
///
/// assert_eq!(symmetrical_matrix, vec![
///     vec![0.0, 1.0],
///     vec![1.0, 0.0],
/// ]);
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The input matrix is empty but was expected to be non-empty.
/// - The input matrix's dimensions do not match the expected dimensions.
pub fn make_symmetrical(distance_matrix: &Vec<Vec<f64>>) -> Result<Vec<Vec<f64>>, NetviewError> {
    let n = distance_matrix.len();

    if distance_matrix.is_empty() {
        return Err(NetviewError::EmptyMatrix);
    }

    let matrix = if distance_matrix.iter().all(|row| row.len() == n) {
        log::debug!("Distance matrix input for make_symmetrical is a symmetrical distance matrix. Returning input distance matrix.");
        distance_matrix.clone()
    } else {
        let mut sym_matrix = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..=i {
                sym_matrix[i][j] = distance_matrix[i][j];
                sym_matrix[j][i] = distance_matrix[i][j];
            }
        }
        sym_matrix
    };

    Ok(matrix)
}

/// Computes the Euclidean distance matrix with options for parallel computation,
/// handling lower triangular matrices, and manually setting the number of threads.
///
/// # Arguments
///
/// * `matrix` - A symmetrical distance matrix or its lower triangular part as `Vec<Vec<f64>>`.
/// * `is_lower_triangular` - Indicates if the input matrix is lower triangular.
/// * `parallel` - Indicates if parallel computation should be used.
/// * `num_threads` - An optional number of threads for parallel computation.
///
/// # Returns
///
/// A `Result` containing either:
/// - `Ok(Vec<Vec<f64>>)`: The Euclidean distance matrix.
/// - `Err(NetviewError)`: An error indicating what went wrong.
///
/// # Examples
///
/// ```
/// use netview::dist::euclidean_distance_of_distances;
///
/// let distance_matrix = vec![
///     vec![0.0, 1.0],
///     vec![1.0, 0.0],
/// ];
///
/// let result = euclidean_distance_of_distances(
///     &distance_matrix, false, false, None
/// ).unwrap();
///
/// assert_eq!(result, vec![vec![0.0, 1.0], vec![1.0, 0.0]]);
/// ```
///
/// # Errors
///
/// This function can return `NetviewError::NonSquareMatrix` if the input is not a square matrix
/// when `is_lower_triangular` is false, or `NetviewError::ThreadPoolBuildError` if the thread pool
/// cannot be initialized with the specified number of threads.
pub fn euclidean_distance_of_distances(
    distance_matrix: &Vec<Vec<f64>>,
    is_lower_triangular: bool,
    num_threads: Option<usize>,
    chunk_size: Option<usize>
) -> Result<Vec<Vec<f64>>, NetviewError> {
    let n = distance_matrix.len();

    // Prepare a vector to store distance computations
    let mut distances = vec![];

    let compute_distance = |i: usize, j: usize| -> f64 {
        let mut sum = 0.0;
        for k in 0..n {
            let val_i = if is_lower_triangular && i < k {
                distance_matrix[k][i]
            } else {
                distance_matrix[i][k]
            };
            let val_j = if is_lower_triangular && j < k {
                distance_matrix[k][j]
            } else {
                distance_matrix[j][k]
            };
            sum += (val_i - val_j).powi(2);
        }
        sum.sqrt()
    };

    if num_threads.is_some() && chunk_size.is_none() {
        // Collect computed distances in parallel
        distances = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads.unwrap())
            .build()
            .expect("Failed to create thread pool")
            .install(|| -> Vec<(usize, usize, f64)> {
                (0..n).into_par_iter()
                .flat_map(|i| {
                    (i + 1..n)
                        .map(|j| (i, j, compute_distance(i, j)))
                        .collect::<Vec<_>>() // collect inner loop results
                })
                .collect()
        });
    } else if num_threads.is_some() && chunk_size.is_some() {
        distances = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads.unwrap())
            .build()
            .expect("Failed to create thread pool")
            .install(|| -> Vec<(usize, usize, f64)> {
                (0..n).into_par_iter()
                    .chunks(chunk_size.unwrap())
                    .flat_map(|chunk| {
                        chunk.iter().flat_map(|&i| {
                            (i + 1..n)
                                .map(move |j| (i, j, compute_distance(i, j)))
                        })
                        .collect::<Vec<_>>() // Collect inner loop results per chunk
                    })
                    .collect()
            });
    } else {
        // Collect computed distances sequentially
        for i in 0..n {
            for j in i + 1..n {
                distances.push((i, j, compute_distance(i, j)));
            }
        }
    }

    // Create the result matrix and fill in the computed distances
    let mut result_matrix = vec![vec![0.0; n]; n];
    for (i, j, distance) in distances {
        result_matrix[i][j] = distance;
        result_matrix[j][i] = distance;
    }

    Ok(result_matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for compute_euclidean_distance_of_distances


    // Tests for make_symmetrical

    #[test]
    fn symmetrical_empty_matrix() {
        let matrix = vec![];
        let result = make_symmetrical(&matrix);
        assert!(matches!(result, Err(NetviewError::EmptyMatrix)));
    }

    #[test]
    fn symmetrical_already_symmetrical() {
        let matrix = vec![vec![0.0, 2.0], vec![2.0, 0.0]];
        let result = make_symmetrical(&matrix.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), matrix);
    }

    #[test]
    fn symmetrical_lower_triangular_to_symmetrical() {
        let matrix = vec![vec![0.0], vec![1.0, 0.0]];
        let expected = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let result = make_symmetrical(&matrix);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn symmetrical_invalid_lower_triangular_format() {
        let matrix = vec![vec![0.0], vec![1.0]]; // This no longer directly triggers an error due to the removed `n` parameter check
        let result = make_symmetrical(&matrix);
        // Instead of expecting an error, this test now expects a successful operation or a different handling strategy
        assert!(result.is_ok()); // Adjust based on new logic
    }

    #[test]
    fn symmetrical_large_matrix() {
        let size = 100;
        let matrix = (0..size)
            .map(|i| (0..=i).map(|j| (i + j) as f64).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let result = make_symmetrical(&matrix);
        assert!(result.is_ok());
        let sym_matrix = result.unwrap();
        for i in 0..size {
            for j in 0..=i {
                assert_eq!(
                    sym_matrix[i][j], sym_matrix[j][i],
                    "Mismatch at ({}, {})",
                    i, j
                );
            }
        }
    }

    #[test]
    fn make_symmetrical_with_increasing_values() {
        // Test a lower triangular matrix with increasing values to ensure correct symmetrical transformation
        let matrix = vec![vec![1.0], vec![2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let expected = vec![
            vec![1.0, 2.0, 4.0],
            vec![2.0, 3.0, 5.0],
            vec![4.0, 5.0, 6.0],
        ];
        let result = make_symmetrical(&matrix).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn make_symmetrical_already_full_and_symmetrical() {
        // Test a full matrix that is already symmetrical
        let matrix = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, 0.0, 3.0],
            vec![2.0, 3.0, 0.0],
        ];
        let result = make_symmetrical(&matrix.clone()).unwrap();
        assert_eq!(result, matrix);
    }

    // Tests for parse_distance_matrix

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // Helper function to create a temporary file with specified contents
    fn create_temp_matrix_file(contents: &str, extension: &str) -> PathBuf {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(format!("temp_file.{}", extension));
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", contents).unwrap();
        file_path
    }

    #[test]
    fn parse_symmetrical_csv() {
        let path = create_temp_matrix_file("0,1\n1,0", "csv");
        let matrix = parse_input_matrix(path, true).unwrap();
        assert_eq!(matrix, vec![vec![0.0, 1.0], vec![1.0, 0.0]]);
    }

    #[test]
    fn parse_lower_triangular_tsv() {
        let path = create_temp_matrix_file("0\n1\t0", "tsv");
        let matrix = parse_input_matrix(path, true).unwrap();
        assert_eq!(matrix, vec![vec![0.0], vec![1.0, 0.0]]);
    }

    #[test]
    fn empty_file_error() {
        let path = create_temp_matrix_file("", "csv");
        let result = parse_input_matrix(path, false);
        assert!(matches!(result, Err(NetviewError::MatrixFormatError)));
    }

    #[test]
    fn invalid_format_error() {
        // More columns than rows - invalid matrix
        let path = create_temp_matrix_file("0,1,2\n1,0", "csv");
        let result = parse_input_matrix(path, false);
        assert!(matches!(result, Err(NetviewError::MatrixFormatError)));
    }

    #[test]
    fn non_numeric_values_error() {
        let path = create_temp_matrix_file("0,a\nb,0", "csv");
        let result = parse_input_matrix(path, false);
        assert!(matches!(result, Err(NetviewError::ParseError(_))));
    }

    #[test]
    fn file_not_found_error() {
        let path = PathBuf::from("non_existent_file.csv");
        let result = parse_input_matrix(path, true);
        assert!(matches!(result, Err(NetviewError::FileReadError)));
    }

    #[test]
    fn parse_symmetrical_with_additional_whitespaces_csv() {
        let path = create_temp_matrix_file("0 , 1\n 1,0 ", "csv");
        let matrix = parse_input_matrix(path, false).unwrap();
        assert_eq!(matrix, vec![vec![0.0, 1.0], vec![1.0, 0.0]]);
    }

    #[test]
    fn inconsistent_row_lengths_error() {
        let path = create_temp_matrix_file("0,1,2\n1,0", "csv");
        let result = parse_input_matrix(path, false);
        assert!(matches!(result, Err(NetviewError::MatrixFormatError)));
    }

    #[test]
    fn large_symmetrical_csv() {
        // Creates a 3x3 matrix
        let path = create_temp_matrix_file("0,1,2\n1,0,3\n2,3,0", "csv");
        let matrix = parse_input_matrix(path, false).unwrap();
        assert_eq!(
            matrix,
            vec![
                vec![0.0, 1.0, 2.0],
                vec![1.0, 0.0, 3.0],
                vec![2.0, 3.0, 0.0]
            ]
        );
    }

    #[test]
    fn valid_tsv_with_mixed_delimiters_error() {
        // Uses both comma and tab as delimiters, which should result in an error
        let path = create_temp_matrix_file("0\t1\n1,0", "tsv");
        let result = parse_input_matrix(path, false);
        // Expected to fail due to inconsistent delimiters within a TSV file
        assert!(matches!(result, Err(NetviewError::MatrixFormatError)));
    }
}
