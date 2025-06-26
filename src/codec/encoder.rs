use byteorder::{BigEndian, ByteOrder};

use crate::codec::types::MessageHeader;
use crate::config::manager::ConfigManager;
use crate::config::types::{FieldDef, FieldType, BaseFieldDef};
use crate::message::{Message, FieldValue};
use crate::util::{MessageError, MessageResult};
use crate::codec::types::{
    TYPE_PRICE_SCALE,
    TYPE_QUANTITY_SCALE,
    TYPE_AMOUNT_SCALE,
    validate_price, 
    validate_quantity, 
    validate_amount,
    validate_date_format, 
    validate_ntime_format,
};

/// 消息编码器，用于将 Message 对象编码为二进制数据
pub struct MessageEncoder<'a> {
    /// 配置管理器，用于获取消息定义
    config_manager: &'a ConfigManager,
    /// 编码缓冲区
    buffer: Vec<u8>,
}

impl<'a> MessageEncoder<'a> {
    /// 创建一个新的消息编码器
    pub fn new(config_manager: &'a ConfigManager) -> Self {
        Self {
            config_manager,
            buffer: Vec::new(),
        }
    }

    /// 编码消息
    pub fn encode(&mut self, message: &Message) -> MessageResult<Vec<u8>> {
        // 清空缓冲区
        self.buffer.clear();
        
        // 获取消息定义
        let message_def = self.config_manager.get_message_def(message.msg_type)
            .ok_or_else(|| MessageError::UnknownMessageType(message.msg_type))?;

        // 预留消息头部空间
        self.buffer.resize(MessageHeader::SIZE, 0);
        
        // 编码消息字段
        for field_def in &message_def.fields {
            let field_value = message.get_field(&field_def.base.name)
                .cloned()
                .unwrap_or_else(|| self.get_default_value(&field_def.base, Some(field_def)));
            
            self.encode_field(&field_def.base, Some(field_def), &field_value)?;
        }

        // 编码扩展字段
        if message.has_field("BizID") && message_def.extensions.len() > 0 {
            let msg_type = message.msg_type;
            let biz_id = message.get_field("BizID").unwrap().as_u32().unwrap();
            let biz_extension = self.config_manager.get_extension(msg_type, biz_id);
            
            if let Some(biz_extension) = biz_extension {
                for field_def in &biz_extension.fields {
                    let field_value = message.get_field(&field_def.name)
                        .cloned()
                        .unwrap_or_else(|| self.get_default_value(&field_def, None));
                    self.encode_field(&field_def, None, &field_value)?;
                }
            }
        }
        
        // 计算消息体长度
        let body_length = (self.buffer.len() - MessageHeader::SIZE) as u32;
        
        // 编码消息头部
        BigEndian::write_u32(&mut self.buffer[0..4], message.msg_type);
        BigEndian::write_u32(&mut self.buffer[4..8], message.seq_num);
        BigEndian::write_u32(&mut self.buffer[8..12], body_length);
        
        // 计算并添加校验和
        self.append_checksum();
        
        Ok(self.buffer.clone())
    }
    
