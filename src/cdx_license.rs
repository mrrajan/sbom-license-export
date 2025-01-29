use serde_derive::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use csv::Writer;
use std::error::Error;
use regex::Regex;

#[derive(Serialize, Deserialize, Debug)]
pub struct License{
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LicenseEntry{
    pub license: Option<License>,
    pub expression: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Component{
    pub name: String,
    pub licenses: Option<Option<Vec<LicenseEntry>>>,
    pub purl: Option<String>,
    pub cpe: Option<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Components{
    pub metadata: SBOMLicense,
    pub components: Vec<Component>,
}

#[derive(Serialize, Debug)]
pub struct LicenseHeader<'a>{
    name: &'a String,
    package_reference: &'a String,
    license_id: &'a String,
    license_name: &'a String,
    //license_url: &'a String,
    license_expression: &'a String,
    alternate_reference_locator: &'a String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SBOMLicense{
    pub licenses: Option<Option<Vec<LicenseEntry>>>,
}

pub async fn get_cdx_bom_license(filepath: &str, output_path: &String){
    let mut file = File::open(filepath).await.expect("Error reading the file, make sure the path exists");
    let mut content_str = String::new();
    file.read_to_string(&mut content_str).await.expect("Error Reading file to variable");
    let data: Components = serde_json::from_str(&content_str).expect("Error converting Json");
    let _ = write_cdx_csv(&data, output_path).await;
}

pub async fn write_cdx_csv(comp: &Components, csv_path: &String) -> Result<(), Box<dyn Error>>{
    //let mut license_record = csv::Writer::from_writer(io::stdout());
    let mut wtr = Writer::from_path(csv_path)?;
        for component in &comp.components{      
        let package_name = component.name.clone();
        let mut cpe_name = "";
        if let Some(inner_cpe) = &component.cpe{
            if let Some(cpe) = inner_cpe{
                cpe_name = &cpe;
            }
        }
        if let Some(purl_nonempty) = &component.purl{
            let purl = purl_nonempty;
            if let Some(inner_licenses) = &component.licenses{
                if let Some(licenses) = inner_licenses{
                    let mut license_id = "";
                    let mut license_exp = "";
                    let mut license_name = "";
                    //let mut license_url = "";
                    for entry in licenses{
                        if let Some(license) = &entry.license{
                            if let Some(id)=&license.id{
                                license_id = id;
                            }
                            if let Some(name)=&license.name{
                                license_name = name;
                            }
                            // if let Some(url)=&license.url{
                            //     license_url = url;
                            // }
                            if !license_id.is_empty() && license_id != ""{    
                                let _ = wtr.serialize(LicenseHeader{
                                    name: &package_name.to_string(),
                                    package_reference: &purl.to_string(),
                                    license_id: &license_id.to_string(),
                                    license_name: &license_name.to_string(),
                                    //license_url:  &license_url.to_string(),
                                    license_expression: &license_exp.to_string(),
                                    alternate_reference_locator: &cpe_name.to_string(),
                                    }
                                );
                            }else{
                                let _ = wtr.serialize(LicenseHeader{
                                    name: &package_name.to_string(),
                                    package_reference: &purl.to_string(),
                                    license_id: &"".to_string(),
                                    license_name: &license_name.to_string(),
                                    //license_url:  &license_url.to_string(),
                                    license_expression: &license_exp.to_string(),
                                    alternate_reference_locator: &cpe_name.to_string(),
                                    }
                                );

                            }
                        }
                        if let Some(expression)=&entry.expression{
                            license_exp = expression;
                            if !license_exp.is_empty() && license_exp != ""{
                                let _ = wtr.serialize(LicenseHeader{
                                    name: &package_name.to_string(),
                                    package_reference: &purl.to_string(),
                                    license_id: &license_id.to_string(),
                                    license_name: &license_name.to_string(),
                                    //license_url:  &license_url.to_string(),
                                    license_expression: &license_exp.to_string(),
                                    alternate_reference_locator: &cpe_name.to_string(),
                                    }
                                );
                            }
                        }
                        
                        // This block is to split license expressions into individuals and handle them on each row.
                        // //let expression = license_exp.replace("(","").replace(")","");
                        // let re = Regex::new(r" OR | AND ").unwrap();
                        // let expression_list: Vec<&str> = re.split(&license_exp).collect();
                        // //let expression_list = expression.split(" OR ").clone();
                        // for exp in expression_list{
                        //     if !exp.is_empty() && exp != ""{
                        //         let _ = wtr.serialize(LicenseHeader{
                        //             name: &package_name.to_string(),
                        //             package_reference: &purl.to_string(),
                        //             license_id: &exp.replace("(","").replace(")","").to_string(),
                        //             license_name: &license_name.to_string(),
                        //             //license_url:  &license_url.to_string(),
                        //             //license_expression: &license_exp.to_string(),
                        //             alternate_reference_locator: &cpe_name.to_string;
                        //             }
                        //         );
                        //     }
                            
                        // }

                    }
                }
            }
        }
    }
    wtr.flush()?;
    Ok(())
}
