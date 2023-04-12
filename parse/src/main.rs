use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProofClaim {
    index: u32,
    amount: String,
    proof: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JsonFile {
    token: String,
    #[serde(rename = "networkId")]
    network_id: u32,
    #[serde(rename = "merkleRoot")]
    merkle_root: String,
    #[serde(rename = "tokenTotal")]
    token_total: String,
    claims: HashMap<String, ProofClaim>,
}

fn read_and_parse_json_file<P: AsRef<Path>>(
    path: P,
) -> Result<JsonFile, Box<dyn std::error::Error>> {
    // Read the file
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Parse the JSON content
    let json_data: JsonFile = serde_json::from_reader(reader)?;

    Ok(json_data)
}

fn write_json_file<P: AsRef<Path>>(
    path: P,
    json_data: &JsonFile,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    let json_string = serde_json::to_string_pretty(json_data)?;

    file.write_all(json_string.as_bytes())?;
    Ok(())
}

fn convert_hex_to_base10(json_data: &mut JsonFile) {
    for (_address, claim) in json_data.claims.iter_mut() {
        let amount_base10 = BigInt::parse_bytes(&claim.amount[2..].as_bytes(), 16)
            .unwrap_or_else(|| BigInt::from_u64(0).unwrap());
        claim.amount = amount_base10.to_string();
    }
}

fn main() {
    let input_path = "output.json";
    let output_path = "new-output.json";

    match read_and_parse_json_file(input_path) {
        Ok(mut json_data) => {
            convert_hex_to_base10(&mut json_data);

            if let Err(error) = write_json_file(output_path, &json_data) {
                println!("Error writing JSON file: {}", error);
            }
        }
        Err(error) => println!("Error reading JSON file: {}", error),
    }
}
