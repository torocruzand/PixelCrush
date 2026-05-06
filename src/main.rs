mod app;
mod image_processor;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    let app = app::App::new()?;
    app.run()
}
