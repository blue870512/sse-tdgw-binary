mod parser;
pub mod types;
pub mod manager;

pub use parser::XmlConfigParser;
pub use types::{BizExtension, FieldDef, MessageDef, MessageConfig};
pub use manager::ConfigManager;