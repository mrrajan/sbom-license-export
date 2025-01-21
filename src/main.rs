mod cdx_license;
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
                    Arg::new("output_path")
                        .help("SBOM License CSV Path")
                        .short('o')
                        .long("csv_path")
                        .required(false)
                ).get_matches();

    let sbom_file = cli.get_one::<String>("sbom_file").unwrap();
    let sbom_type = cli.get_one::<String>("sbom_type").unwrap();
    let default_path = "license_cdx.csv".to_string();
    let csv_path = cli.get_one::<String>("output_path").unwrap_or(&default_path);
    if sbom_type == "cdx"{
        cdx_license::get_bom_license(sbom_file, csv_path).await;
    }
}