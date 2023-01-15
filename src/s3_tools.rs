use aws_config::SdkConfig;
use aws_sdk_s3::types::{ByteStream};
use aws_sdk_s3::{Client};
use std::error::Error;
use std::path::Path;
use aws_config::meta::region::RegionProviderChain;

pub async fn upload_image_to_s3(
    image_key: &str,
    config: SdkConfig,
) -> Result<(), Box<dyn Error>> {

    debug!("Starting file creation");
    let client = Client::new(&config);

    let bucket_name = std::env::var("MEDIA_BUCKET").unwrap();
    let file_path = format!("/tmp/{}", image_key);
    debug!("Obatining image data");
    let body = ByteStream::from_path(Path::new(file_path.as_str())).await;

    debug!("Executing image save");
    let result = client
        .put_object()
        .bucket(bucket_name)
        .key(image_key)
        .body(body.unwrap())
        .send()
        .await;

    match result {
        Ok(_) => {
            debug!("Object created")
        }
        Err(e) => {
            error!("{:?}", e.into_service_error().kind)
        }
    }

    Ok(())
}

pub async fn create_config() -> Option<SdkConfig> {
    trace!("region provider");
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");

    trace!("config creation");
    Some(aws_config::from_env().region(region_provider).load().await)
}
