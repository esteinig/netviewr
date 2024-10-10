
#[cfg(feature = "plot")]
use plotters::prelude::DrawingAreaErrorKind;
#[cfg(feature = "plot")]
use plotters_bitmap::BitMapBackendError;

use thiserror::Error;

/// An error that can occur in k-nearest neighbor calculations.
#[derive(Error, Debug)]
pub enum NetviewError {
    #[error("The distance matrix is invalid or empty.")]
    InvalidMatrix,
    #[error("The value of k is out of valid range.")]
    InvalidK,
    #[error("Input matrix is empty, but a non-empty matrix was expected.")]
    EmptyMatrix,
    #[error("Input matrix dimensions are incorrect or inconsistent for the expected operation.")]
    InvalidDimensions,
    #[error("Input matrix is not in a proper lower triangular format.")]
    InvalidLowerTriangularFormat,
    #[error("The matrix must be square for non-lower triangular matrices.")]
    NonSquareMatrix,
    #[error("Error setting up thread pool")]
    ThreadPoolBuildError,
    #[error("Matrix dimensions are inconsistent or invalid.")]
    InvalidMatrixDimensions,
    #[error("Failed to read the file.")]
    FileReadError,
    #[error("Failed to parse the matrix: {0}")]
    ParseError(String),
    #[error("The matrix is not symmetrical or properly lower triangular.")]
    MatrixFormatError,
    #[error("Error opening or creating the graph file: {0}")]
    GraphFileError(String),
    #[error("Error serializing the graph: {0}")]
    GraphSerializationError(String),
    #[error("Error deserializing the graph: {0}")]
    GraphDeserializationError(String),
    #[error("Error writing to file: {0}")]
    WriteError(String),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Error retrieving NodeIndex during graph construction")]
    NodeIndexError,
    #[error("Failed to parse `skani` output matrix into symmetrical distance matrix")]
    ParseSkaniMatrix,
    #[error("Failed to find node with index {0} in the graph")]
    NodeNotFoundError(usize),
    #[error("Number of labels must be the same as number of nodes in the graph ({0})")]
    NodeLabelLengthError(usize),
    #[error(transparent)]
    NeedletailParseError(#[from] needletail::errors::ParseError),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    NifflerError(#[from] niffler::Error),
    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to parse TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    
    #[cfg(feature = "plot")]
    #[error(transparent)]
    PlottersBitmapError(#[from] plotters_bitmap::BitMapBackendError),
    #[cfg(feature = "plot")]
    #[error(transparent)]
    PlottersDrawinAreaBitmapError(#[from] DrawingAreaErrorKind<BitMapBackendError>),
}