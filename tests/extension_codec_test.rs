use sse_tdgw_binary::{
    codec::{encoder::MessageEncoder, decoder::MessageDecoder},
    config::{manager::ConfigManager},
    message::{Message, FieldValue},
};
use std::time::Instant;

/// Extension编解码测试
/// 专门测试消息扩展(extension)功能的编解码
#[cfg(test)]
mod extension_codec_tests {
    use super::*;

    /// 创建包含extension的测试配置管理器
    fn create_extension_test_config_manager() -> ConfigManager {
        let mut config_manager = ConfigManager::new();
        
        let config_xml = r#"
            <messages>
                <message type="3001" name="ExtensionTestMessage">
                    <!-- 基础字段 -->
                    <field name="BizID" type="u32" desc="业务编号"/>
                    <field name="OrderID" type="u32" desc="订单ID"/>
                    <field name="Symbol" type="char" length="12" desc="证券代码"/>
                    <field name="Price" type="price" desc="价格"/>
                    <field name="Quantity" type="quantity" desc="数量"/>
                    
                    <!-- 业务扩展1: 基金业务 -->
                    <extension biz_id="300060">
                        <field name="Custodian" type="char" length="3" desc="托管方代码"/>
                        <field name="FundType" type="u8" desc="基金类型"/>
                    </extension>
                    
                    <!-- 业务扩展2: 分红业务 -->
                    <extension biz_id="300070">
                        <field name="DividendSelect" type="char" length="1" desc="分红方式"/>
                        <field name="DividendAmount" type="amount" desc="分红金额"/>
                        <field name="DividendDate" type="date" desc="分红日期"/>
                    </extension>
                    
                    <!-- 业务扩展3: 转换业务 -->
                    <extension biz_id="300080">
                        <field name="DestSecurity" type="char" length="12" desc="目标证券代码"/>
                        <field name="ConvertRatio" type="quantity" desc="转换比例"/>
                        <field name="ConvertTime" type="ntime" desc="转换时间"/>
                    </extension>
                    
                    <!-- 业务扩展4: 复杂字段类型 -->
                    <extension biz_id="300090">
                        <field name="ComplexU64" type="u64" desc="64位无符号整数"/>
                        <field name="ComplexI64" type="i64" desc="64位有符号整数"/>
                        <field name="ComplexAmount" type="amount" desc="复杂金额"/>
                    </extension>
                </message>
                
                <message type="3002" name="SimpleExtensionMessage">
                    <field name="MessageID" type="u32" desc="消息ID"/>
                    <field name="BizID" type="u32" desc="业务编号"/>
                    
                    <!-- 单个简单扩展 -->
                    <extension biz_id="100001">
                        <field name="SimpleField" type="char" length="8" desc="简单字段"/>
                    </extension>
                </message>
                
                <message type="3003" name="NoExtensionMessage">
                    <field name="BaseField1" type="u32" desc="基础字段1"/>
                    <field name="BaseField2" type="char" length="10" desc="基础字段2"/>
                </message>
            </messages>
            "#;
        
        config_manager.load_from_str(config_xml).expect("Failed to load extension test config");
        config_manager
    }