    /// 获取字段的默认值
    fn get_default_value(&self, base_field_def: &BaseFieldDef, _field_def: Option<&FieldDef>) -> FieldValue {
        match base_field_def.r#type {
            FieldType::U8 => FieldValue::U8(0),
            FieldType::U16 => FieldValue::U16(0),
            FieldType::U32 => FieldValue::U32(0),
            FieldType::U64 => FieldValue::U64(0),
            FieldType::I64 => FieldValue::I64(0),
            FieldType::Price => FieldValue::Float(0.0),
            FieldType::Quantity => FieldValue::Float(0.0),
            FieldType::Amount => FieldValue::Float(0.0),
            FieldType::Date => FieldValue::U32(0),
            FieldType::NTime => FieldValue::U64(0),
            FieldType::Char => {
                let length = base_field_def.length.unwrap_or(1);
                FieldValue::Str(" ".repeat(length))
            },
            FieldType::Array => {
                // 对于数组类型，返回空数组
                FieldValue::Array(Vec::new())
            },
        }
    }
    
    /// 验证FieldValue与FieldType是否匹配
    fn validate_field_type_match(field_type: &FieldType, value: &FieldValue) -> bool {
        match (field_type, value) {
            (FieldType::U8, FieldValue::U8(_)) => true,
            (FieldType::U16, FieldValue::U16(_)) => true,
            (FieldType::U32, FieldValue::U32(_)) => true,
            (FieldType::U64, FieldValue::U64(_)) => true,
            (FieldType::I64, FieldValue::I64(_)) => true,
            (FieldType::Char, FieldValue::Str(_)) => true,
            (FieldType::Price, FieldValue::Float(_)) => true,
            (FieldType::Quantity, FieldValue::Float(_)) => true,
            (FieldType::Amount, FieldValue::Float(_)) => true,
            (FieldType::Date, FieldValue::U32(_)) => true,
            (FieldType::NTime, FieldValue::U64(_)) => true,
            (FieldType::Array, FieldValue::Array(_)) => true,
            _ => false,
        }
    }

    /// 编码字段
    fn encode_field(&mut self, base_field_def: &BaseFieldDef, field_def: Option<&FieldDef>, value: &FieldValue) -> MessageResult<()> {
        // 检查FieldValue类型与FieldDef类型是否一致
        if !Self::validate_field_type_match(&base_field_def.r#type, value) {
            return Err(MessageError::InvalidFieldValue(format!(
                "Field '{}' type mismatch: expected {:?}, got {:?}",
                base_field_def.name,
                base_field_def.r#type,
                std::mem::discriminant(value)
            )));
        }
        match base_field_def.r#type {
            FieldType::U8 => {
                let val = match value {
                    FieldValue::U8(v) => *v,
                    _ => 0,
                };
                self.buffer.push(val);
            },
            FieldType::U16 => {
                let val = match value {
                    FieldValue::U16(v) => *v,
                    _ => 0,
                };
                let mut bytes = [0u8; 2];
                BigEndian::write_u16(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::U32 => {
                let val = match value {
                    FieldValue::U32(v) => *v,
                    _ => 0,
                };
                let mut bytes = [0u8; 4];
                BigEndian::write_u32(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::U64 => {
                let val = match value {
                    FieldValue::U64(v) => *v,
                    _ => 0,
                };
                let mut bytes = [0u8; 8];
                BigEndian::write_u64(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::I64 => {
                let val = match value {
                    FieldValue::I64(v) => *v,
                    _ => 0,
                };
                let mut bytes = [0u8; 8];
                BigEndian::write_i64(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::Char => {
                let length = base_field_def.length.ok_or_else(|| {
                    MessageError::FieldEncodeError(format!("Char field {} missing length", base_field_def.name))
                })?;
                
                let string_val = match value {
                    FieldValue::Str(s) => s.clone(),
                    _ => " ".repeat(length),
                };
                
                // 截断或填充字符串到指定长度
                let mut bytes = string_val.into_bytes();
                bytes.resize(length, b' '); // 用空格填充
                bytes.truncate(length); // 截断到指定长度
                
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::Price => {
                let val = match value {
                    FieldValue::Float(v) => {
                        let encoded_val = (*v * TYPE_PRICE_SCALE) as i64;
                        // 验证Price值范围：转换为i64后不能大于9999999999999
                        if !validate_price(encoded_val) {
                            return Err(MessageError::ValueExceedsRange(
                                format!("Price value {} exceeds maximum allowed value (-99999999.99999, 99999999.99999)", v)
                            ));
                        }
                        encoded_val
                    },
                    _ => 0,
                };
                let mut bytes = [0u8; 8];
                BigEndian::write_i64(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::Quantity => {
                let val = match value {
                    FieldValue::Float(v) => {
                        let encoded_val = (*v * TYPE_QUANTITY_SCALE) as i64;
                        // 验证Quantity值范围：转换为i64后不能大于999999999999999
                        if !validate_quantity(encoded_val) {
                            return Err(MessageError::ValueExceedsRange(
                                format!("Quantity value {} exceeds maximum allowed value (-999999999999.999, 999999999999.999)", v)
                            ));
                        }
                        encoded_val
                    },
                    _ => 0,
                };
                let mut bytes = [0u8; 8];
                BigEndian::write_i64(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::Amount => {
                let val = match value {
                    FieldValue::Float(v) => {
                        let encoded_val = (*v * TYPE_AMOUNT_SCALE) as i64;
                        // 验证Amount值范围：转换为i64后不能大于999999999999999
                        if !validate_amount(encoded_val) {
                            return Err(MessageError::ValueExceedsRange(
                                format!("Amount value {} exceeds maximum allowed value (-9999999999999.99999, 9999999999999.99999)", v)
                            ));
                        }
                        encoded_val
                    },
                    _ => 0,
                };
                let mut bytes = [0u8; 8];
                BigEndian::write_i64(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::Date => {
                let val = match value {
                    FieldValue::U32(v) => {
                        // 验证Date格式 YYYYMMDD
                        if !validate_date_format(*v) {
                            return Err(MessageError::InvalidFieldValue(format!(
                                "Invalid date format: {}. Expected YYYYMMDD format with valid year (0000-9999), month (01-12), and day (01-31)", 
                                v
                            )));
                        }
                        *v
                    },
                    _ => 0,
                };
                let mut bytes = [0u8; 4];
                BigEndian::write_u32(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::NTime => {
                let val = match value {
                    FieldValue::U64(v) => {
                        // 验证NTime格式 HHMMSSsssnnnn
                        if !validate_ntime_format(*v) {
                            return Err(MessageError::InvalidFieldValue(format!(
                                "Invalid ntime format: {}. Expected HHMMSSsssnnnn format with valid hour (00-23), minute (00-59), second (00-59), millisecond (000-999), and hundred nanosecond (0000-9999)", 
                                v
                            )));
                        }
                        *v
                    },
                    _ => 0,
                };
                let mut bytes = [0u8; 8];
                BigEndian::write_u64(&mut bytes, val);
                self.buffer.extend_from_slice(&bytes);
            },
            FieldType::Array => {
                // 如果是数组类型，需要完整的字段定义
                let field_def = field_def.ok_or_else(|| {
                    MessageError::ArrayElementEncodeError(format!("Array field {} missing field definition", base_field_def.name))
                })?;
                self.encode_array(field_def, value)?;
            },
        }
        Ok(())
    }
    
    /// 编码数组字段
    fn encode_array(&mut self, field_def: &FieldDef, value: &FieldValue) -> MessageResult<()> {
        // 获取数组长度字段定义
        let length_field_def = field_def.length_field.as_ref().ok_or_else(|| {
            MessageError::ArrayCountEncodeError(format!("Array field {} missing length field", field_def.base.name))
        })?;
        
        // 获取数组元素结构定义
        let struct_def = field_def.r#struct.as_ref().ok_or_else(|| {
            MessageError::ArrayElementEncodeError(format!("Array field {} missing struct definition", field_def.base.name))
        })?;
        
        let array_elements = match value {
            FieldValue::Array(elements) => elements,
            _ => {
                // 如果不是数组类型，使用空数组
                &Vec::new()
            }
        };
        
        // 编码数组长度
        let length = array_elements.len();
        let length_value = match length_field_def.r#type {
            FieldType::U8 => FieldValue::U8(length as u8),
            FieldType::U16 => FieldValue::U16(length as u16),
            FieldType::U32 => FieldValue::U32(length as u32),
            _ => return Err(MessageError::InvalidArrayCountType),
        };
        
        self.encode_field(length_field_def, None, &length_value)?;
        
        // 编码数组元素
        for element in array_elements {
            // 为每个结构字段编码
            for (i, field) in struct_def.fields.iter().enumerate() {
                let field_value = element.get(i)
                    .cloned()
                    .unwrap_or_else(|| self.get_default_value(field, None));
                
                self.encode_field(field, None, &field_value)?;
            }
        }
        
        Ok(())
    }
    
    /// 计算并添加校验和
    fn append_checksum(&mut self) {
        // 计算校验和 - 使用 uint8 累加然后转换为 uint32
        let mut checksum: u8 = 0;
        for byte in &self.buffer {
            checksum = checksum.wrapping_add(*byte);
        }
        
        // 添加校验和到缓冲区
        let mut checksum_bytes = [0u8; 4];
        BigEndian::write_u32(&mut checksum_bytes, checksum as u32);
        self.buffer.extend_from_slice(&checksum_bytes);
    }
    
    /// 获取编码后的数据长度
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::manager::ConfigManager;
    use crate::message::{Message, FieldValue};

    const CONFIG_STR: &str = r#"<messages>
<message type="40" name="Logon">
  <field name="SenderCompID" type="char" length="32" desc="发送方代码"/>
  <field name="TargetCompID" type="char" length="32" desc="接收方代码"/>
  <field name="HeartBtInt" type="u16" desc="心跳间隔（秒）"/>
  <field name="PrtcVersion" type="char" length="8" desc="协议版本"/>
  <field name="TradeDate" type="date" desc="交易日期（YYYYMMDD）"/>
  <field name="QSize" type="u32" desc="客户端最大队列长度"/>
</message>
</messages>"#;

    #[test]
    fn test_encode_logon_message() {
        let mut config_manager = ConfigManager::new();
        config_manager.load_from_str(CONFIG_STR).unwrap();
        
        let mut encoder = MessageEncoder::new(&config_manager);
        
        let mut message = Message::new(40, 1);
        message.add_field("SenderCompID".to_string(), FieldValue::Str("SENDER123".to_string()));
        message.add_field("TargetCompID".to_string(), FieldValue::Str("TARGET456".to_string()));
        message.add_field("HeartBtInt".to_string(), FieldValue::U16(30));
        message.add_field("PrtcVersion".to_string(), FieldValue::Str("1.0".to_string()));
        message.add_field("TradeDate".to_string(), FieldValue::U32(20231201));
        message.add_field("QSize".to_string(), FieldValue::U32(1000));
        
        let encoded = encoder.encode(&message).unwrap();
        
        // 验证编码结果不为空
        assert!(!encoded.is_empty());
        
        // 验证消息头部
        assert_eq!(BigEndian::read_u32(&encoded[0..4]), 40); // msg_type
        assert_eq!(BigEndian::read_u32(&encoded[4..8]), 1);  // seq_num
        
        // 验证消息体长度
        let body_length = BigEndian::read_u32(&encoded[8..12]);
        assert_eq!(encoded.len(), MessageHeader::SIZE + body_length as usize + 4); // +4 for checksum

        println!("{:?}", encoded);
    }
    
    #[test]
    fn test_encode_with_default_values() {
        let mut config_manager = ConfigManager::new();
        config_manager.load_from_str(CONFIG_STR).unwrap();
        
        let mut encoder = MessageEncoder::new(&config_manager);
        
        // 创建一个只有部分字段的消息
        let mut message = Message::new(40, 2);
        message.add_field("SenderCompID".to_string(), FieldValue::Str("TEST".to_string()));
        message.add_field("TradeDate".to_string(), FieldValue::U32(20231201));
        
        let encoded = encoder.encode(&message).unwrap();
        
        // 验证编码结果不为空
        assert!(!encoded.is_empty());
        
        // 验证消息头部
        assert_eq!(BigEndian::read_u32(&encoded[0..4]), 40); // msg_type
        assert_eq!(BigEndian::read_u32(&encoded[4..8]), 2);  // seq_num
    }
}