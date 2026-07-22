mod cdx_license;
mod compare;
mod spdx_license;
use clap::{Command, Arg};
use simplelog::*;

#[tokio::main]
async fn main(){
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            std::fs::File::create("sbom_license.log").unwrap(),
        ),
    ]).unwrap();
    let cli = Command::new("SBOMLX")
                .about("To export licenses from given SBOM file")
                .arg(
                    Arg::new("sbom_file")
                        .help("SBOM file path")
                        .short('p')
                        .long("sbom_file")
                        .required(true)
                )
                .arg(
                    Arg::new("sbom_type")
                        .help("SBOM file path")
                        .short('t')
                        .long("sbom_type")
                        .required(true)
                )
                .arg(
                    Arg::new("csv_path")
                        .help("SBOM License CSV Path")
                        .short('o')
                        .long("csv_path")
                        .required(false)
                )
                .arg(
                    Arg::new("ref_file_path")
                        .help("SPDX SBOM License Reference Path")
                        .short('r')
                        .long("ref_file_path")
                        .required(false)
                )
                .arg(
                    Arg::new("compare_path")
                        .help("Directory with trustify license export to compare against")
                        .short('c')
                        .long("compare_path")
                        .required(false)
                ).get_matches();

    let sbom_file = cli.get_one::<String>("sbom_file").unwrap();
    let sbom_type = cli.get_one::<String>("sbom_type").unwrap();
    let sbom_stem = std::path::Path::new(sbom_file.as_str())
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("license");
    let default_path = format!("{}_licenses.csv", sbom_stem);
    let csv_path = cli.get_one::<String>("csv_path").unwrap_or(&default_path);
    let compare_path = cli.get_one::<String>("compare_path");
    if sbom_type == "cdx"{
        cdx_license::get_cdx_bom_license(sbom_file, csv_path).await;
        if let Some(cmp) = compare_path {
            compare::compare_licenses(csv_path, None, cmp, sbom_type);
        }
    } else if sbom_type == "spdx"{
        let default_ref_path = format!("{}_license_ref.csv", sbom_stem);
        let ref_file_path = cli.get_one::<String>("ref_file_path").unwrap_or(&default_ref_path);
        spdx_license::get_spdx_bom_license(sbom_file, csv_path, ref_file_path).await;
        if let Some(cmp) = compare_path {
            compare::compare_licenses(csv_path, Some(ref_file_path.as_str()), cmp, sbom_type);
        }
    }
}