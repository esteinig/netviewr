use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use needletail::parse_fastx_file;
use crate::error::NetviewError;
use crate::label::{read_labels_from_file, write_labels_to_file, Label};
use crate::utils::write_fasta;

pub struct Dereplicator<'a> {
    fasta_path: &'a PathBuf,
    label_path: &'a PathBuf,
    max_per_label: usize,
}

impl<'a> Dereplicator<'a> {
    // Constructor
    pub fn new(fasta_path: &'a PathBuf, label_path: &'a PathBuf, max_per_label: usize) -> Self {
        Dereplicator {
            fasta_path,
            label_path,
            max_per_label,
        }
    }

    // Function to perform the dereplication
    pub fn dereplicate(&self, output_fasta: &PathBuf, output_labels: &PathBuf, exclude: &Vec<String>, min_length: usize) -> Result<(), NetviewError> {
        // Load sequences from FASTA
        let sequences = self.load_fasta_sequences(self.fasta_path, min_length)?;

        // Load labels from CSV/TSV
        let labels = read_labels_from_file(self.label_path, false)?; // Assuming CSV here, adjust as needed

        // Group sequences by label and dereplicate
        let selected_sequences = self.group_and_select_sequences(&sequences, &labels, exclude);

        // Write the dereplicated sequences to the output FASTA file
        let mut fasta_writer = BufWriter::new(File::create(output_fasta)?);
        let mut fasta_labels = Vec::new();

        for (label, seq) in selected_sequences {
            write_fasta(label.id.as_bytes(), &seq, &mut fasta_writer, needletail::parser::LineEnding::Unix)?;
            fasta_labels.push(label)
        }

        write_labels_to_file(&fasta_labels, output_labels, false)?;

        Ok(())
    }

    // Load sequences from the FASTA file using needletail
    fn load_fasta_sequences(&self, fasta: &PathBuf, min_length: usize) -> Result<HashMap<String, Vec<u8>>, NetviewError> {
        let mut sequences = HashMap::new();
        let mut reader = parse_fastx_file(fasta)?;

        // Iterate through the FASTA file and store sequences by ID
        while let Some(record) = reader.next() {
            let record = record?;
            let id = std::str::from_utf8(record.id())?
                .split_whitespace()
                .collect::<Vec<_>>()[0]
                .to_string();

            let seq = record.seq().to_vec();  // Convert sequence to Vec<u8>

            if record.num_bases() >= min_length {
                sequences.insert(id, seq);
            }
        }

        Ok(sequences)
    }

    // Group sequences by label and select up to `max_per_label` sequences for each label
    fn group_and_select_sequences(
        &self,
        sequences: &HashMap<String, Vec<u8>>,
        labels: &[Label],
        exclude: &Vec<String>,
    ) -> HashMap<Label, Vec<u8>> {

        let mut label_groups: HashMap<Option<String>, Vec<&Label>> = HashMap::new();
        let mut selected_sequences: HashMap<Label, Vec<u8>> = HashMap::new();
        let mut used_ids = HashSet::new();

        // Group labels by their label value (label field in the Label struct)
        for label in labels {
            label_groups
                .entry(label.label.clone())
                .or_insert_with(Vec::new)
                .push(label);
        }

        // For each label group, select up to `max_per_label` sequences
        for (label, label_list) in label_groups {

            // Exclude unlabelled from dereplication 
            if let Some(label) = label {

                // Exclude specific labels from dereplication 
                if exclude.contains(&label) {
                    continue
                }

                let mut count = 0;

                for label_entry in label_list {
                    if let Some(seq) = sequences.get(&label_entry.id) {
                        if count < self.max_per_label && !used_ids.contains(&label_entry.id) {
                            selected_sequences.insert(label_entry.clone(), seq.clone());
                            used_ids.insert(label_entry.id.clone());
                            count += 1;
                        }
                    }
                }
            }
            
        }
        selected_sequences
    }
}
