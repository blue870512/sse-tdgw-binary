use quick_xml::de::from_str;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::types::{MessageConfig, FieldType};
use crate::util::{ConfigError, ConfigResult};

/// XML配置解析器，用于解析消息定义XML文件
pub struct XmlConfigParser;

impl XmlConfigParser {
    /// 从文件加载XML配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> ConfigResult<MessageConfig> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        Self::parse_xml(&content)
    }
    
    /// 解析XML字符串
    pub fn parse_xml(xml: &str) -> ConfigResult<MessageConfig> {
        let config: MessageConfig = from_str(xml)
            .map_err(|e| ConfigError::XmlError(e))?;
        
        // 验证配置
        Self::validate_config(&config)?;
        
        Ok(config)
    }
    
    /// 验证配置的有效性
    fn validate_config(config: &MessageConfig) -> ConfigResult<()> {
        // 检查消息定义
        for message in &config.messages {
            // 检查消息类型是否有效
            if message.msg_type == 0 {
                return Err(ConfigError::InvalidMessageType);
            }
            
            // 检查消息名称是否为空
            if message.name.is_empty() {
                return Err(ConfigError::InvalidMessageName);
            }
            
            // 检查字段定义
            for field in &message.fields {
                // 检查字段名称是否为空
                if field.base.name.is_empty() {
                    return Err(ConfigError::InvalidFieldName);
                }
                
                // 检查数组类型字段
                if field.base.r#type == FieldType::Array {
                    // 数组类型字段必须有length_field和struct定义
                    if field.length_field.is_none() || field.r#struct.is_none() {
                        return Err(ConfigError::InvalidArrayDefinition);
                    }
                    
                    // 检查struct字段列表是否为空
                    if field.r#struct.as_ref().unwrap().fields.is_empty() {
                        return Err(ConfigError::InvalidArrayStructure);
                    }
                    
                    // 检查struct中的每个字段
                    for struct_field in &field.r#struct.as_ref().unwrap().fields {
                        if struct_field.name.is_empty() {
                            return Err(ConfigError::InvalidFieldName);
                        }
                    }
                }
            }
            
            // 检查扩展字段
            for extension in &message.extensions {
                // 检查biz_id是否有效
                if extension.biz_id == 0 {
                    return Err(ConfigError::InvalidBizId);
                }
                
                // 检查扩展字段列表
                for field in &extension.fields {
                    if field.name.is_empty() {
                        return Err(ConfigError::InvalidFieldName);
                    }
                }
            }
        }
        
        Ok(())
    }
}