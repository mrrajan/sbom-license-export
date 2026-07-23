use serde_derive::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use csv::{QuoteStyle, Writer, WriterBuilder};
use std::error::Error; 
use log::info;
use regex::Regex;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReferenceObj{
    pub referenceCategory: String,
    pub referenceLocator: String,
    pub referenceType: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageObj{
    pub licenseDeclared: Option<String>,
    pub licenseConcluded: Option<String>,
    pub externalRefs: Option<Option<Vec<ReferenceObj>>>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packages{
    pub packages: Vec<PackageObj>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LicenseInfo{
    pub extractedText: String,
    pub licenseId: String,
    pub name: String,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HasLicenseInfo{
    pub hasExtractedLicensingInfos: Option<Option<Vec<LicenseInfo>>>,
    pub documentNamespace: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LicenseHeader{
    name: String,
    namespace: String,
    group: String,
    version: String,
    #[serde(rename = "package reference")]
    package_reference: String,
    license: String,
    #[serde(rename = "alternate package reference")]
    alternate_ref: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LicenseRefHeader{
    #[serde(rename = "licenseId")]
    license_id: String,
    name: String,
    #[serde(rename = "extracted text")]
    extracted_text: String,
    comment: String,
}

pub async fn get_spdx_bom_license(filepath: &str, output_path: &String, ref_file_path: &String){
    let mut file = File::open(filepath).await.expect("Error reading the file, make sure the path exists");
    let mut content_str = String::new();
    file.read_to_string(& mut content_str).await.expect("Error Reading file to variable");
    let data: Packages = serde_json::from_str(&content_str).expect("Error converting Json");
    let license_extract: HasLicenseInfo = serde_json::from_str(&content_str).expect("Error converting Json");
    //let _ = write_spdx_csv(&data, &license_extract, output_path).await;
    let _ = write_simple_spdx_csv(&data, &license_extract, output_path).await;
    let _ = write_ref_csv(&license_extract, ref_file_path).await;
}

pub async fn write_ref_csv(licenseRef: &HasLicenseInfo, ref_file_path: &String) -> Result<(), Box<dyn Error>>{
    let mut wrt_ref = WriterBuilder::new()
        .delimiter(b'\t')
        .quote_style(QuoteStyle::Always)
        .from_path(ref_file_path)?;

    wrt_ref.write_record(&["licenseId", "name", "extracted text", "comment"])?;

    if let Some(inner_license_map) = &licenseRef.hasExtractedLicensingInfos{
        if let Some(license_map) = inner_license_map{
            for license_info in license_map{
                wrt_ref.write_record(&[
                    &license_info.licenseId,
                    &license_info.name,
                    &license_info.extractedText,
                    &license_info.comment,
                ])?;
            }
        }
    }
    wrt_ref.flush()?;
    Ok(())
}

pub async fn write_simple_spdx_csv(packages: &Packages, license_extract: &HasLicenseInfo, csv_path: &String) -> Result<(), Box<dyn Error>>{
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .quote_style(QuoteStyle::Always)
        .has_headers(false)
        .from_path(csv_path)?;

    wtr.write_record(&["name", "namespace", "group", "version", "package reference", "license", "alternate package reference"])?;

    for package in &packages.packages{
        let mut purl = "";
        let mut license_declared = "";
        let mut license_concluded = "";
        let mut alternate_ref = Vec::new();
        if let Some(license_expression) = &package.licenseDeclared{
            license_declared = license_expression;
        }
        if let Some(license_expression) = &package.licenseConcluded{
            license_concluded = license_expression;
        }
        if let Some(inner_external_ref) = &package.externalRefs{
            if let Some(external_refs) = inner_external_ref{
                for reference in external_refs{
                    if &reference.referenceType == "purl"{
                        purl = &reference.referenceLocator;
                    }else{
                        alternate_ref.push(reference.referenceLocator.clone());
                    }
                }
            }
        }
        let alternate_ref_str = alternate_ref.join("\n");
        if !license_declared.is_empty() {
            wtr.serialize(LicenseHeader{
                name: license_extract.name.to_string(),
                namespace: license_extract.documentNamespace.to_string(),
                group: "".to_string(),
                version: "".to_string(),
                package_reference: purl.to_string(),
                license: license_declared.to_string(),
                alternate_ref: alternate_ref_str.clone(),
            });
        }
        if !license_concluded.is_empty() {
            wtr.serialize(LicenseHeader{
                name: license_extract.name.to_string(),
                namespace: license_extract.documentNamespace.to_string(),
                group: "".to_string(),
                version: "".to_string(),
                package_reference: purl.to_string(),
                license: license_concluded.to_string(),
                alternate_ref: alternate_ref_str,
            });
        }
    }
    wtr.flush()?;
    Ok(())
}

pub async fn write_spdx_csv(packages: &Packages, licenseRef: &HasLicenseInfo, csv_path: &String) -> Result<(), Box<dyn Error>>{
    // This block of code is extensive
    // It is capable of splitting the Licenses under license Declared field and remove the brackets and write it on separate rows
    // It is capable of mapping the license ID's against hasExtractedLicensingInfos field in SBOM and update "license name" column
    // Right now, it is not being used
    let mut wtr = Writer::from_path(csv_path)?;
    for package in &packages.packages{
        let package_name = &package.name;
        let re = Regex::new(r" OR | AND ").unwrap();
        if let Some(license_id_spdx) = &package.licenseDeclared{
            let license_id_list = license_id_spdx.replace("(","").replace(")","");
            let license_ids: Vec<&str> = re.split(&license_id_list).collect();
            for id in license_ids{
                let mut license_name = id;
                if let Some(license_map) = &licenseRef.hasExtractedLicensingInfos{
                    if let Some(mapping) = license_map{
                        for map in mapping{
                            if map.licenseId == license_name{
                                license_name = &map.name;
                                break;
                            }
                        }
                    }
                }
                if let Some(inner_external_ref) = &package.externalRefs{
                    if let Some(external_refs) = inner_external_ref{
                        let mut purl = "";
                        let mut alternate_ref = Vec::new();
                        for reference in external_refs{
                            if &reference.referenceType == "purl"{
                                purl = &reference.referenceLocator;
                            }else{
                                alternate_ref.push(reference.referenceLocator.clone());
                            }
                        }
                        wtr.serialize(LicenseHeader{
                            name: package_name.to_string(),
                            namespace: licenseRef.documentNamespace.to_string(),
                            group: "".to_string(),
                            version: "".to_string(),
                            package_reference: purl.to_string(),
                            license: id.to_string(),
                            alternate_ref: alternate_ref.join("\n").to_string(),
                        });
                    }
                }
            }
        }
    }
    wtr.flush()?;
    Ok(())
}
