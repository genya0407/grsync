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

struct Grsync {
    host: String,
    output_dir: String,
    force: bool,
}

impl Grsync {
    fn from_cli(host: String, output_dir: String, force: bool) -> Self {
        Self {
            host,
            output_dir,
            force,
        }
    }

    pub fn download(&self) {
        self.wait_for_server();

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap();

        let photos_url = format!("http://{}/v1/photos", self.host);
        let photos = client
            .get(&photos_url)
            .send()
            .unwrap()
            .json::<Photos>()
            .unwrap();
        if photos.error_code != 200 {
            panic!(
                "Failed to GET {}, errMsg: {}",
                photos_url, photos.error_message
            )
        }

        for dir in photos.dirs {
            let dir_path = std::path::Path::new(&self.output_dir).join(&dir.name);
            std::fs::create_dir_all(&dir_path).unwrap();
            for file_name in dir.files {
                let target_file_path = dir_path.join(&file_name);
                if !self.force && target_file_path.try_exists().unwrap() {
                    log::info!("Skipping {}", file_name);
                } else {
                    log::info!("Downloading {} ...", file_name);

                    let photo_url =
                        format!("http://{}/v1/photos/{}/{}", self.host, &dir.name, file_name);
                    let resp = client.get(&photo_url).send().unwrap();
                    if resp.status() != 200 {
                        panic!(
                            "Failed to GET {}, message: {}",
                            photo_url,
                            resp.text().unwrap()
                        );
                    }

                    let mut file = std::fs::File::create(target_file_path).unwrap();
                    let mut content = std::io::Cursor::new(resp.bytes().unwrap());
                    std::io::copy(&mut content, &mut file).unwrap();
                }
            }
        }
    }

    fn wait_for_server(&self) {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(1))
            .build()
            .unwrap();

        let props_url = format!("http://{}/v1/props", self.host);
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
}

#[argopt::cmd]
#[opt(author, version, about, long_about = None)]
fn main(
    #[opt(long = "host", default_value = "192.168.0.1")] host: String,
    #[opt(short = 'o', long = "output-dir", default_value = "downloaded_photos")]
    output_dir: String,
    /// Download all images, including already downloaded ones
    #[opt(long)]
    force: bool,
) {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .unwrap();

    Grsync::from_cli(host, output_dir, force).download();
}
