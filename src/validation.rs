use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::BufWriter;
use needletail::parser::LineEnding;
use rand::seq::SliceRandom;
use csv::WriterBuilder;
use needletail::parse_fastx_file;

use crate::error::NetviewError;
use crate::label::{read_labels_from_file, Label};
use crate::utils::write_fasta;


// Function to load FASTA sequences from the provided file path using needletail
fn load_fasta_sequences(fasta: &PathBuf) -> Result<HashMap<String, Vec<u8>>, NetviewError> {
    let mut sequences = HashMap::new();
    let mut reader = parse_fastx_file(&fasta)?;

    // Iterate through the FASTA file and store sequences by ID
    while let Some(record) = reader.next() {
        let record = record?;
        let id = std::str::from_utf8(record.id())?
            .split_whitespace()
            .collect::<Vec<_>>()[0]
            .to_string();

        let seq = record.seq().to_vec();  // Convert sequence to Vec<u8>

        sequences.insert(id, seq);
    }

    Ok(sequences)
}

pub struct CrossFoldValidation {
    labels: Vec<Label>,                       // Vector of all labels
    seqs: HashMap<String, Vec<u8>>,           // Path to the input FASTA file
    k_folds: usize,                           // Number of folds for cross-validation
    max_samples_per_label: Option<usize>,     // Maximum number of samples per label (optional)
    outdir: PathBuf,                          // Output directory for cross-validation data
}

impl CrossFoldValidation {
    pub fn new(
        labels: &PathBuf, 
        fasta: &PathBuf, 
        k_folds: usize, 
        max_samples_per_label: Option<usize>,  // Add max_samples_per_label here
        outdir: &PathBuf
    ) -> Result<Self, NetviewError> {
        if !outdir.exists() {
            fs::create_dir_all(&outdir)?;
        }

        let labels = read_labels_from_file(labels, false)?;
        let seqs = load_fasta_sequences(fasta)?;

        Ok(Self {
            labels,
            seqs,
            k_folds,
            max_samples_per_label,
            outdir: outdir.to_owned(),
        })
    }

    // Main function to generate k-fold cross-validation data
    pub fn generate_k_folds(&self) -> Result<(), NetviewError> {
        
        // Group labels by their class for stratification
        let label_groups = self.group_labels_by_class();

        // Perform stratified sampling for k-fold cross-validation
        let folds = self.stratified_k_fold_sampling(&label_groups)?;

        // Output each fold's training and test data
        for (fold_idx, (train_ids, test_ids)) in folds.iter().enumerate() {
            self.write_fold_data(fold_idx, train_ids, test_ids, &self.seqs)?;
        }

        Ok(())
    }

    // Function to group labels by their class for stratification
    fn group_labels_by_class(&self) -> HashMap<Option<String>, Vec<Label>> {
        let mut label_groups: HashMap<Option<String>, Vec<Label>> = HashMap::new();
        for label in &self.labels {
            label_groups
                .entry(label.label.clone())
                .or_insert_with(Vec::new)
                .push(label.clone());
        }
        label_groups
    }

    // Function to perform stratified sampling for k-fold cross-validation with an optional limit on the number of samples per label
    fn stratified_k_fold_sampling(
        &self,
        label_groups: &HashMap<Option<String>, Vec<Label>>,
    ) -> Result<Vec<(Vec<String>, Vec<String>)>, NetviewError> {
        let mut folds = vec![(Vec::new(), Vec::new()); self.k_folds]; // (train_ids, test_ids) for each fold
        let mut rng = rand::thread_rng();

        for labels in label_groups.values() {
            // Shuffle the labels within each class to ensure randomness
            let mut shuffled_labels = labels.clone();
            shuffled_labels.shuffle(&mut rng);

            // Apply the maximum number of samples per label if specified
            let selected_labels = if let Some(max) = self.max_samples_per_label {
                shuffled_labels.into_iter().take(max).collect::<Vec<_>>() // Take only up to 'max' samples
            } else {
                shuffled_labels // Take all samples if no max is specified
            };

            // Split the selected labels across k folds
            for (i, label) in selected_labels.iter().enumerate() {
                let fold_idx = i % self.k_folds;
                // Assign to training or test set for each fold
                for (train, test) in folds.iter_mut().enumerate() {
                    if fold_idx == train {
                        test.1.push(label.id.clone());
                    } else {
                        test.0.push(label.id.clone());
                    }
                }
            }
        }

        Ok(folds)
    }

    // Function to write the training and test data for each fold
    fn write_fold_data(
        &self,
        fold_idx: usize,
        train_ids: &Vec<String>,
        test_ids: &Vec<String>,
        fasta_sequences: &HashMap<String, Vec<u8>>,
    ) -> Result<(), NetviewError> {
        // Create a directory for this fold
        let fold_dir = self.outdir.join(format!("fold_{}", fold_idx));
        fs::create_dir_all(&fold_dir)?;

        // Write training sequences
        let train_fasta_path = fold_dir.join("train_sequences.fasta");
        let mut train_fasta = BufWriter::new(File::create(train_fasta_path)?);
        for id in train_ids {
            if let Some(seq) = fasta_sequences.get(id) {
                write_fasta(id.as_bytes(), &seq, &mut train_fasta, LineEnding::Unix)?;
            }
        }

        // Write test sequences
        let test_fasta_path = fold_dir.join("test_sequences.fasta");
        let mut test_fasta = BufWriter::new(File::create(test_fasta_path)?);
        for id in test_ids {
            if let Some(seq) = fasta_sequences.get(id) {
                write_fasta(id.as_bytes(), &seq, &mut test_fasta, LineEnding::Unix)?;
            }
        }

        // Write training labels
        let train_labels_path = fold_dir.join("train_labels.csv");
        let mut train_wtr = WriterBuilder::new().from_path(train_labels_path)?;
        for label in &self.labels {
            if train_ids.contains(&label.id) {
                train_wtr.serialize(label)?;
            }
        }
        train_wtr.flush()?;

        // Write test labels
        let test_labels_path = fold_dir.join("test_labels.csv");
        let mut test_wtr = WriterBuilder::new().from_path(test_labels_path)?;
        for label in &self.labels {
            if test_ids.contains(&label.id) {
                test_wtr.serialize(label)?;
            }
        }
        test_wtr.flush()?;

        Ok(())
    }
}
