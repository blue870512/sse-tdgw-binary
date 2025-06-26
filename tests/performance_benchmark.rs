use std::time::{Duration, Instant};
use sse_tdgw_binary::codec::encoder::MessageEncoder;
use sse_tdgw_binary::codec::decoder::MessageDecoder;
use sse_tdgw_binary::config::manager::ConfigManager;
use sse_tdgw_binary::message::{Message, FieldValue};

/// 性能基准测试
/// 测试编解码器在各种场景下的性能表现
#[cfg(test)]
mod performance_tests {
    use super::*;

    /// 创建性能测试用的配置管理器
    fn create_performance_config_manager() -> ConfigManager {
        let mut config_manager = ConfigManager::new();
        
        let config_xml = r#"
        <messages>
            <message type="4001" name="SmallMessage">
                <field name="id" type="u32" desc="ID"/>
                <field name="value" type="i64" desc="值"/>
            </message>
            <message type="4002" name="MediumMessage">
                <field name="header_id" type="u32" desc="头部ID"/>
                <field name="timestamp" type="u64" desc="时间戳"/>
                <field name="price" type="price" desc="价格"/>
                <field name="quantity" type="quantity" desc="数量"/>
                <field name="amount" type="amount" desc="金额"/>
                <field name="symbol" type="char" length="16" desc="代码"/>
                <field name="date" type="date" desc="日期"/>
                <field name="time" type="ntime" desc="时间"/>
                <field name="status" type="u8" desc="状态"/>
                <field name="flags" type="u16" desc="标志"/>
            </message>
            <message type="4003" name="LargeMessage">
                <field name="msg_id" type="u64" desc="消息ID"/>
                <field name="seq_no" type="u32" desc="序列号"/>
                <field name="timestamp" type="u64" desc="时间戳"/>
                <field name="symbol" type="char" length="32" desc="证券代码"/>
                <field name="name" type="char" length="64" desc="证券名称"/>
                <field name="market" type="char" length="8" desc="市场"/>
                <field name="sector" type="char" length="16" desc="行业"/>
                <field name="currency" type="char" length="4" desc="币种"/>
                <field name="price" type="price" desc="价格"/>
                <field name="prev_close" type="price" desc="昨收价"/>
                <field name="open_price" type="price" desc="开盘价"/>
                <field name="high_price" type="price" desc="最高价"/>
                <field name="low_price" type="price" desc="最低价"/>
                <field name="volume" type="quantity" desc="成交量"/>
                <field name="turnover" type="amount" desc="成交额"/>
                <field name="bid_price1" type="price" desc="买一价"/>
                <field name="bid_volume1" type="quantity" desc="买一量"/>
                <field name="ask_price1" type="price" desc="卖一价"/>
                <field name="ask_volume1" type="quantity" desc="卖一量"/>
                <field name="trade_date" type="date" desc="交易日期"/>
                <field name="trade_time" type="ntime" desc="交易时间"/>
                <field name="status" type="u8" desc="状态"/>
                <field name="halt_flag" type="u8" desc="停牌标志"/>
                <field name="limit_up" type="price" desc="涨停价"/>
                <field name="limit_down" type="price" desc="跌停价"/>
            </message>
            <message type="4004" name="ArrayMessage">
                <field name="items" type="array" desc="项目数组">
                    <length_field name="NoGroups" type="u16" desc="同步请求项个数"/>
                    <struct>
                        <field name="item_id" type="u32" desc="项目ID"/>
                        <field name="item_price" type="price" desc="项目价格"/>
                        <field name="item_qty" type="quantity" desc="项目数量"/>
                    </struct>
                </field>
            </message>
        </messages>
        "#;
        
        config_manager.load_from_str(config_xml).expect("Failed to load performance test config");
        config_manager
    }

    /// 创建小消息
    fn create_small_message(seq: u32) -> Message {
        let mut message = Message::new(4001, seq);
        message.add_field("id".to_string(), FieldValue::U32(seq));
        message.add_field("value".to_string(), FieldValue::I64(seq as i64 * 1000));
        message
    }

