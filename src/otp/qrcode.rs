use anyhow::{bail, Context};
use image::ColorType;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};

pub fn detect_from_webcam(device_index: i8) -> anyhow::Result<()> {
    if let Some(backend) = nokhwa::native_api_backend() {
        let index = if let Some(camera_info) = nokhwa::query(backend)
            .context("Could not request webcam devices")?
            .into_iter()
            .find(|camera_info| {
                camera_info
                    .index()
                    .as_index()
                    .map(|index| index as i8 == device_index)
                    .unwrap_or_default()
            }) {
            camera_info.index().as_index().unwrap()
        } else {
            bail!("Could not find webcam with index {device_index:?}")
        };
        println!("index: {index}");
        let format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
        // let mut camera = Camera::new(index, format)
        //     .with_context(|| format!("Could not initialize webcam {device_name:?}"))?;
        let mut camera = Camera::new(nokhwa::utils::CameraIndex::Index(index), format).unwrap();
        camera.open_stream().with_context(|| {
            format!("Could not open stream on webcam with index {device_index}")
        })?;
        loop {
            let frame = camera
                .frame()
                .with_context(|| format!("Could not get frame from webcam {device_index:?}"))
                .map_err(|err| {
                    let _ = camera.stop_stream();
                    err
                })?;
            let image_buffer = frame
                .decode_image::<RgbFormat>()
                .with_context(|| format!("Could not decode image from webcam {device_index:?}"))
                .map_err(|err| {
                    let _ = camera.stop_stream();
                    err
                })?;

            image::save_buffer(
                "image.png",
                &image_buffer,
                camera.resolution().width(),
                camera.resolution().height(),
                ColorType::Rgb8,
            )
            .unwrap();
            let image = image::open("image.png")?;
            let image = image.to_luma8();
            let mut img = rqrr::PreparedImage::prepare(image);
            let grids = img.detect_grids();
            grids.iter().for_each(|grid| {
                let (_meta, text) = grid.decode().unwrap();
                println!("{text}");
            });
        }

        Ok(())
    } else {
        bail!("Could not initialize webcam backend")
    }
}
