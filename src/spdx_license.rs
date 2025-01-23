use serde_derive::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use csv::Writer;
use std::error::Error; 
use log::info;


#[derive(Serialize, Deserialize, Debug)]
pub struct ReferenceObj{
    pub referenceCategory: String,
    pub referenceLocator: String,
    pub referenceType: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageObj{
    pub licenseDeclared: Option<String>,
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HasLicenseInfo{
    pub hasExtractedLicensingInfos: Option<Option<Vec<LicenseInfo>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LicenseHeader{
    name: String,
    package_reference: String,
    license_id: String,
    license_name: String,
    alternate_ref: String,
}

pub async fn get_spdx_bom_license(filepath: &str, output_path: &String){
    let mut file = File::open(filepath).await.expect("Error reading the file, make sure the path exists");
    let mut content_str = String::new();
    file.read_to_string(& mut content_str).await.expect("Error Reading file to variable");
    let data: Packages = serde_json::from_str(&content_str).expect("Error converting Json");
    let license_extract: HasLicenseInfo = serde_json::from_str(&content_str).expect("Error converting Json");
    let _ = write_spdx_csv(&data, &license_extract, output_path).await;
}

pub async fn write_spdx_csv(packages: &Packages, licenseRef: &HasLicenseInfo, csv_path: &String) -> Result<(), Box<dyn Error>>{
    let mut wtr = Writer::from_path(csv_path)?;
    for package in &packages.packages{
        let package_name = &package.name;
        if let Some(license_id_spdx) = &package.licenseDeclared{
            let license_ids = license_id_spdx.split("OR").clone();
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
                            package_reference: purl.to_string(),
                            license_id: id.to_string(),
                            license_name: license_name.to_string(),
                            alternate_ref: alternate_ref.join(" ").to_string(),
                        });
                    }
                }
            }
        }
    }
    wtr.flush()?;
    Ok(())
}
