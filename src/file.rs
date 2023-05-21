//use cfg_if::cfg_if;
use symphonia::core::io::MediaSourceStream;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&location.origin().unwrap()).unwrap();

    base.join(file_name).unwrap()
}

pub async fn load_image(file_name: &str) -> image::DynamicImage {
    let data = {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let url = format_url(file_name);

                reqwest::get(url)
                    .await
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap()
                    .to_vec()
            } else {
                let path = std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                    .join(file_name);

                std::fs::read(path).unwrap()
            }
        }
    };

    image::load_from_memory(&data).unwrap()
}

pub async fn load_sound(file_name: &str) -> MediaSourceStream {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec();

            MediaSourceStream::new(Box::new(std::io::Cursor::new(data)), Default::default())
        } else {
            let path = std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
                .join(file_name);
            dbg!(&path);

            //std::fs::read(path).unwrap()
            MediaSourceStream::new(Box::new(std::fs::File::open(path).unwrap()), Default::default())
        }
    }
}
