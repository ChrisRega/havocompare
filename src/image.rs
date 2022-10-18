use crate::report;
use image_compare::ToColorMap;
use schemars_derive::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{error, info};

#[derive(JsonSchema, Deserialize, Serialize, Debug)]
pub struct ImageCompareConfig {
    threshold: f64,
}

pub fn compare_paths<P: AsRef<Path>>(
    nominal_path: P,
    actual_path: P,
    config: &ImageCompareConfig,
    rule_name: &str,
) -> report::FileCompareResult {
    let mut diffs: Vec<String> = Vec::new();
    let nominal = image::open(nominal_path.as_ref())
        .expect("Could not load nominal image file")
        .into_rgb8();
    let actual = image::open(actual_path.as_ref())
        .expect("Could not load actual image file")
        .into_rgb8();

    let result = image_compare::rgb_hybrid_compare(&nominal, &actual)
        .expect("Nominal and actual had different dimensions");
    if result.score < config.threshold {
        let color_map = result.image.to_color_map();
        let nominal_file_name = nominal_path
            .as_ref()
            .file_name()
            .expect("Could not extract file_name from nominal path");
        let out_path = (nominal_file_name.to_string_lossy() + "diff_image.png").to_string();
        info!("Writing diff image to: {}", out_path.as_str());
        color_map
            .save(PathBuf::from(&out_path))
            .expect("Could not write diff-image");

        let error_message = format!(
            "Diff for image {} was not met, expected {}, found {}",
            nominal_path.as_ref().to_string_lossy(),
            config.threshold,
            result.score
        );

        error!("{}", &error_message);

        diffs.push(error_message);
        diffs.push(out_path);
    }

    report::write_image_detail(
        nominal_path.as_ref(),
        actual_path.as_ref(),
        &diffs,
        rule_name,
    )
}
