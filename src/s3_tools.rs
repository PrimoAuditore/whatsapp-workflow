use actix_web::rt::task::spawn_blocking;
use actix_web::web::block;
use actix_web::Handler;
use aws_config::meta::region::RegionProviderChain;
use aws_config::SdkConfig;
use aws_sdk_s3::error::PutObjectError;
use aws_sdk_s3::output::PutObjectOutput;
use aws_sdk_s3::types::{ByteStream, SdkError};
use aws_sdk_s3::{Client, Config};
use std::error::Error;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::path::Path;
use std::sync::{Arc, Once};

pub async fn upload_image(
    image_key: &str,
    config: &Option<SdkConfig>,
) -> Result<(), Box<dyn Error>> {
    let config_updated = config.clone().unwrap();
    debug!("Starting file creation");
    let client = Client::new(&config_updated);

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
