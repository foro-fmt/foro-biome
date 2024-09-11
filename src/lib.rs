mod format;

use crate::format::{format, FormatResult};
use foro_plugin_utils::data_json_utils::JsonGetter;
use foro_plugin_utils::foro_plugin_setup;
use serde_json::{json, Value};
use std::path::PathBuf;

pub fn main_with_json(input: Value) -> Value {
    let start = std::time::Instant::now();

    let target = String::get_value(&input, ["target"]).unwrap();
    let target_content = String::get_value(&input, ["target-content"]).unwrap();
    let current_dir = PathBuf::from(String::get_value(&input, ["current-dir"]).unwrap());

    let result = match format(target, target_content, current_dir) {
        Ok(FormatResult::Success { formatted_content }) => {
            json!({
                "format-status": "success",
                "formatted-content": formatted_content,
            })
        }
        Ok(FormatResult::Ignored) => {
            json!({
                "format-status": "ignored",
            })
        }
        Ok(FormatResult::Error { error }) => {
            json!({
                "format-status": "error",
                "format-error": error,
            })
        }
        Err(e) => {
            json!({
                "plugin-panic": e.to_string(),
            })
        }
    };

    println!("time: {:?}", start.elapsed());

    result
}

foro_plugin_setup!(main_with_json);
