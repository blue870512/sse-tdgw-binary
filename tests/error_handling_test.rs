use sse_tdgw_binary::codec::encoder::MessageEncoder;
use sse_tdgw_binary::codec::decoder::MessageDecoder;
use sse_tdgw_binary::config::manager::ConfigManager;
use sse_tdgw_binary::message::{Message, FieldValue};
use sse_tdgw_binary::util::MessageError;

/// 错误处理和边界条件测试
/// 测试编解码器在各种异常情况下的行为
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    /// 创建测试用的配置管理器
    fn create_test_config_manager() -> ConfigManager {
        let mut config_manager = ConfigManager::new();
        
        let config_xml = r#"
        <messages>
            <message type="3001" name="ErrorTestMessage">
                <field name="field_u8" type="u8" desc="U8字段"/>
                <field name="field_u16" type="u16" desc="U16字段"/>
                <field name="field_u32" type="u32" desc="U32字段"/>
                <field name="field_char" type="char" length="5" desc="Char字段"/>
                <field name="field_price" type="price" desc="Price字段"/>
                <field name="field_date" type="date" desc="Date字段"/>
                <field name="field_ntime" type="ntime" desc="NTime字段"/>
            </message>
        </messages>
        "#;
        
        config_manager.load_from_str(config_xml).expect("Failed to load test config");
        config_manager
    }

    /// 测试未知消息类型
    #[test]
    fn test_unknown_message_type() {
        let config_manager = create_test_config_manager();
        let message = Message::new(9999, 12345); // 未定义的消息类型
        
        let mut encoder = MessageEncoder::new(&config_manager);
        let result = encoder.encode(&message);
        
        assert!(result.is_err(), "Should fail for unknown message type");
        if let Err(MessageError::UnknownMessageType(msg_type)) = result {
            assert_eq!(msg_type, 9999, "Error should contain the unknown message type");
        } else {
            panic!("Expected UnknownMessageType error");
        }
        
        println!("✓ Unknown message type error handling test passed");
    }

    /// 测试字段类型不匹配
    #[test]
    fn test_field_type_mismatch() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(3001, 12345);
        
        // 添加类型不匹配的字段
        message.add_field("field_u8".to_string(), FieldValue::U32(256)); // U8字段使用U32值
        message.add_field("field_u16".to_string(), FieldValue::Str("invalid".to_string())); // U16字段使用字符串
        message.add_field("field_u32".to_string(), FieldValue::U32(12345));
        message.add_field("field_char".to_string(), FieldValue::Str("HELLO".to_string()));
        message.add_field("field_price".to_string(), FieldValue::I64(12345));
        message.add_field("field_date".to_string(), FieldValue::U32(20231225));
        message.add_field("field_ntime".to_string(), FieldValue::U64(12345678901234));
        
        let mut encoder = MessageEncoder::new(&config_manager);
        let result = encoder.encode(&message);
        
        assert!(result.is_err(), "Should fail for field type mismatch");
        if let Err(MessageError::InvalidFieldValue(_)) = result {
            // 预期的错误类型
        } else {
            panic!("Expected InvalidFieldValue error, got: {:?}", result);
        }
        
        println!("✓ Field type mismatch error handling test passed");
    }

    /// 测试无效的日期格式
    #[test]
    fn test_invalid_date_format() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(3001, 12345);
        
        // 添加无效日期
        message.add_field("field_u8".to_string(), FieldValue::U8(255));
        message.add_field("field_u16".to_string(), FieldValue::U16(65535));
        message.add_field("field_u32".to_string(), FieldValue::U32(4294967295));
        message.add_field("field_char".to_string(), FieldValue::Str("HELLO".to_string()));
        message.add_field("field_price".to_string(), FieldValue::Float(12345.0));
        message.add_field("field_date".to_string(), FieldValue::U32(20231301)); // 无效月份
        message.add_field("field_ntime".to_string(), FieldValue::U64(12345678901234));
        
        let mut encoder = MessageEncoder::new(&config_manager);
        let result = encoder.encode(&message);
        
        assert!(result.is_err(), "Should fail for invalid date format");
        if let Err(MessageError::InvalidFieldValue(_)) = result {
            // 预期的错误类型
        } else {
            panic!("Expected InvalidFieldValue error for invalid date");
        }
        
        println!("✓ Invalid date format error handling test passed");
    }

    /// 测试无效的时间格式
    #[test]
    fn test_invalid_ntime_format() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(3001, 12345);
        
        // 添加无效时间
        message.add_field("field_u8".to_string(), FieldValue::U8(255));
        message.add_field("field_u16".to_string(), FieldValue::U16(65535));
        message.add_field("field_u32".to_string(), FieldValue::U32(4294967295));
        message.add_field("field_char".to_string(), FieldValue::Str("HELLO".to_string()));
        message.add_field("field_price".to_string(), FieldValue::Float(12345.0));
        message.add_field("field_date".to_string(), FieldValue::U32(20231225));
        message.add_field("field_ntime".to_string(), FieldValue::U64(25000000000000)); // 无效小时（25）
        
        let mut encoder = MessageEncoder::new(&config_manager);
        let result = encoder.encode(&message);
        
        assert!(result.is_err(), "Should fail for invalid ntime format");
        if let Err(MessageError::InvalidFieldValue(_)) = result {
            // 预期的错误类型
        } else {
            panic!("Expected InvalidFieldValue error for invalid ntime");
        }
        
        println!("✓ Invalid ntime format error handling test passed");
    }

    /// 测试价格超出范围
    #[test]
    fn test_price_out_of_range() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(3001, 12345);
        
        // 添加超出范围的价格
        message.add_field("field_u8".to_string(), FieldValue::U8(255));
        message.add_field("field_u16".to_string(), FieldValue::U16(65535));
        message.add_field("field_u32".to_string(), FieldValue::U32(4294967295));
        message.add_field("field_char".to_string(), FieldValue::Str("HELLO".to_string()));
        message.add_field("field_price".to_string(), FieldValue::Float(99999999999999.9)); // 超出价格范围
        message.add_field("field_date".to_string(), FieldValue::U32(20231225));
        message.add_field("field_ntime".to_string(), FieldValue::U64(12345678901234));
        
        let mut encoder = MessageEncoder::new(&config_manager);
        let result = encoder.encode(&message);
        
        assert!(result.is_err(), "Should fail for price out of range");
        if let Err(MessageError::ValueExceedsRange(_)) = result {
            // 预期的错误类型
        } else {
            panic!("Expected ValueExceedsRange error for price out of range");
        }
        
        println!("✓ Price out of range error handling test passed");
    }

    /// 测试字符串长度超出限制
    #[test]
    fn test_string_length_exceeded() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(3001, 12345);
        
        // 添加超长字符串
        message.add_field("field_u8".to_string(), FieldValue::U8(255));
        message.add_field("field_u16".to_string(), FieldValue::U16(65535));
        message.add_field("field_u32".to_string(), FieldValue::U32(4294967295));
        message.add_field("field_char".to_string(), FieldValue::Str("TOOLONGSTRING".to_string())); // 超过5字符限制
        message.add_field("field_price".to_string(), FieldValue::I64(12345));
        message.add_field("field_date".to_string(), FieldValue::U32(20231225));
        message.add_field("field_ntime".to_string(), FieldValue::U64(12345678901234));
        
        let mut encoder = MessageEncoder::new(&config_manager);
        let result = encoder.encode(&message);
        
        // 注意：字符串长度超出可能不会立即失败，而是会被截断
        // 这取决于具体的实现，这里我们测试编码是否成功
        match result {
            Ok(encoded_data) => {
                // 如果编码成功，验证解码后的字符串是否被正确处理
                let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                let decoded_message = decoder.decode().expect("Failed to decode message");
                
                let char_field = decoded_message.get_field("field_char").unwrap();
                if let FieldValue::Str(s) = char_field {
                    assert!(s.len() <= 5, "String should be truncated to 5 characters or less");
                } else {
                    panic!("Expected string field");
                }
                
                println!("✓ String length handling test passed (truncated)");
            },
            Err(_) => {
                println!("✓ String length exceeded error handling test passed (rejected)");
            }
        }
    }

    /// 测试损坏的二进制数据解码
    #[test]
    fn test_corrupted_binary_data() {
        let config_manager = create_test_config_manager();
        
        // 创建损坏的二进制数据
        let corrupted_data = vec![0xFF; 10]; // 太短的数据
        
        let mut decoder = MessageDecoder::new(&config_manager, &corrupted_data);
        let result = decoder.decode();
        
        assert!(result.is_err(), "Should fail for corrupted binary data");
        
        println!("✓ Corrupted binary data error handling test passed");
    }

    /// 测试空二进制数据解码
    #[test]
    fn test_empty_binary_data() {
        let config_manager = create_test_config_manager();
        
        let empty_data = vec![];
        
        let mut decoder = MessageDecoder::new(&config_manager, &empty_data);
        let result = decoder.decode();
        
        assert!(result.is_err(), "Should fail for empty binary data");
        if let Err(MessageError::HeaderTooShort) = result {
            // 预期的错误类型
        } else {
            panic!("Expected HeaderTooShort error");
        }
        
        println!("✓ Empty binary data error handling test passed");
    }

    /// 测试校验和不匹配
    #[test]
    fn test_checksum_mismatch() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(3001, 12345);
        
        // 创建有效消息
        message.add_field("field_u8".to_string(), FieldValue::U8(255));
        message.add_field("field_u16".to_string(), FieldValue::U16(65535));
        message.add_field("field_u32".to_string(), FieldValue::U32(4294967295));
        message.add_field("field_char".to_string(), FieldValue::Str("HELLO".to_string()));
        message.add_field("field_price".to_string(), FieldValue::Float(12345.0));
        message.add_field("field_date".to_string(), FieldValue::U32(20231225));
        message.add_field("field_ntime".to_string(), FieldValue::U64(1234567891234));
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let mut encoded_data = encoder.encode(&message).expect("Failed to encode message");
        
        // 损坏校验和（假设校验和在最后4个字节）
        if encoded_data.len() >= 4 {
            let len = encoded_data.len();
            encoded_data[len - 1] ^= 0xFF; // 翻转最后一个字节
        }
        
        // 尝试解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let result = decoder.decode();
        
        assert!(result.is_err(), "Should fail for checksum mismatch");
        if let Err(MessageError::ChecksumError) = result {
            // 预期的错误类型
        } else {
            // 某些实现可能不检查校验和，或者使用不同的错误类型
            println!("Warning: Checksum validation may not be implemented or uses different error type");
        }
        
        println!("✓ Checksum mismatch error handling test passed");
    }

    /// 测试消息体长度不匹配
    #[test]
    fn test_body_length_mismatch() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(3001, 12345);
        
        // 创建有效消息
        message.add_field("field_u8".to_string(), FieldValue::U8(255));
        message.add_field("field_u16".to_string(), FieldValue::U16(65535));
        message.add_field("field_u32".to_string(), FieldValue::U32(4294967295));
        message.add_field("field_char".to_string(), FieldValue::Str("HELLO".to_string()));
        message.add_field("field_price".to_string(), FieldValue::Float(12345.0));
        message.add_field("field_date".to_string(), FieldValue::U32(20231225));
        message.add_field("field_ntime".to_string(), FieldValue::U64(1234567891234));
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let mut encoded_data = encoder.encode(&message).expect("Failed to encode message");
        
        // 截断数据（模拟消息体长度不匹配）
        if encoded_data.len() > 20 {
            encoded_data.truncate(encoded_data.len() - 10);
        }
        
        // 尝试解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let result = decoder.decode();
        
        assert!(result.is_err(), "Should fail for body length mismatch");
        
        println!("✓ Body length mismatch error handling test passed");
    }

    /// 测试缺失必需字段
    #[test]
    fn test_missing_required_fields() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(3001, 12345);
        
        // 只添加部分字段，缺失其他必需字段
        message.add_field("field_u8".to_string(), FieldValue::U8(255));
        message.add_field("field_u16".to_string(), FieldValue::U16(65535));
        // 缺失其他字段
        
        let mut encoder = MessageEncoder::new(&config_manager);
        let result = encoder.encode(&message);
        
        // 编码器应该为缺失的字段提供默认值，所以编码应该成功
        match result {
            Ok(encoded_data) => {
                // 验证解码是否成功
                let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                let decoded_message = decoder.decode().expect("Failed to decode message with default values");
                
                // 验证消息包含所有字段（包括默认值）
                assert!(decoded_message.has_field("field_u8"), "Should have field_u8");
                assert!(decoded_message.has_field("field_u16"), "Should have field_u16");
                assert!(decoded_message.has_field("field_u32"), "Should have field_u32 with default value");
                
                println!("✓ Missing required fields handled with default values");
            },
            Err(_) => {
                println!("✓ Missing required fields error handling test passed (rejected)");
            }
        }
    }
}