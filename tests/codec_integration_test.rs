use std::collections::HashMap;
use indexmap::IndexMap;
use sse_tdgw_binary::codec::encoder::MessageEncoder;
use sse_tdgw_binary::codec::decoder::MessageDecoder;
use sse_tdgw_binary::config::manager::ConfigManager;
use sse_tdgw_binary::config::types::{MessageDef, FieldDef, BaseFieldDef, FieldType};
use sse_tdgw_binary::message::{Message, FieldValue};

/// 编解码集成测试
/// 测试所有字段类型的编码和解码，确保往返一致性
#[cfg(test)]
mod codec_tests {
    use super::*;

    /// 创建测试用的配置管理器，包含所有字段类型
    fn create_test_config_manager() -> ConfigManager {
        let mut config_manager = ConfigManager::new();
        
        // 手动插入消息定义到配置管理器
        // 注意：这里我们需要访问ConfigManager的私有字段，可能需要添加一个测试用的方法
        // 暂时使用load_from_str方法
        let config_xml = r#"
        <messages>
            <message type="1001" name="TestMessage">
                <field name="field_u8" type="u8" desc="U8字段"/>
                <field name="field_u16" type="u16" desc="U16字段"/>
                <field name="field_u32" type="u32" desc="U32字段"/>
                <field name="field_u64" type="u64" desc="U64字段"/>
                <field name="field_i64" type="i64" desc="I64字段"/>
                <field name="field_char" type="char" length="10" desc="Char字段"/>
                <field name="field_price" type="price" desc="Price字段"/>
                <field name="field_quantity" type="quantity" desc="Quantity字段"/>
                <field name="field_amount" type="amount" desc="Amount字段"/>
                <field name="field_date" type="date" desc="Date字段"/>
                <field name="field_ntime" type="ntime" desc="NTime字段"/>
            </message>
        </messages>
        "#;
        
        config_manager.load_from_str(config_xml).expect("Failed to load test config");
        config_manager
    }

    /// 创建测试消息，包含所有字段类型的示例值
    fn create_test_message() -> Message {
        let mut message = Message::new(1001, 12345);
        
        // 添加各种类型的字段值
        message.add_field("field_u8".to_string(), FieldValue::U8(255));
        message.add_field("field_u16".to_string(), FieldValue::U16(65535));
        message.add_field("field_u32".to_string(), FieldValue::U32(4294967295));
        message.add_field("field_u64".to_string(), FieldValue::U64(18446744073709551615));
        message.add_field("field_i64".to_string(), FieldValue::I64(-9223372036854775808));
        message.add_field("field_char".to_string(), FieldValue::Str("HELLO".to_string()));
        message.add_field("field_price".to_string(), FieldValue::Float(99999999.99999));
        message.add_field("field_quantity".to_string(), FieldValue::Float(999999999999.999));
        message.add_field("field_amount".to_string(), FieldValue::Float(9999999999998.99999));
        message.add_field("field_date".to_string(), FieldValue::U32(20231225)); // 日期：2023年12月25日
        message.add_field("field_ntime".to_string(), FieldValue::U64(1234567891234)); // 时间：12:34:56.789.1234
        
        message
    }

    /// 测试基本编解码往返
    #[test]
    fn test_encode_decode_roundtrip() {
        let config_manager = create_test_config_manager();
        let original_message = create_test_message();
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message)
            .expect("Failed to encode message");
        
        println!("Encoded data length: {} bytes", encoded_data.len());
        println!("Encoded data (hex): {}", hex::encode(&encoded_data));
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode message");
        
        // 验证消息头
        assert_eq!(decoded_message.msg_type, original_message.msg_type, "Message type mismatch");
        assert_eq!(decoded_message.seq_num, original_message.seq_num, "Sequence number mismatch");
        
        // 验证字段数量
        assert_eq!(decoded_message.field_count(), original_message.field_count(), "Field count mismatch");
        
        // 验证每个字段
        for (field_name, original_value) in &original_message.fields {
            let decoded_value = decoded_message.get_field(field_name)
                .expect(&format!("Field '{}' not found in decoded message", field_name));
            
            assert_eq!(decoded_value, original_value, 
                "Field '{}' value mismatch: expected {:?}, got {:?}", 
                field_name, original_value, decoded_value);
        }
        
