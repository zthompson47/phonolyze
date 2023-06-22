use anyhow::Result;
use image::DynamicImage;
use symphonia::core::io::MediaSourceStream;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> Result<reqwest::Url> {
    let window = web_sys::window().unwrap();
    let origin = window.location().origin().unwrap();
    let base = reqwest::Url::parse(&origin)?;

    Ok(base.join(file_name)?)
}

pub async fn load_image(file_name: &str) -> Result<DynamicImage> {
    let data = {
        #[cfg(target_arch = "wasm32")]
        {
            let url = format_url(file_name)?;

            reqwest::get(url).await?.bytes().await?.to_vec()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = std::path::Path::new("www").join(file_name);

            std::fs::read(path)?
        }
    };

    Ok(image::load_from_memory(&data)?)
}

pub async fn load_sound(file_name: &str) -> Result<MediaSourceStream> {
    let cursor = Box::new({
        #[cfg(target_arch = "wasm32")]
        {
            let url = format_url(file_name)?;
            let data = reqwest::get(url).await?.bytes().await?.to_vec();

            std::io::Cursor::new(data)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = std::path::Path::new("www").join(file_name);

            std::fs::File::open(path)?
        }
    });

    Ok(MediaSourceStream::new(cursor, Default::default()))
}
