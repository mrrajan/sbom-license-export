use serde_derive::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use csv::Writer;
use std::error::Error;

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
    pub licenses: Option<Option<Vec<LicenseEntry>>>,
    #[serde[rename = "bom-ref"]]
    pub bomref: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Components{
    pub components: Vec<Component>,
}

#[derive(Serialize, Debug)]
pub struct LicenseHeader<'a>{
    package_reference: &'a String,
    license_id: &'a String,
    license_name: &'a String,
    license_url: &'a String,
    license_expression: &'a String,
}

pub async fn get_bom_license(filepath: &str, output_path: &String){
    let mut file = File::open(filepath).await.expect("Error reading the file, make sure the path exists");
    let mut content_str = String::new();
    file.read_to_string(&mut content_str).await.expect("Error Reading file to variable");
    let data: Components = serde_json::from_str(&content_str).expect("Error converting Json");
    let _ =write_to_csv(&data, output_path).await;
}

pub async fn write_to_csv(comp: &Components, csv_path: &String) -> Result<(), Box<dyn Error>>{
    //let mut license_record = csv::Writer::from_writer(io::stdout());
    let mut wtr = Writer::from_path(csv_path)?;
    for component in &comp.components{        
        let bom_ref = component.bomref.clone();
        if let Some(innerlicenses) = &component.licenses{
            if let Some(licenses) = innerlicenses{
                let mut licenseid = "";
                let mut licenseexp = "";
                let mut licensename = "";
                let mut licenseurl = "";
                for entry in licenses{
                    if let Some(license) = &entry.license{
                        if let Some(id)=&license.id{
                            licenseid = id;
                        }
                        if let Some(name)=&license.name{
                            licensename = name;
                        }
                        if let Some(url)=&license.url{
                            licenseurl = url;
                        }
                        
                    }
                    if let Some(expression)=&entry.expression{
                        licenseexp = expression;
                    }
                    let _ =wtr.serialize(LicenseHeader{
                            package_reference: &bom_ref.to_string(),
                            license_id: &licenseid.to_string(),
                            license_name: &licensename.to_string(),
                            license_url:  &licenseurl.to_string(),
                            license_expression: &licenseexp.to_string(),
                        }
                    );
                }
            }
        }

    }
    wtr.flush()?;
    Ok(())
}
