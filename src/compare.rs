use csv::ReaderBuilder;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

fn find_file_matching(dir: &Path, suffix: &str) -> Option<String> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(suffix) {
                return Some(entry.path().to_string_lossy().to_string());
            }
        }
    }
    None
}

fn read_tsv_keys(
    path: &str,
    purl_col: &str,
    license_col: &str,
    type_col: Option<&str>,
) -> Result<HashSet<(String, String, String)>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .quoting(true)
        .from_path(path)?;
    let headers = rdr.headers()?.clone();
    let purl_idx = headers.iter().position(|h| h == purl_col)
        .ok_or_else(|| format!("Column '{}' not found in {}", purl_col, path))?;
    let lic_idx = headers.iter().position(|h| h == license_col)
        .ok_or_else(|| format!("Column '{}' not found in {}", license_col, path))?;
    let type_idx = type_col.map(|tc| headers.iter().position(|h| h == tc)
        .ok_or_else(|| format!("Column '{}' not found in {}", tc, path)))
        .transpose()?;

    let mut keys = HashSet::new();
    for result in rdr.records() {
        let record = result?;
        let purl = record.get(purl_idx).unwrap_or("").to_string();
        let lic = record.get(lic_idx).unwrap_or("").to_string();
        let ltype = type_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .to_string();
        keys.insert((purl, ltype, lic));
    }
    Ok(keys)
}

fn read_ref_keys(path: &str) -> Result<HashSet<(String, String)>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(path)?;
    let headers = rdr.headers()?.clone();
    let id_idx = headers.iter().position(|h| h == "licenseId")
        .ok_or_else(|| format!("Column 'licenseId' not found in {}", path))?;
    let name_idx = headers.iter().position(|h| h == "name")
        .ok_or_else(|| format!("Column 'name' not found in {}", path))?;

    let mut keys = HashSet::new();
    for result in rdr.records() {
        let record = result?;
        let id = record.get(id_idx).unwrap_or("").to_string();
        let name = record.get(name_idx).unwrap_or("").to_string();
        keys.insert((id, name));
    }
    Ok(keys)
}

fn print_license_diff(our_keys: &HashSet<(String, String, String)>, their_keys: &HashSet<(String, String, String)>) {
    let common = our_keys.intersection(their_keys).count();
    let only_ours: Vec<_> = our_keys.difference(their_keys).collect();
    let only_theirs: Vec<_> = their_keys.difference(our_keys).collect();

    println!("  Common entries: {}", common);
    println!("  Only in our output: {}", only_ours.len());
    println!("  Only in comparator: {}", only_theirs.len());

    if !only_ours.is_empty() {
        println!("  --- Only in our output (first 10): ---");
        for (i, (purl, ltype, lic)) in only_ours.iter().enumerate() {
            if i >= 10 { break; }
            println!("    purl={}, type={}, license={}", purl, ltype, lic);
        }
    }
    if !only_theirs.is_empty() {
        println!("  --- Only in comparator (first 10): ---");
        for (i, (purl, ltype, lic)) in only_theirs.iter().enumerate() {
            if i >= 10 { break; }
            println!("    purl={}, type={}, license={}", purl, ltype, lic);
        }
    }
}

pub fn compare_licenses(our_csv: &str, our_ref_csv: Option<&str>, compare_dir: &str, sbom_type: &str) {
    let dir = Path::new(compare_dir);
    if !dir.is_dir() {
        println!("Compare path '{}' is not a directory", compare_dir);
        return;
    }

    let their_licenses = find_file_matching(dir, "_sbom_licenses.csv");
    let their_ref = find_file_matching(dir, "_license_ref.csv");

    if let Some(their_lic_path) = &their_licenses {
        println!("\n=== License CSV Comparison ===");
        println!("Our file:   {}", our_csv);
        println!("Their file: {}", their_lic_path);

        let our_result = read_tsv_keys(our_csv, "package reference", "license", None);

        let their_result = read_tsv_keys(their_lic_path, "package purl", "license", None);

        match (our_result, their_result) {
            (Ok(our_keys), Ok(their_keys)) => {
                println!("  Our rows:   {}", our_keys.len());
                println!("  Their rows: {}", their_keys.len());
                print_license_diff(&our_keys, &their_keys);
            }
            (Err(e), _) => println!("  Error reading our file: {}", e),
            (_, Err(e)) => println!("  Error reading comparator file: {}", e),
        }
    } else {
        println!("\nNo *_sbom_licenses.csv found in {}", compare_dir);
    }

    if sbom_type == "spdx" {
        if let (Some(our_ref), Some(their_ref_path)) = (our_ref_csv, &their_ref) {
            println!("\n=== License Ref Comparison ===");
            println!("Our file:   {}", our_ref);
            println!("Their file: {}", their_ref_path);

            match (read_ref_keys(our_ref), read_ref_keys(their_ref_path)) {
                (Ok(our_keys), Ok(their_keys)) => {
                    let common = our_keys.intersection(&their_keys).count();
                    let only_ours: Vec<_> = our_keys.difference(&their_keys).collect();
                    let only_theirs: Vec<_> = their_keys.difference(&our_keys).collect();

                    println!("  Our entries:   {}", our_keys.len());
                    println!("  Their entries: {}", their_keys.len());
                    println!("  Common: {}", common);
                    println!("  Only in our output: {}", only_ours.len());
                    println!("  Only in comparator: {}", only_theirs.len());

                    for (id, name) in &only_ours {
                        println!("    [ours] {} -> {}", id, name);
                    }
                    for (id, name) in &only_theirs {
                        println!("    [theirs] {} -> {}", id, name);
                    }
                }
                (Err(e), _) => println!("  Error reading our file: {}", e),
                (_, Err(e)) => println!("  Error reading comparator file: {}", e),
            }
        } else if our_ref_csv.is_some() && their_ref.is_none() {
            println!("\nNo *_license_ref.csv found in {}", compare_dir);
        }
    }
}