    /// 创建中等大小消息
    fn create_medium_message(seq: u32) -> Message {
        let mut message = Message::new(4002, seq);
        message.add_field("header_id".to_string(), FieldValue::U32(seq));
        message.add_field("timestamp".to_string(), FieldValue::U64(1703500800000000000 + seq as u64));
        message.add_field("price".to_string(), FieldValue::Float(10000000.0 + (seq % 1000000) as f64));
        message.add_field("quantity".to_string(), FieldValue::Float(1000000.0 + (seq % 100000) as f64));
        message.add_field("amount".to_string(), FieldValue::Float(100000000000.0 + (seq % 100000000) as f64));
        message.add_field("symbol".to_string(), FieldValue::Str(format!("SH{:06}", seq % 1000000)));
        message.add_field("date".to_string(), FieldValue::U32(20231225));
        message.add_field("time".to_string(), FieldValue::U64(930000000000 + (seq as u64 % 10000000)));
        message.add_field("status".to_string(), FieldValue::U8((seq % 256) as u8));
        message.add_field("flags".to_string(), FieldValue::U16((seq % 65536) as u16));
        message
    }

    /// 创建大消息
    fn create_large_message(seq: u32) -> Message {
        let mut message = Message::new(4003, seq);
        message.add_field("msg_id".to_string(), FieldValue::U64(seq as u64));
        message.add_field("seq_no".to_string(), FieldValue::U32(seq));
        message.add_field("timestamp".to_string(), FieldValue::U64(1703500800000000000 + seq as u64));
        message.add_field("symbol".to_string(), FieldValue::Str(format!("SH{:06}.SSE", seq % 1000000)));
        message.add_field("name".to_string(), FieldValue::Str(format!("测试证券{:06}", seq % 1000000)));
        message.add_field("market".to_string(), FieldValue::Str("SSE".to_string()));
        message.add_field("sector".to_string(), FieldValue::Str(format!("SECTOR{:02}", seq % 100)));
        message.add_field("currency".to_string(), FieldValue::Str("CNY".to_string()));
        message.add_field("price".to_string(), FieldValue::Float(10000000.0 + (seq % 1000000) as f64));
        message.add_field("prev_close".to_string(), FieldValue::Float(9950000.0 + (seq % 1000000) as f64));
        message.add_field("open_price".to_string(), FieldValue::Float(9980000.0 + (seq % 1000000) as f64));
        message.add_field("high_price".to_string(), FieldValue::Float(10050000.0 + (seq % 1000000) as f64));
        message.add_field("low_price".to_string(), FieldValue::Float(9900000.0 + (seq % 1000000) as f64));
        message.add_field("volume".to_string(), FieldValue::Float(1000000000.0 + (seq as f64 % 100000000.0)));
        message.add_field("turnover".to_string(), FieldValue::Float(10000000000.0 + (seq as f64 % 1000000000.0)));
        message.add_field("bid_price1".to_string(), FieldValue::Float(9999000.0 + (seq as f64 % 1000000.0)));
        message.add_field("bid_volume1".to_string(), FieldValue::Float(100000.0 + (seq as f64 % 100000.0)));
        message.add_field("ask_price1".to_string(), FieldValue::Float(10001000.0 + (seq as f64 % 100000.0)));
        message.add_field("ask_volume1".to_string(), FieldValue::Float(100000.0 + (seq as f64 % 100000.0)));
        message.add_field("trade_date".to_string(), FieldValue::U32(20231225));
        message.add_field("trade_time".to_string(), FieldValue::U64(930000000000 + (seq as u64 % 10000000)));
        message.add_field("status".to_string(), FieldValue::U8(1));
        message.add_field("halt_flag".to_string(), FieldValue::U8(0));
        message.add_field("limit_up".to_string(), FieldValue::Float(11000000.0 + (seq as f64 % 1000000.0)));
        message.add_field("limit_down".to_string(), FieldValue::Float(9000000.0 + (seq as f64 % 1000000.0)));
        message
    }

