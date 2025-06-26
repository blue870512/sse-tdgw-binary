use sse_tdgw_binary::codec::encoder::MessageEncoder;
use sse_tdgw_binary::codec::decoder::MessageDecoder;
use sse_tdgw_binary::config::manager::ConfigManager;
use sse_tdgw_binary::config::types::{MessageDef, FieldDef, BaseFieldDef, FieldType};
use sse_tdgw_binary::message::{Message, FieldValue};

/// 数组类型编解码测试
/// 专门测试数组字段的编码和解码功能
#[cfg(test)]
mod array_codec_tests {
    use super::*;

    /// 创建包含数组字段的测试配置管理器
    fn create_array_test_config_manager() -> ConfigManager {
        let mut config_manager = ConfigManager::new();
        
        // 创建包含数组字段的测试消息定义
        let config_xml = r#"
        <messages>
            <message type="2001" name="ArrayTestMessage">
                <field name="item" type="u16" desc="数组长度"/>
                <field name="simple_array" type="array" desc="简单数组">
                    <length_field name="NoItems" type="u16" desc="数组项个数"/>
                    <struct>
                        <field name="item_id" type="u32" desc="项目ID"/>
                        <field name="item_name" type="char" length="8" desc="项目名称"/>
                        <field name="item_price" type="price" desc="项目价格"/>
                    </struct>
                </field>
                <field name="nested_array" type="array" desc="嵌套数组">
                    <length_field name="NoGroups" type="u16" desc="嵌套数组项个数"/>
                    <struct>
                        <field name="group_id" type="u16" desc="组ID"/>
                        <field name="sub_count" type="u16" desc="子项数量"/>
                        <field name="sub_items" type="quantity" desc="子项数组"/>
                    </struct>
                </field>
            </message>
        </messages>
        "#;
        
        config_manager.load_from_str(config_xml).expect("Failed to load array test config");
        config_manager
    }

    /// 创建包含数组数据的测试消息
    fn create_array_test_message() -> Message {
        let mut message = Message::new(2001, 54321);
        
        // 简单数组数据
        let simple_array_data = vec![
            vec![
                FieldValue::U32(1001),
                FieldValue::Str("ITEM001".to_string()),
                FieldValue::Float(123.45000), // 123.45000
            ],
            vec![
                FieldValue::U32(1002),
                FieldValue::Str("ITEM002".to_string()),
                FieldValue::Float(678.90000), // 678.90000
            ],
            vec![
                FieldValue::U32(1003),
                FieldValue::Str("ITEM003".to_string()),
                FieldValue::Float(999.99000), // 999.99000
            ],
        ];
        
        // 嵌套数组数据
        let nested_array_data = vec![
            vec![
                FieldValue::U16(100),
                FieldValue::U16(2), // sub_count
                FieldValue::Float(1234.56000), // 1234.56000
            ],
            vec![
                FieldValue::U16(200),
                FieldValue::U16(3), // sub_count
                FieldValue::Float(7890.12000), // 7890.12000
            ],
        ];
        
        // 添加字段
        message.add_field("item".to_string(), FieldValue::U16(3));
        message.add_field("simple_array".to_string(), FieldValue::Array(simple_array_data));
        message.add_field("nested_array".to_string(), FieldValue::Array(nested_array_data));
        
        message
    }

    /// 测试简单数组编解码
    #[test]
    fn test_simple_array_encode_decode() {
        let config_manager = create_array_test_config_manager();
        let original_message = create_array_test_message();
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message)
            .expect("Failed to encode array message");
        
        println!("Array message encoded data length: {} bytes", encoded_data.len());
        // 将二进制数据转换为十六进制字符串并打印
        println!("Array message encoded data (hex): {}", encoded_data.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode array message");
        
        // 验证消息头
        assert_eq!(decoded_message.msg_type, original_message.msg_type, "Message type mismatch");
        assert_eq!(decoded_message.seq_num, original_message.seq_num, "Sequence number mismatch");
        
        // 验证字段数量
        assert_eq!(decoded_message.field_count(), original_message.field_count(), "Field count mismatch");
        
        // 验证数组长度字段
        let item = decoded_message.get_field("item").unwrap();
        assert_eq!(item, &FieldValue::U16(3), "Item count mismatch");
        
        // 验证简单数组
        let simple_array = decoded_message.get_field("simple_array").unwrap();
        if let FieldValue::Array(array_data) = simple_array {
            assert_eq!(array_data.len(), 3, "Simple array length mismatch");
            
            // 验证第一个数组项
            assert_eq!(array_data[0][0], FieldValue::U32(1001), "First item ID mismatch");
            if let FieldValue::Str(name) = &array_data[0][1] {
                assert!(name.starts_with("ITEM001"), "First item name mismatch");
            } else {
                panic!("Expected string field for item name");
            }
            assert_eq!(array_data[0][2], FieldValue::Float(123.45000), "First item price mismatch");
        } else {
            panic!("Expected array field for simple_array");
        }
        
        println!("✓ Simple array encode/decode test passed");
    }

