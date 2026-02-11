mod delimited;
mod json;
mod plaintext;
mod toml;

use anyhow::Result;

pub use self::delimited::{
    EqualDelimitedRecordRenderer, SpaceDelimitedPairsRenderer, SpaceDelimitedRecordRenderer,
};
pub use self::json::JsonRenderer;
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
        Format::SpaceDelimitedPairs => Box::new(SpaceDelimitedPairsRenderer),
        Format::SpaceDelimitedRecord => Box::new(SpaceDelimitedRecordRenderer),
        Format::EqualDelimitedRecord => Box::new(EqualDelimitedRecordRenderer),
        Format::Plaintext => Box::new(PlaintextRenderer),
    }
}
