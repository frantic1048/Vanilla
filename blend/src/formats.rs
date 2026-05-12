mod delimited;
mod json;
mod jsonc;
mod plaintext;
mod toml;

use anyhow::Result;

pub use self::delimited::{
    EqualsRecordLinesRenderer, SpacePairLinesRenderer, SpaceRecordLinesRenderer,
};
pub use self::json::JsonRenderer;
pub use self::jsonc::JsoncRenderer;
pub use self::plaintext::PlaintextRenderer;
pub use self::toml::TomlRenderer;

use crate::nickel::Format;

/// Trait for rendering config values to a specific format
pub trait FormatRenderer {
    /// Render a JSON value to the target format string
    fn render(&self, value: &serde_json::Value) -> Result<String>;

    /// Parse a string in this format to a JSON value
    fn parse(&self, content: &str) -> Result<serde_json::Value>;
}

/// Get a renderer for the given format
pub fn get_renderer(format: Format) -> Box<dyn FormatRenderer> {
    match format {
        Format::Toml => Box::new(TomlRenderer),
        Format::Json => Box::new(JsonRenderer),
        Format::Yaml => Box::new(JsonRenderer), // YAML uses JSON-compatible structure
        Format::SpacePairLines => Box::new(SpacePairLinesRenderer),
        Format::SpaceRecordLines => Box::new(SpaceRecordLinesRenderer),
        Format::EqualsRecordLines => Box::new(EqualsRecordLinesRenderer),
        Format::Jsonc => Box::new(JsoncRenderer),
        Format::Plaintext => Box::new(PlaintextRenderer),
    }
}
