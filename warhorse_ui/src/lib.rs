pub mod schema;

use std::fs;
use std::path::PathBuf;

pub use serde_json;

/// A macro to generate Rust code from schema files and save it to the output directory.
#[macro_export]
macro_rules! generate_schema_code {
    ({
        schemas: [$($schema:expr),* $(,)?],
        out_dir: $out_dir:expr
    }) => {
        {
            // Watch all .wh files in assets directory
            $(
                println!("cargo:rerun-if-changed=assets/{}", $schema);
            )*

            // Include all schema files
            let code = warhorse_ui::schema::parse::generate_rust_code(&[
                $(include_str!(concat!("assets/", $schema))),*
            ])?;

            // Save the generated code to the output directory
            warhorse_ui::save_generated_code(&code, $out_dir)?;
        }
    };
}

pub fn save_generated_code(
    code: &str,
    out_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the generated directory if it doesn't exist
    let generated_dir = PathBuf::from(out_dir).join("generated");
    fs::create_dir_all(&generated_dir)?;

    // Write the generated code to a file
    let output_file = generated_dir.join("warhorse_ui_schema.rs");
    fs::write(output_file, code)?;
    Ok(())
}

