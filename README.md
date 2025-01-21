# License Extractor

This Rust-based tool is designed to extract license information from an SBOM (Software Bill of Materials) file and return the results in a CSV format. The tool parses the SBOM, identifies the licenses associated with each package, and exports this data into a CSV file for further analysis or reporting.

## Features

- **SBOM Parsing**: Reads and parses SBOM files (e.g., in SPDX, CycloneDX formats).
- **License Extraction**: Extracts the license(s) associated with each package listed in the SBOM.
- **CSV Export**: Outputs the package name, version, and associated license(s) in CSV format.
- **Configurable Output Path**: Allows users to specify the output path for the CSV file.

## Requirements

- **Rust**: Ensure that you have [Rust](https://www.rust-lang.org/tools/install) installed on your machine.
- **SBOM File**: You will need a valid SBOM file in a supported format (e.g., SPDX or CycloneDX).

## Extracting Licenses

Clone the repository

   ```bash
   git clone https://github.com/mrrajan/sbom-license-export.git
   cd sbom-license-export
   ```

Run the command

    cargo run -- --sbom_file <sbom file path> --sbom_type <sbom type> -o <output csv location>

    