    /// 创建数组消息
    fn create_array_message(seq: u32, array_size: usize) -> Message {
        let mut message = Message::new(4004, seq);
        
        let mut array_data = Vec::new();
        for i in 0..array_size {
            array_data.push(vec![
                FieldValue::U32(seq * 1000 + i as u32),
                FieldValue::Float(10000000.0 + (i as f64 % 1000000.0)),
                FieldValue::Float(1000000.0 + (i as f64 % 100000.0)),
            ]);
        }
        
        message.add_field("items".to_string(), FieldValue::Array(array_data));
        message
    }

    /// 执行性能测试的辅助函数
    fn run_performance_test<F>(test_name: &str, iterations: usize, test_fn: F) -> (Duration, f64)
    where
        F: Fn() -> (),
    {
        // 预热
        for _ in 0..100 {
            test_fn();
        }
        
        // 实际测试
        let start_time = Instant::now();
        for _ in 0..iterations {
            test_fn();
        }
        let elapsed = start_time.elapsed();
        
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        println!("[{}] {} iterations in {:?} ({:.2} ops/sec)", 
                test_name, iterations, elapsed, ops_per_sec);
        
        (elapsed, ops_per_sec)
    }

    /// 测试小消息编码性能
    #[test]
    fn test_small_message_encode_performance() {
        let config_manager = create_performance_config_manager();
        let iterations = 10000;
        
        let (elapsed, ops_per_sec) = run_performance_test(
            "Small Message Encode",
            iterations,
            || {
                let message = create_small_message(12345);
                let mut encoder = MessageEncoder::new(&config_manager);
                let _ = encoder.encode(&message).expect("Encode failed");
            }
        );
        
        // 性能断言：小消息编码应该很快
        assert!(ops_per_sec > 1000.0, "Small message encoding should be faster than 1000 ops/sec, got {:.2}", ops_per_sec);
        
        println!("✓ Small message encode performance test passed");
    }

    /// 测试小消息解码性能
    #[test]
    fn test_small_message_decode_performance() {
        let config_manager = create_performance_config_manager();
        let message = create_small_message(12345);
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message).expect("Encode failed");
        let iterations = 10000;
        
        let (elapsed, ops_per_sec) = run_performance_test(
            "Small Message Decode",
            iterations,
            || {
                let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                let _ = decoder.decode().expect("Decode failed");
            }
        );
        
        // 性能断言：小消息解码应该很快
        assert!(ops_per_sec > 1000.0, "Small message decoding should be faster than 1000 ops/sec, got {:.2}", ops_per_sec);
        
