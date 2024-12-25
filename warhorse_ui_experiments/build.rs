use std::error::Error;
use warhorse_ui::generate_schema_code;

fn main() -> Result<(), Box<dyn Error>> {
    generate_schema_code!({
        schemas: [
            "test.wh"
        ],
        out_dir: std::env::var("OUT_DIR")?
    });
    Ok(())
}