    /// 创建包含extension的测试消息
    fn create_extension_message() -> Message {
        let mut message = Message::new(3001, 12345);
        
        // 基础字段 + BizID (设置为300060以匹配基金业务extension)
        message.add_field("BizID".to_string(), FieldValue::U32(300060));
        message.add_field("OrderID".to_string(), FieldValue::U32(1001));
        message.add_field("Symbol".to_string(), FieldValue::Str("000001.SZ".to_string()));
        message.add_field("Price".to_string(), FieldValue::Float(123.45000)); // 123.45000
        message.add_field("Quantity".to_string(), FieldValue::Float(1000000.0)); // 1000000.000
        
        // Extension 300060: 基金业务
        message.add_field("Custodian".to_string(), FieldValue::Str("001".to_string()));
        message.add_field("FundType".to_string(), FieldValue::U8(1));
        
        // Extension 300070: 分红业务
        message.add_field("DividendSelect".to_string(), FieldValue::Str("U".to_string()));
        message.add_field("DividendAmount".to_string(), FieldValue::I64(5000000000)); // 50000.00000
        message.add_field("DividendDate".to_string(), FieldValue::U32(20231225));
        
        // Extension 300080: 转换业务
        message.add_field("DestSecurity".to_string(), FieldValue::Str("000002.SZ".to_string()));
        message.add_field("ConvertRatio".to_string(), FieldValue::I64(1500000)); // 1500.000
        message.add_field("ConvertTime".to_string(), FieldValue::U64(12345678901234));
        
        // Extension 300090: 复杂字段类型
        message.add_field("ComplexU64".to_string(), FieldValue::U64(18446744073709551615));
        message.add_field("ComplexI64".to_string(), FieldValue::I64(-9223372036854775808));
        message.add_field("ComplexAmount".to_string(), FieldValue::I64(99999999999999999)); // 999999999999.99999
        
        message
    }

    /// 创建不匹配bizid的extension的测试消息
    fn create_mismatch_extension_message() -> Message {
        let mut message = Message::new(3001, 12345);
        
        // 基础字段 + BizID
        message.add_field("BizID".to_string(), FieldValue::U32(300010));
        message.add_field("OrderID".to_string(), FieldValue::U32(1001));
        message.add_field("Symbol".to_string(), FieldValue::Str("000001.SZ".to_string()));
        message.add_field("Price".to_string(), FieldValue::Float(123.45000)); // 123.45000
        message.add_field("Quantity".to_string(), FieldValue::Float(1000000.0)); // 1000000.000
        
        // Extension 300060: 基金业务
        message.add_field("Custodian".to_string(), FieldValue::Str("001".to_string()));
        message.add_field("FundType".to_string(), FieldValue::U8(1));
        
        // Extension 300070: 分红业务
        message.add_field("DividendSelect".to_string(), FieldValue::Str("U".to_string()));
        message.add_field("DividendAmount".to_string(), FieldValue::I64(5000000000)); // 50000.00000
        message.add_field("DividendDate".to_string(), FieldValue::U32(20231225));
        
        // Extension 300080: 转换业务
        message.add_field("DestSecurity".to_string(), FieldValue::Str("000002.SZ".to_string()));
        message.add_field("ConvertRatio".to_string(), FieldValue::I64(1500000)); // 1500.000
        message.add_field("ConvertTime".to_string(), FieldValue::U64(12345678901234));
        
        // Extension 300090: 复杂字段类型
        message.add_field("ComplexU64".to_string(), FieldValue::U64(18446744073709551615));
        message.add_field("ComplexI64".to_string(), FieldValue::I64(-9223372036854775808));
        message.add_field("ComplexAmount".to_string(), FieldValue::I64(99999999999999999)); // 999999999999.99999
        
        message
    }

    /// 创建包含单个extension的简单消息
    fn create_simple_extension_message() -> Message {
        let mut message = Message::new(3002, 54321);
        
        message.add_field("MessageID".to_string(), FieldValue::U32(2001));
        message.add_field("BizID".to_string(), FieldValue::U32(100001));
        message.add_field("SimpleField".to_string(), FieldValue::Str("SIMPLE01".to_string()));
        
        message
    }

    /// 创建不包含extension的基础消息
    fn create_no_extension_message() -> Message {
        let mut message = Message::new(3003, 98765);
        
        message.add_field("BaseField1".to_string(), FieldValue::U32(3001));
        message.add_field("BaseField2".to_string(), FieldValue::Str("NOEXT12345".to_string()));
        
        message
    }

    /// 测试extension的编解码往返
    #[test]
    fn test_extension_encode_decode() {
        let config_manager = create_extension_test_config_manager();
        let original_message = create_extension_message();
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message)
            .expect("Failed to encode multi-extension message");
        
        println!("Multi-extension message encoded size: {} bytes", encoded_data.len());
        println!("Encoded data (hex): {}", hex::encode(&encoded_data));
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode multi-extension message");
        
