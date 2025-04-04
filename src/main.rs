use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::process;
use std::collections::HashSet;

use clap::{Parser, Subcommand};
use serde_json::{Value, Map};

#[derive(Parser)]
#[clap(name = "csv-json-converter")]
#[clap(about = "A CLI tool to convert between CSV and JSON formats")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert CSV to JSON
    CsvToJson {
        /// Input CSV file
        #[clap(short, long)]
        input: PathBuf,
        
        /// Output JSON file
        #[clap(short, long)]
        output: PathBuf,
        
        /// Output as array of objects (default) or as array of arrays
        #[clap(short, long, default_value = "false")]
        array_format: bool,
        
        /// CSV delimiter character (default: ,)
        #[clap(short, long, default_value = ",")]
        delimiter: char,
        
        /// CSV has headers (default: true)
        #[clap(long, default_value = "true")]
        has_headers: bool,
        
        /// Trim whitespace from fields (default: false)
        #[clap(long, default_value = "false")]
        trim: bool,
    },
    /// Convert JSON to CSV
    JsonToCsv {
        /// Input JSON file
        #[clap(short, long)]
        input: PathBuf,
        
        /// Output CSV file
        #[clap(short, long)]
        output: PathBuf,
        
        /// CSV delimiter character (default: ,)
        #[clap(short, long, default_value = ",")]
        delimiter: char,
        
        /// Quote all non-numeric fields (default: false)
        #[clap(long, default_value = "false")]
        quote_all: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::CsvToJson { 
            input, 
            output, 
            array_format, 
            delimiter, 
            has_headers, 
            trim 
        } => {
            csv_to_json(input, output, array_format, delimiter, has_headers, trim)?;
        }
        Commands::JsonToCsv { 
            input, 
            output, 
            delimiter, 
            quote_all 
        } => {
            json_to_csv(input, output, delimiter, quote_all)?;
        }
    }

    Ok(())
}

fn csv_to_json(
    input: PathBuf, 
    output: PathBuf, 
    array_format: bool,
    delimiter: char,
    has_headers: bool,
    trim: bool
) -> Result<(), Box<dyn Error>> {
    // Open the CSV file
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    
    // Create a new CSV reader with the specified options
    let builder = csv::ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(has_headers)
        .trim(if trim { csv::Trim::All } else { csv::Trim::None })
        .from_reader(reader);
    
    let mut csv_reader = builder;
    
    // Prepare the output file
    let file = File::create(output)?;
    let writer = BufWriter::new(file);
    
    if array_format {
        // Create an array of arrays format
        let mut rows = Vec::new();
        
        // If there are headers, add them as the first row
        if has_headers {
            let headers: Vec<String> = csv_reader.headers()?
                .iter()
                .map(String::from)
                .collect();
            rows.push(headers);
        }
        
        // Add data rows
        for result in csv_reader.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(String::from).collect();
            rows.push(row);
        }
        
        serde_json::to_writer_pretty(writer, &rows)?;
    } else {
        // Create an array of objects format
        let mut json_array = Vec::new();
        
        if has_headers {
            // Get headers once
            let headers = csv_reader.headers()?.clone();
            
            // Process records
            for result in csv_reader.records() {
                let record = result?;
                let mut obj = Map::new();
                
                for (i, header) in headers.iter().enumerate() {
                    if let Some(value) = record.get(i) {
                        obj.insert(header.to_string(), Value::String(value.to_string()));
                    }
                }
                
                json_array.push(Value::Object(obj));
            }
        } else {
            // No headers, use positional indices
            let mut _record_num = 0;
            for result in csv_reader.records() {
                let record = result?;
                let mut obj = Map::new();
                
                for (i, value) in record.iter().enumerate() {
                    obj.insert(format!("field{}", i), Value::String(value.to_string()));
                }
                
                json_array.push(Value::Object(obj));
                _record_num += 1;
            }
        }
        
        serde_json::to_writer_pretty(writer, &json_array)?;
    }
    
    println!("CSV successfully converted to JSON");
    Ok(())
}

fn json_to_csv(
    input: PathBuf, 
    output: PathBuf,
    delimiter: char,
    quote_all: bool
) -> Result<(), Box<dyn Error>> {
    // Open the JSON file
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;
    
    // Prepare the output file
    let file = File::create(output)?;
    
    // Configure the CSV writer
    let writer = csv::WriterBuilder::new()
        .delimiter(delimiter as u8)
        .quote_style(if quote_all { 
            csv::QuoteStyle::Always 
        } else { 
            csv::QuoteStyle::Necessary 
        })
        .from_writer(file);
    
    let mut csv_writer = writer;
    
    // Process based on JSON format
    match json {
        Value::Array(array) => {
            if array.is_empty() {
                return Ok(());
            }
            
            // Check if array of arrays or array of objects
            match &array[0] {
                Value::Array(_) => {
                    // Array of arrays format
                    for (i, row) in array.iter().enumerate() {
                        if let Value::Array(values) = row {
                            let str_values: Vec<String> = values
                                .iter()
                                .map(|v| match v {
                                    Value::String(s) => s.clone(),
                                    Value::Null => String::new(),
                                    _ => v.to_string(),
                                })
                                .collect();
                            
                            csv_writer.write_record(&str_values)?;
                        } else {
                            return Err(format!("Row {i} is not an array").into());
                        }
                    }
                },
                Value::Object(_) => {
                    // Array of objects format
                    // Preserve the order of headers from the first object
                    // and add any additional headers from other objects
                    let mut ordered_headers: Vec<String> = Vec::new();
                    let mut seen_headers = HashSet::new();
                    
                    // First, collect headers from the first object to establish initial order
                    if let Value::Object(first_obj) = &array[0] {
                        for key in first_obj.keys() {
                            ordered_headers.push(key.clone());
                            seen_headers.insert(key.clone());
                        }
                    }
                    
                    // Then collect any additional headers from other objects
                    for obj in &array {
                        if let Value::Object(map) = obj {
                            for key in map.keys() {
                                if !seen_headers.contains(key) {
                                    ordered_headers.push(key.clone());
                                    seen_headers.insert(key.clone());
                                }
                            }
                        }
                    }
                    
                    // Write headers
                    csv_writer.write_record(&ordered_headers)?;
                    
                    // Write data rows
                    for obj in &array {
                        if let Value::Object(map) = obj {
                            let row: Vec<String> = ordered_headers
                                .iter()
                                .map(|header| {
                                    map.get(header)
                                        .map(|v| match v {
                                            Value::String(s) => s.clone(),
                                            Value::Null => String::new(),
                                            _ => v.to_string(),
                                        })
                                        .unwrap_or_default()
                                })
                                .collect();
                            
                            csv_writer.write_record(&row)?;
                        }
                    }
                },
                _ => return Err("JSON array must contain arrays or objects".into()),
            }
        },
        _ => return Err("JSON must be an array".into()),
    }
    
    // Flush the writer to ensure all data is written
    csv_writer.flush()?;
    println!("JSON successfully converted to CSV");
    Ok(())
}