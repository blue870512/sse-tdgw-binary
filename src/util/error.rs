use thiserror::Error;

// 编解码相关错误
#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Invalid field type")]
    InvalidFieldType,
    
    #[error("Buffer too small")]
    BufferTooSmall,
    
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Checksum verification failed")]
    ChecksumError,
}

// 配置相关错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("XML parsing error: {0}")]
    XmlError(#[from] quick_xml::DeError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Message type {0} not found")]
    MessageNotFound(u32),
    
    #[error("Invalid message type")]
    InvalidMessageType,
    
    #[error("Invalid message name")]
    InvalidMessageName,
    
    #[error("Invalid field name")]
    InvalidFieldName,
    
    #[error("Invalid array definition")]
    InvalidArrayDefinition,
    
    #[error("Invalid array length field")]
    InvalidArrayLengthField,
    
    #[error("Invalid array structure")]
    InvalidArrayStructure,
    
    #[error("Invalid business ID")]
    InvalidBizId,
    
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

// 消息解析相关错误
#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Data too short for header")]
    HeaderTooShort,
    
    #[error("Data too short for message body and checksum")]
    BodyTooShort,
    
    #[error("Unknown message type: {0}")]
    UnknownMessageType(u32),
    
    #[error("Failed to decode field: {0}")]
    FieldDecodeError(String),
    
    #[error("Failed to decode array count field: {0}")]
    ArrayCountDecodeError(String),
    
    #[error("Array count field must be U32 type")]
    InvalidArrayCountType,
    
    #[error("Failed to decode array element field: {0}")]
    ArrayElementDecodeError(String),
    
    #[error("Failed to encode field: {0}")]
    FieldEncodeError(String),
    
    #[error("Failed to encode array count field: {0}")]
    ArrayCountEncodeError(String),
    
    #[error("Failed to encode array element field: {0}")]
    ArrayElementEncodeError(String),
    
    #[error("Invalid field value: {0}")]
    InvalidFieldValue(String),
    
    #[error("Codec error: {0}")]
    CodecError(#[from] CodecError),
    
    #[error("Config error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Checksum error")]
    ChecksumError,

    #[error("Value exceeds range: {0}")]
    ValueExceedsRange(String),

    #[error("Unknown business extension: {0}")]
    UnknownBizExtension(u32),
}

// 类型别名
pub type CodecResult<T> = std::result::Result<T, CodecError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type MessageResult<T> = std::result::Result<T, MessageError>;