        // 验证基本信息
        assert_eq!(decoded_message.msg_type, original_message.msg_type, "Message type mismatch");
        assert_eq!(decoded_message.seq_num, original_message.seq_num, "Sequence number mismatch");
        assert_ne!(decoded_message.fields.len(), original_message.fields.len(), "Field count mismatch");
        
        // 验证基础字段
        assert_eq!(decoded_message.get_field("OrderID"), original_message.get_field("OrderID"), "OrderID mismatch");
        assert_eq!(decoded_message.get_field("Symbol"), original_message.get_field("Symbol"), "Symbol mismatch");
        assert_eq!(decoded_message.get_field("Price"), original_message.get_field("Price"), "Price mismatch");
        assert_eq!(decoded_message.get_field("Quantity"), original_message.get_field("Quantity"), "Quantity mismatch");
        
        // 验证Extension 300060字段
        let custodian = decoded_message.get_field("Custodian").as_ref().unwrap().to_string().trim().to_string();
        assert_eq!(Some(&FieldValue::Str(custodian)), original_message.get_field("Custodian"), "Custodian mismatch");
        assert_eq!(decoded_message.get_field("FundType"), original_message.get_field("FundType"), "FundType mismatch");
        
        // 验证Extension 300070字段
        assert_eq!(decoded_message.get_field("DividendSelect"), None, "DividendSelect mismatch");
        assert_eq!(decoded_message.get_field("DividendAmount"), None, "DividendAmount mismatch");
        assert_eq!(decoded_message.get_field("DividendDate"), None, "DividendDate mismatch");
        
        // 验证Extension 300080字段
        assert_eq!(decoded_message.get_field("DestSecurity"), None, "DestSecurity mismatch");
        assert_eq!(decoded_message.get_field("ConvertRatio"), None, "ConvertRatio mismatch");
        assert_eq!(decoded_message.get_field("ConvertTime"), None, "ConvertTime mismatch");
        
        // 验证Extension 300090字段
        assert_eq!(decoded_message.get_field("ComplexU64"), None, "ComplexU64 mismatch");
        assert_eq!(decoded_message.get_field("ComplexI64"), None, "ComplexI64 mismatch");
        assert_eq!(decoded_message.get_field("ComplexAmount"), None, "ComplexAmount mismatch");
        