        println!("✓ Small message decode performance test passed");
    }

    /// 测试中等消息编解码性能
    #[test]
    fn test_medium_message_roundtrip_performance() {
        let config_manager = create_performance_config_manager();
        let iterations = 5000;
        
        let (elapsed, ops_per_sec) = run_performance_test(
            "Medium Message Roundtrip",
            iterations,
            || {
                let message = create_medium_message(12345);
                let mut encoder = MessageEncoder::new(&config_manager);
                let encoded_data = encoder.encode(&message).expect("Encode failed");
                
                let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                let _ = decoder.decode().expect("Decode failed");
            }
        );
        
        // 性能断言：中等消息往返应该合理快速
        assert!(ops_per_sec > 500.0, "Medium message roundtrip should be faster than 500 ops/sec, got {:.2}", ops_per_sec);
        
        println!("✓ Medium message roundtrip performance test passed");
    }

    /// 测试大消息编解码性能
    #[test]
    fn test_large_message_roundtrip_performance() {
        let config_manager = create_performance_config_manager();
        let iterations = 1000;
        
        let (elapsed, ops_per_sec) = run_performance_test(
            "Large Message Roundtrip",
            iterations,
            || {
                let message = create_large_message(12345);
                let mut encoder = MessageEncoder::new(&config_manager);
                let encoded_data = encoder.encode(&message).expect("Encode failed");
                
                let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                let _ = decoder.decode().expect("Decode failed");
            }
        );
        
        // 性能断言：大消息往返应该在合理范围内
        assert!(ops_per_sec > 100.0, "Large message roundtrip should be faster than 100 ops/sec, got {:.2}", ops_per_sec);
        
        println!("✓ Large message roundtrip performance test passed");
    }

    /// 测试小数组消息性能
    #[test]
    fn test_small_array_message_performance() {
        let config_manager = create_performance_config_manager();
        let iterations = 2000;
        
        let (elapsed, ops_per_sec) = run_performance_test(
            "Small Array Message (10 items)",
            iterations,
            || {
                let message = create_array_message(12345, 10);
                let mut encoder = MessageEncoder::new(&config_manager);
                let encoded_data = encoder.encode(&message).expect("Encode failed");
                
                let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                let _ = decoder.decode().expect("Decode failed");
            }
        );
        
        // 性能断言：小数组消息应该合理快速
        assert!(ops_per_sec > 200.0, "Small array message should be faster than 200 ops/sec, got {:.2}", ops_per_sec);
        
        println!("✓ Small array message performance test passed");
    }

    /// 测试大数组消息性能
    #[test]
    fn test_large_array_message_performance() {
        let config_manager = create_performance_config_manager();
        let iterations = 100;
        
        let (elapsed, ops_per_sec) = run_performance_test(
            "Large Array Message (1000 items)",
            iterations,
            || {
                let message = create_array_message(12345, 1000);
                let mut encoder = MessageEncoder::new(&config_manager);
                let encoded_data = encoder.encode(&message).expect("Encode failed");
                
                let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                let _ = decoder.decode().expect("Decode failed");
            }
        );
        
        // 性能断言：大数组消息应该在合理范围内
        assert!(ops_per_sec > 10.0, "Large array message should be faster than 10 ops/sec, got {:.2}", ops_per_sec);
        
        println!("✓ Large array message performance test passed");
    }

    /// 测试批量消息处理性能
    #[test]
    fn test_batch_message_processing_performance() {
        let config_manager = create_performance_config_manager();
        let batch_size = 1000;
        
        // 准备批量消息
        let mut messages = Vec::new();
        for i in 0..batch_size {
            messages.push(create_medium_message(i as u32));
        }
        
        let start_time = Instant::now();
        
        // 批量编码
        let mut encoded_messages = Vec::new();
        for message in &messages {
            let mut encoder = MessageEncoder::new(&config_manager);
            let encoded_data = encoder.encode(message).expect("Encode failed");
            encoded_messages.push(encoded_data);
        }
        
        let encode_time = start_time.elapsed();
        
        // 批量解码
        let decode_start = Instant::now();
        let mut decoded_messages = Vec::new();
        for encoded_data in &encoded_messages {
            let mut decoder = MessageDecoder::new(&config_manager, encoded_data);
            let decoded_message = decoder.decode().expect("Decode failed");
            decoded_messages.push(decoded_message);
        }
        
        let decode_time = decode_start.elapsed();
        let total_time = start_time.elapsed();
        
        let encode_ops_per_sec = batch_size as f64 / encode_time.as_secs_f64();
        let decode_ops_per_sec = batch_size as f64 / decode_time.as_secs_f64();
        let total_ops_per_sec = batch_size as f64 / total_time.as_secs_f64();
        
        println!("[Batch Processing] {} messages:", batch_size);
        println!("  Encode: {:?} ({:.2} ops/sec)", encode_time, encode_ops_per_sec);
        println!("  Decode: {:?} ({:.2} ops/sec)", decode_time, decode_ops_per_sec);
        println!("  Total:  {:?} ({:.2} ops/sec)", total_time, total_ops_per_sec);
        
        // 验证数据一致性
        assert_eq!(messages.len(), decoded_messages.len(), "Message count mismatch");
        for (i, (original, decoded)) in messages.iter().zip(decoded_messages.iter()).enumerate() {
            assert_eq!(original.msg_type, decoded.msg_type, "Message type mismatch at index {}", i);
            assert_eq!(original.seq_num, decoded.seq_num, "Sequence number mismatch at index {}", i);
        }
        
        // 性能断言
        assert!(total_ops_per_sec > 100.0, "Batch processing should be faster than 100 ops/sec, got {:.2}", total_ops_per_sec);
        
        println!("✓ Batch message processing performance test passed");
    }

    /// 测试内存使用效率
    #[test]
    fn test_memory_efficiency() {
        let config_manager = create_performance_config_manager();
        let iterations = 1000;
        
        // 测试编码后的数据大小
        let small_message = create_small_message(12345);
        let medium_message = create_medium_message(12345);
        let large_message = create_large_message(12345);
        let array_message = create_array_message(12345, 100);
        
        let mut encoder = MessageEncoder::new(&config_manager);
        
        let small_encoded = encoder.encode(&small_message).expect("Encode failed");
        let medium_encoded = encoder.encode(&medium_message).expect("Encode failed");
        let large_encoded = encoder.encode(&large_message).expect("Encode failed");
        let array_encoded = encoder.encode(&array_message).expect("Encode failed");
        
        println!("[Memory Efficiency] Encoded message sizes:");
        println!("  Small message:  {} bytes", small_encoded.len());
        println!("  Medium message: {} bytes", medium_encoded.len());
        println!("  Large message:  {} bytes", large_encoded.len());
        println!("  Array message:  {} bytes (100 items)", array_encoded.len());
        
        // 基本的大小合理性检查
        assert!(small_encoded.len() < medium_encoded.len(), "Small message should be smaller than medium");
        assert!(medium_encoded.len() < large_encoded.len(), "Medium message should be smaller than large");
        
        // 数组消息大小应该与数组大小成正比
        let array_10 = encoder.encode(&create_array_message(12345, 10)).expect("Encode failed");
        let array_100 = encoder.encode(&create_array_message(12345, 100)).expect("Encode failed");
        
        println!("  Array 10 items:  {} bytes", array_10.len());
        println!("  Array 100 items: {} bytes", array_100.len());
        
        // 100个元素的数组应该比10个元素的数组大很多
        assert!(array_100.len() > array_10.len() * 5, "Array size should scale with element count");
        
        println!("✓ Memory efficiency test passed");
    }

    /// 综合性能报告
    #[test]
    fn test_comprehensive_performance_report() {
        let config_manager = create_performance_config_manager();
        
        println!("\n=== Comprehensive Performance Report ===");
        
        // 测试不同消息类型的性能
        let test_cases: Vec<(&str, Box<dyn Fn() -> Message>, usize)> = vec![
            ("Small", Box::new(|| create_small_message(12345)), 5000),
            ("Medium", Box::new(|| create_medium_message(12345)), 2000),
            ("Large", Box::new(|| create_large_message(12345)), 500),
            ("Array-10", Box::new(|| create_array_message(12345, 10)), 1000),
            ("Array-100", Box::new(|| create_array_message(12345, 100)), 200),
        ];
        
        for (name, message_fn, iterations) in test_cases {
            let message = message_fn();
            
            // 编码性能
            let (encode_time, encode_ops) = run_performance_test(
                &format!("{} Encode", name),
                iterations,
                || {
                    let mut encoder = MessageEncoder::new(&config_manager);
                    let _ = encoder.encode(&message).expect("Encode failed");
                }
            );
            
            // 解码性能（先编码一次获取数据）
            let mut encoder = MessageEncoder::new(&config_manager);
            let encoded_data = encoder.encode(&message).expect("Encode failed");
            
            let (decode_time, decode_ops) = run_performance_test(
                &format!("{} Decode", name),
                iterations,
                || {
                    let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
                    let _ = decoder.decode().expect("Decode failed");
                }
            );
            
            println!("[{}] Size: {} bytes, Encode: {:.2} ops/sec, Decode: {:.2} ops/sec", 
                    name, encoded_data.len(), encode_ops, decode_ops);
        }
        
        println!("=== Performance Report Complete ===\n");
        println!("✓ Comprehensive performance report test passed");
    }
}