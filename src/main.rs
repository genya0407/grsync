/*
# GR_HOST is FIXED. DO NOT CHANGE!!
GR_HOST = "http://192.168.0.1/"
PHOTO_LIST_URI = "v1/photos"
GR_PROPS = "v1/props"
STARTDIR = ""
STARTFILE = ""
SUPPORT_DEVICE = ['RICOH GR II', 'RICOH GR III']
DEVICE = "RICOH GR II"
*/

#[derive(serde::Deserialize, Debug)]
struct Dir {
    name: String,
    files: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
struct Photos {
    #[serde(rename = "errMsg")]
    error_message: String,
    #[serde(rename = "errCode")]
    error_code: u64,
    dirs: Vec<Dir>,
}

#[argopt::cmd]
#[opt(author, version, about, long_about = None)]
fn main(
    #[opt(long = "host", default_value = "192.168.0.1")] host: String,
    #[opt(short = 'o', long = "output-dir", default_value = "downloaded_photos")]
    output_dir: String,
    /// Download all images including already downloaded ones
    #[opt(long)]
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()?;

    wait_for_connection(&host);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap();

    let photos_url = format!("http://{}/v1/photos", host);
    let photos = client.get(&photos_url).send()?.json::<Photos>()?;
    if photos.error_code != 200 {
        panic!(
            "Failed to GET {}, errMsg: {}",
            photos_url, photos.error_message
        )
    }

    for dir in photos.dirs {
        let dir_path = std::path::Path::new(&output_dir).join(&dir.name);
        std::fs::create_dir_all(&dir_path)?;
        for file_name in dir.files {
            let target_file_path = dir_path.join(&file_name);
            if !force && target_file_path.try_exists()? {
                log::info!("Skipping {}", file_name);
            } else {
                log::info!("Downloading {} ...", file_name);

                let photo_url = format!("http://{}/v1/photos/{}/{}", host, &dir.name, file_name);
                let resp = client.get(&photo_url).send()?;
                if resp.status() != 200 {
                    panic!("Failed to GET {}, message: {}", photo_url, resp.text()?);
                }

                let mut file = std::fs::File::create(target_file_path)?;
                let mut content = std::io::Cursor::new(resp.bytes()?);
                std::io::copy(&mut content, &mut file)?;
            }
        }
    }

    Ok(())
}

fn wait_for_connection(host: &str) {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .unwrap();

    let props_url = format!("http://{}/v1/props", host);
    loop {
        match client.get(&props_url).send() {
            Ok(_) => break,
            Err(_) => {
                log::warn!("Failed to fetch {}. Retrying...", props_url);
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
        }
    }
}