        println!("✓ All fields match between original and decoded messages");
    }

    /// 测试边界值
    #[test]
    fn test_boundary_values() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(1001, 0);
        
        // 测试各类型的边界值
        message.add_field("field_u8".to_string(), FieldValue::U8(0));
        message.add_field("field_u16".to_string(), FieldValue::U16(0));
        message.add_field("field_u32".to_string(), FieldValue::U32(0));
        message.add_field("field_u64".to_string(), FieldValue::U64(0));
        message.add_field("field_i64".to_string(), FieldValue::I64(0));
        message.add_field("field_char".to_string(), FieldValue::Str("".to_string()));
        message.add_field("field_price".to_string(), FieldValue::Float(-99999999.99999));
        message.add_field("field_quantity".to_string(), FieldValue::Float(-999999999999.999));
        message.add_field("field_amount".to_string(), FieldValue::Float(-9999999999998.99999));
        message.add_field("field_date".to_string(), FieldValue::U32(10101)); // 最小有效日期
        message.add_field("field_ntime".to_string(), FieldValue::U64(0)); // 最小时间
        
        // 编解码测试
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message).expect("Failed to encode boundary values");
        
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode().expect("Failed to decode boundary values");
        
        // 验证
        assert_eq!(decoded_message.msg_type, message.msg_type);
        assert_eq!(decoded_message.field_count(), message.field_count());
        
        println!("✓ Boundary values test passed");
    }

    /// 测试最大值
    #[test]
    fn test_maximum_values() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(1001, u32::MAX);
        
        // 测试各类型的最大值
        message.add_field("field_u8".to_string(), FieldValue::U8(u8::MAX));
        message.add_field("field_u16".to_string(), FieldValue::U16(u16::MAX));
        message.add_field("field_u32".to_string(), FieldValue::U32(u32::MAX));
        message.add_field("field_u64".to_string(), FieldValue::U64(u64::MAX));
        message.add_field("field_i64".to_string(), FieldValue::I64(i64::MAX));
        message.add_field("field_char".to_string(), FieldValue::Str("1234567890".to_string())); // 10字符

        message.add_field("field_price".to_string(), FieldValue::Float(99999999.99999));
        message.add_field("field_quantity".to_string(), FieldValue::Float(999999999999.999));
        message.add_field("field_amount".to_string(), FieldValue::Float(9999999999999.98999));
        message.add_field("field_date".to_string(), FieldValue::U32(99991231)); // 最大日期
        message.add_field("field_ntime".to_string(), FieldValue::U64(2359599999999)); // 最大时间
        
        // 编解码测试
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message).expect("Failed to encode maximum values");
        
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode().expect("Failed to decode maximum values");
        
        // 验证
        assert_eq!(decoded_message.msg_type, message.msg_type);
        assert_eq!(decoded_message.field_count(), message.field_count());
        
        println!("✓ Maximum values test passed");
    }

    /// 测试负数值
    #[test]
    fn test_negative_values() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(1001, 999);
        
        // 测试负数值（仅适用于I64类型）
        message.add_field("field_u8".to_string(), FieldValue::U8(128));
        message.add_field("field_u16".to_string(), FieldValue::U16(32768));
        message.add_field("field_u32".to_string(), FieldValue::U32(2147483648));
        message.add_field("field_u64".to_string(), FieldValue::U64(9223372036854775808));
        message.add_field("field_i64".to_string(), FieldValue::I64(-1234567890));
        message.add_field("field_char".to_string(), FieldValue::Str("TEST".to_string()));
        message.add_field("field_price".to_string(), FieldValue::Float(-1.0));
        message.add_field("field_quantity".to_string(), FieldValue::Float(-1.0));
        message.add_field("field_amount".to_string(), FieldValue::Float(-1.0));
        message.add_field("field_date".to_string(), FieldValue::U32(20200229)); // 闰年日期
        message.add_field("field_ntime".to_string(), FieldValue::U64(12000000000000)); // 中午时间
        
        // 编解码测试
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message).expect("Failed to encode negative values");
        
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode().expect("Failed to decode negative values");
        
        // 验证
        assert_eq!(decoded_message.msg_type, message.msg_type);
        assert_eq!(decoded_message.field_count(), message.field_count());
        
        println!("✓ Negative values test passed");
    }

    /// 测试特殊字符串值
    #[test]
    fn test_special_string_values() {
        let config_manager = create_test_config_manager();
        let mut message = Message::new(1001, 777);
        
        // 测试特殊字符串值
        message.add_field("field_u8".to_string(), FieldValue::U8(42));
        message.add_field("field_u16".to_string(), FieldValue::U16(1024));
        message.add_field("field_u32".to_string(), FieldValue::U32(1048576));
        message.add_field("field_u64".to_string(), FieldValue::U64(1073741824));
        message.add_field("field_i64".to_string(), FieldValue::I64(1099511627776));
        message.add_field("field_char".to_string(), FieldValue::Str("ABC123".to_string())); // 混合字符
        message.add_field("field_price".to_string(), FieldValue::Float(1000.50));
        message.add_field("field_quantity".to_string(), FieldValue::Float(1500.000));
        message.add_field("field_amount".to_string(), FieldValue::Float(25000.75000));
        message.add_field("field_date".to_string(), FieldValue::U32(20231201)); // 12月1日
        message.add_field("field_ntime".to_string(), FieldValue::U64(930451234567)); // 09:30:45.123.4567
        
        // 编解码测试
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message).expect("Failed to encode special values");
        
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode().expect("Failed to decode special values");
        
        // 验证
        assert_eq!(decoded_message.msg_type, message.msg_type);
        assert_eq!(decoded_message.field_count(), message.field_count());
        
        // 特别验证字符串字段
        let decoded_char = decoded_message.get_field("field_char").unwrap();
        if let FieldValue::Str(s) = decoded_char {
            assert!(s.starts_with("ABC123"), "String field should start with 'ABC123'");
        } else {
            panic!("Expected string field");
        }
        
        println!("✓ Special string values test passed");
    }

    /// 性能测试：大量消息编解码
    #[test]
    fn test_performance() {
        let config_manager = create_test_config_manager();
        let test_message = create_test_message();
        
        let start_time = std::time::Instant::now();
        let iterations = 1000;
        
        for i in 0..iterations {
            let mut message = test_message.clone();
            message.seq_num = i;
            
            // 编码
            let mut encoder = MessageEncoder::new(&config_manager);
            let encoded_data = encoder.encode(&message)
                .expect(&format!("Failed to encode message {}", i));
            
            // 解码
            let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
            let decoded_message = decoder.decode()
                .expect(&format!("Failed to decode message {}", i));
            
            // 快速验证
            assert_eq!(decoded_message.msg_type, message.msg_type);
            assert_eq!(decoded_message.seq_num, message.seq_num);
        }
        
        let elapsed = start_time.elapsed();
        println!("✓ Performance test: {} iterations in {:?} ({:.2} msg/sec)", 
                iterations, elapsed, iterations as f64 / elapsed.as_secs_f64());
    }
}