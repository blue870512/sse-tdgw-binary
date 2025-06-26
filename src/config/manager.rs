use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use quick_xml::de::{from_reader, from_str};

use crate::util::ConfigResult;
use super::types::{BizExtension, MessageDef, MessageConfig};

/// 配置管理器，用于加载和管理消息定义
pub struct ConfigManager {
    messages: HashMap<u32, MessageDef>,
    extentions: HashMap<u32, HashMap<u32, BizExtension>>,
}

impl ConfigManager {
    /// 创建一个新的配置管理器实例
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            extentions: HashMap::new(),
        }
    }
    
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> ConfigResult<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: MessageConfig = from_reader(reader)?;
        self.load_config(config)
    }

    /// 从字符串加载配置
    pub fn load_from_str(&mut self, config_str: &str) -> ConfigResult<()> {
        let config: MessageConfig = from_str(config_str)?;
        self.load_config(config)
    }
    
    /// 加载配置的内部方法，处理共同逻辑
    fn load_config(&mut self, config: MessageConfig) -> ConfigResult<()> {
        // 加载消息定义
        for message in config.messages {
            if !self.extentions.contains_key(&message.msg_type) {
                self.extentions.insert(message.msg_type, HashMap::new());
            }

            for extension in &message.extensions {
                self.extentions.get_mut(&message.msg_type).unwrap().insert(extension.biz_id, extension.clone());
            }

            self.messages.insert(message.msg_type, message);
        }
        
        Ok(())
    }

    /// 获取指定类型的消息定义
    pub fn get_message_def(&self, msg_type: u32) -> Option<&MessageDef> {
        self.messages.get(&msg_type)
    }

    /// 获取指定消息类型和业务ID的扩展定义
    pub fn get_extension(&self, msg_type: u32, biz_id: u32) -> Option<&BizExtension> {
        self.extentions.get(&msg_type).and_then(|ext| ext.get(&biz_id))
    }
}