    /// 测试空数组
    #[test]
    fn test_empty_array_encode_decode() {
        let config_manager = create_array_test_config_manager();
        let mut message = Message::new(2001, 99999);
        
        // 创建空数组消息
        message.add_field("item".to_string(), FieldValue::U16(0));
        message.add_field("simple_array".to_string(), FieldValue::Array(vec![]));
        message.add_field("nested_array".to_string(), FieldValue::Array(vec![]));
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message)
            .expect("Failed to encode empty array message");
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode empty array message");
        
        // 验证
        assert_eq!(decoded_message.msg_type, message.msg_type);
        assert_eq!(decoded_message.seq_num, message.seq_num);
        
        // 验证空数组
        let simple_array = decoded_message.get_field("simple_array").unwrap();
        if let FieldValue::Array(array_data) = simple_array {
            assert_eq!(array_data.len(), 0, "Simple array should be empty");
        } else {
            panic!("Expected array field for simple_array");
        }
        
        let nested_array = decoded_message.get_field("nested_array").unwrap();
        if let FieldValue::Array(array_data) = nested_array {
            assert_eq!(array_data.len(), 0, "Nested array should be empty");
        } else {
            panic!("Expected array field for nested_array");
        }
        
        println!("✓ Empty array encode/decode test passed");
    }

    /// 测试大数组
    #[test]
    fn test_large_array_encode_decode() {
        let config_manager = create_array_test_config_manager();
        let mut message = Message::new(2001, 88888);
        
        // 创建大数组（100个元素）
        let mut large_array_data = Vec::new();
        for i in 0..100 {
            large_array_data.push(vec![
                FieldValue::U32(i + 1000),
                FieldValue::Str(format!("ITEM{:03}", i)),
                FieldValue::Float((i + 1) as f64), // 价格递增
            ]);
        }
        
        message.add_field("item".to_string(), FieldValue::U16(100));
        message.add_field("simple_array".to_string(), FieldValue::Array(large_array_data));
        message.add_field("nested_array".to_string(), FieldValue::Array(vec![]));
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message)
            .expect("Failed to encode large array message");
        
        println!("Large array message encoded data length: {} bytes", encoded_data.len());
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode large array message");
        
        // 验证
        assert_eq!(decoded_message.msg_type, message.msg_type);
        assert_eq!(decoded_message.seq_num, message.seq_num);
        
        // 验证大数组
        let simple_array = decoded_message.get_field("simple_array").unwrap();
        if let FieldValue::Array(array_data) = simple_array {
            assert_eq!(array_data.len(), 100, "Large array length mismatch");
            
            // 验证几个关键元素
            assert_eq!(array_data[0][0], FieldValue::U32(1000), "First large item ID mismatch");
            assert_eq!(array_data[99][0], FieldValue::U32(1099), "Last large item ID mismatch");
            assert_eq!(array_data[50][2], FieldValue::Float(51.0), "Middle large item price mismatch");
        } else {
            panic!("Expected array field for simple_array");
        }
        
        println!("✓ Large array encode/decode test passed");
    }

    /// 测试数组字段类型一致性
    #[test]
    fn test_array_field_type_consistency() {
        let config_manager = create_array_test_config_manager();
        let mut message = Message::new(2001, 77777);
        
        // 创建包含各种数据类型的数组
        let mixed_array_data = vec![
            vec![
                FieldValue::U32(u32::MAX),
                FieldValue::Str("MAX_VAL".to_string()),
                FieldValue::Float(9999999.99999),
            ],
            vec![
                FieldValue::U32(0),
                FieldValue::Str("MIN_VAL".to_string()),
                FieldValue::Float(-9999999.99999),
            ],
        ];
        
        message.add_field("item".to_string(), FieldValue::U16(2));
        message.add_field("simple_array".to_string(), FieldValue::Array(mixed_array_data));
        message.add_field("nested_array".to_string(), FieldValue::Array(vec![]));
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message)
            .expect("Failed to encode mixed array message");
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode mixed array message");
        
        // 验证极值
        let simple_array = decoded_message.get_field("simple_array").unwrap();
        if let FieldValue::Array(array_data) = simple_array {
            assert_eq!(array_data[0][0], FieldValue::U32(u32::MAX), "Max U32 value mismatch");
            assert_eq!(array_data[0][2], FieldValue::Float(9999999.99999), "Max I64 value mismatch");
            assert_eq!(array_data[1][0], FieldValue::U32(0), "Min U32 value mismatch");
            assert_eq!(array_data[1][2], FieldValue::Float(-9999999.99999), "Min I64 value mismatch");
        } else {
            panic!("Expected array field for simple_array");
        }
        
        println!("✓ Array field type consistency test passed");
    }
}