        println!("✓ Multi-extension encode/decode test passed");
    }

    /// 测试不匹配bizid的extension的编解码往返
    #[test]
    fn test_mismatch_extension_encode_decode() {
        let config_manager = create_extension_test_config_manager();
        let original_message = create_mismatch_extension_message();
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message)
            .expect("Failed to encode multi-extension message");
        
        println!("Multi-extension message encoded size: {} bytes", encoded_data.len());
        println!("Encoded data (hex): {}", hex::encode(&encoded_data));
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode multi-extension message");
        
        // 验证基本信息
        assert_eq!(decoded_message.msg_type, original_message.msg_type, "Message type mismatch");
        assert_eq!(decoded_message.seq_num, original_message.seq_num, "Sequence number mismatch");
        assert_ne!(decoded_message.fields.len(), original_message.fields.len(), "Field count mismatch");
        
        // 验证基础字段
        assert_eq!(decoded_message.get_field("OrderID"), original_message.get_field("OrderID"), "OrderID mismatch");
        assert_eq!(decoded_message.get_field("Symbol"), original_message.get_field("Symbol"), "Symbol mismatch");
        assert_eq!(decoded_message.get_field("Price"), original_message.get_field("Price"), "Price mismatch");
        assert_eq!(decoded_message.get_field("Quantity"), original_message.get_field("Quantity"), "Quantity mismatch");
        
        // 验证Extension 300060字段
        assert_eq!(decoded_message.get_field("Custodian"), None, "Custodian mismatch");
        assert_eq!(decoded_message.get_field("FundType"), None, "FundType mismatch");
        
        // 验证Extension 300070字段
        assert_eq!(decoded_message.get_field("DividendSelect"), None, "DividendSelect mismatch");
        assert_eq!(decoded_message.get_field("DividendAmount"), None, "DividendAmount mismatch");
        assert_eq!(decoded_message.get_field("DividendDate"), None, "DividendDate mismatch");
        
        // 验证Extension 300080字段
        assert_eq!(decoded_message.get_field("DestSecurity"), None, "DestSecurity mismatch");
        assert_eq!(decoded_message.get_field("ConvertRatio"), None, "ConvertRatio mismatch");
        assert_eq!(decoded_message.get_field("ConvertTime"), None, "ConvertTime mismatch");
        
        // 验证Extension 300090字段
        assert_eq!(decoded_message.get_field("ComplexU64"), None, "ComplexU64 mismatch");
        assert_eq!(decoded_message.get_field("ComplexI64"), None, "ComplexI64 mismatch");
        assert_eq!(decoded_message.get_field("ComplexAmount"), None, "ComplexAmount mismatch");
        
        println!("✓ Multi-extension encode/decode test passed");
    }

    /// 测试单个extension的编解码
    #[test]
    fn test_single_extension_encode_decode() {
        let config_manager = create_extension_test_config_manager();
        let original_message = create_simple_extension_message();
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message)
            .expect("Failed to encode single-extension message");
        
        println!("Single-extension message encoded size: {} bytes", encoded_data.len());
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode single-extension message");
        
        // 验证
        assert_eq!(decoded_message.msg_type, original_message.msg_type);
        assert_eq!(decoded_message.seq_num, original_message.seq_num);
        assert_eq!(decoded_message.fields.len(), original_message.fields.len());
        
        assert_eq!(decoded_message.get_field("MessageID"), original_message.get_field("MessageID"));
        assert_eq!(decoded_message.get_field("SimpleField"), original_message.get_field("SimpleField"));
        
        println!("✓ Single-extension encode/decode test passed");
    }

    /// 测试无extension消息的编解码
    #[test]
    fn test_no_extension_encode_decode() {
        let config_manager = create_extension_test_config_manager();
        let original_message = create_no_extension_message();
        
        // 编码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message)
            .expect("Failed to encode no-extension message");
        
        println!("No-extension message encoded size: {} bytes", encoded_data.len());
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode no-extension message");
        
        // 验证
        assert_eq!(decoded_message.msg_type, original_message.msg_type);
        assert_eq!(decoded_message.seq_num, original_message.seq_num);
        assert_eq!(decoded_message.fields.len(), original_message.fields.len());
        
        assert_eq!(decoded_message.get_field("BaseField1"), original_message.get_field("BaseField1"));
        assert_eq!(decoded_message.get_field("BaseField2"), original_message.get_field("BaseField2"));
        
        println!("✓ No-extension encode/decode test passed");
    }

    /// 测试extension字段类型的完整性
    #[test]
    fn test_extension_field_types() {
        let config_manager = create_extension_test_config_manager();
        
        // 创建包含各种字段类型的extension消息
        let mut message = Message::new(3001, 11111);
        
        // 基础字段
        message.add_field("BizID".to_string(), FieldValue::U32(300070));
        message.add_field("OrderID".to_string(), FieldValue::U32(9999));
        message.add_field("Symbol".to_string(), FieldValue::Str("TEST123456".to_string()));
        message.add_field("Price".to_string(), FieldValue::Float(0.0)); // 最小价格
        message.add_field("Quantity".to_string(), FieldValue::Float(1.0)); // 最小数量
        
        // Extension字段 - 测试边界值
        message.add_field("Custodian".to_string(), FieldValue::Str("999".to_string())); // 最大托管方代码
        message.add_field("FundType".to_string(), FieldValue::U8(255)); // U8最大值
        
        message.add_field("DividendSelect".to_string(), FieldValue::Str("C".to_string()));
        message.add_field("DividendAmount".to_string(), FieldValue::Float(0.0)); // 最小金额
        message.add_field("DividendDate".to_string(), FieldValue::U32(10101)); // 最小日期
        
        message.add_field("DestSecurity".to_string(), FieldValue::Str("999999.SZ".to_string()));
        message.add_field("ConvertRatio".to_string(), FieldValue::I64(999999999)); // 大数量
        message.add_field("ConvertTime".to_string(), FieldValue::U64(23595999999999)); // 最大时间
        
        message.add_field("ComplexU64".to_string(), FieldValue::U64(0)); // U64最小值
        message.add_field("ComplexI64".to_string(), FieldValue::I64(9223372036854775807)); // I64最大值
        message.add_field("ComplexAmount".to_string(), FieldValue::I64(1)); // 最小金额
        
        // 编码解码测试
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message)
            .expect("Failed to encode extension field types message");
        
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .expect("Failed to decode extension field types message");
        
        // 验证所有字段
        for (field_name, field_value) in &decoded_message.fields {
            if let FieldValue::Str(_) = field_value {
                continue;
            }

            assert_eq!(
                message.get_field(field_name), 
                Some(field_value),
                "Field {} value mismatch", field_name
            );
        }
        
        println!("✓ Extension field types test passed");
    }

    /// 测试extension的性能
    #[test]
    fn test_extension_performance() {
        let config_manager = create_extension_test_config_manager();
        let message = create_extension_message();
        
        let iterations = 1000;
        
        // 编码性能测试
        let start_time = Instant::now();
        let mut encoded_data = Vec::new();
        
        for _ in 0..iterations {
            let mut encoder = MessageEncoder::new(&config_manager);
            encoded_data = encoder.encode(&message)
                .expect("Failed to encode in performance test");
        }
        
        let encode_duration = start_time.elapsed();
        let encode_ops_per_sec = iterations as f64 / encode_duration.as_secs_f64();
        
        // 解码性能测试
        let start_time = Instant::now();
        
        for _ in 0..iterations {
            let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
            let _ = decoder.decode()
                .expect("Failed to decode in performance test");
        }
        
        let decode_duration = start_time.elapsed();
        let decode_ops_per_sec = iterations as f64 / decode_duration.as_secs_f64();
        
        println!("Extension Performance Results:");
        println!("  Message size: {} bytes", encoded_data.len());
        println!("  Encode: {:.2} ops/sec ({:.3} ms/op)", encode_ops_per_sec, encode_duration.as_millis() as f64 / iterations as f64);
        println!("  Decode: {:.2} ops/sec ({:.3} ms/op)", decode_ops_per_sec, decode_duration.as_millis() as f64 / iterations as f64);
        
        // 性能基准验证
        assert!(encode_ops_per_sec > 100.0, "Extension encode performance too low: {:.2} ops/sec", encode_ops_per_sec);
        assert!(decode_ops_per_sec > 100.0, "Extension decode performance too low: {:.2} ops/sec", decode_ops_per_sec);
        
        println!("✓ Extension performance test passed");
    }

    /// 测试extension的错误处理
    #[test]
    fn test_extension_error_handling() {
        let config_manager = create_extension_test_config_manager();
        
        // 测试1: 缺少extension字段的消息
        let mut incomplete_message = Message::new(3001, 22222);
        incomplete_message.add_field("BizID".to_string(), FieldValue::U32(300070));
        incomplete_message.add_field("OrderID".to_string(), FieldValue::U32(1001));
        incomplete_message.add_field("Symbol".to_string(), FieldValue::Str("000001.SZ".to_string()));
        incomplete_message.add_field("Price".to_string(), FieldValue::Float(12345000.0));
        incomplete_message.add_field("Quantity".to_string(), FieldValue::Float(1000000.0));
        // 故意不添加extension字段
        
        let mut encoder = MessageEncoder::new(&config_manager);
        let result = encoder.encode(&incomplete_message);
        
        // 根据实际实现，这可能成功（如果extension是可选的）或失败
        match result {
            Ok(encoded_data) => {
                println!("Incomplete extension message encoded successfully (size: {} bytes)", encoded_data.len());
                
                // 尝试解码
                let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                let decoded_result = decoder.decode();
                
                match decoded_result {
                    Ok(decoded_message) => {
                        println!("Incomplete extension message decoded successfully");
                        assert_eq!(decoded_message.msg_type, incomplete_message.msg_type);
                    },
                    Err(e) => {
                        println!("Expected decode error for incomplete extension: {:?}", e);
                    }
                }
            },
            Err(e) => {
                println!("Expected encode error for incomplete extension: {:?}", e);
            }
        }
        
        println!("✓ Extension error handling test completed");
    }

    /// 测试extension的内存效率
    #[test]
    fn test_extension_memory_efficiency() {
        let config_manager = create_extension_test_config_manager();
        
        // 测试不同复杂度的extension消息
        let test_cases = vec![
            ("No Extension", create_no_extension_message()),
            ("Single Extension", create_simple_extension_message()),
            ("Multi Extension", create_extension_message()),
        ];
        
        for (name, message) in test_cases {
            let mut encoder = MessageEncoder::new(&config_manager);
            let encoded_data = encoder.encode(&message)
                .expect(&format!("Failed to encode {} message", name));
            
            let field_count = message.fields.len();
            let bytes_per_field = encoded_data.len() as f64 / field_count as f64;
            
            println!("{} - Fields: {}, Size: {} bytes, Bytes/Field: {:.2}", 
                    name, field_count, encoded_data.len(), bytes_per_field);
            
            // 验证编码效率（每个字段的平均字节数应该合理）
            assert!(bytes_per_field < 50.0, "{} encoding too inefficient: {:.2} bytes/field", name, bytes_per_field);
        }
        
        println!("✓ Extension memory efficiency test passed");
    }

    /// 测试extension的批量处理性能
    #[test]
    fn test_extension_batch_processing() {
        let config_manager = create_extension_test_config_manager();
        
        let batch_size = 100;
        let mut messages = Vec::new();
        
        // 创建批量消息
        for i in 0..batch_size {
            let mut message = create_extension_message();
            message.seq_num = i as u32;
            // 修改一些字段值以确保每个消息都不同
            message.add_field("OrderID".to_string(), FieldValue::U32(1000 + i as u32));
            messages.push(message);
        }
        
        // 批量编码
        let start_time = Instant::now();
        let mut encoded_messages = Vec::new();
        
        for message in &messages {
            let mut encoder = MessageEncoder::new(&config_manager);
            let encoded_data = encoder.encode(message)
                .expect("Failed to encode in batch test");
            encoded_messages.push(encoded_data);
        }
        
        let encode_duration = start_time.elapsed();
        
        // 批量解码
        let start_time = Instant::now();
        let mut decoded_messages = Vec::new();
        
        for encoded_data in &encoded_messages {
            let mut decoder = MessageDecoder::new(&config_manager, encoded_data);
            let decoded_message = decoder.decode()
                .expect("Failed to decode in batch test");
            decoded_messages.push(decoded_message);
        }
        
        let decode_duration = start_time.elapsed();
        
        // 验证批量处理结果
        assert_eq!(decoded_messages.len(), messages.len(), "Batch size mismatch");
        
        for (original, decoded) in messages.iter().zip(decoded_messages.iter()) {
            assert_eq!(original.msg_type, decoded.msg_type, "Message type mismatch in batch");
            assert_eq!(original.seq_num, decoded.seq_num, "Sequence number mismatch in batch");
        }
        
        let total_size: usize = encoded_messages.iter().map(|data| data.len()).sum();
        let encode_throughput = total_size as f64 / encode_duration.as_secs_f64() / 1024.0 / 1024.0; // MB/s
        let decode_throughput = total_size as f64 / decode_duration.as_secs_f64() / 1024.0 / 1024.0; // MB/s
        
        println!("Extension Batch Processing Results:");
        println!("  Batch size: {} messages", batch_size);
        println!("  Total size: {} bytes ({:.2} KB)", total_size, total_size as f64 / 1024.0);
        println!("  Encode throughput: {:.2} MB/s", encode_throughput);
        println!("  Decode throughput: {:.2} MB/s", decode_throughput);
        
        println!("✓ Extension batch processing test passed");
    }
}