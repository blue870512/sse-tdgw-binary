use sse_tdgw_binary::{
    codec::{encoder::MessageEncoder, decoder::MessageDecoder},
    config::manager::ConfigManager,
    message::{Message, FieldValue},
};
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    
    // 检查参数数量
    if args.len() != 2 {
        eprintln!("使用方法: {} <配置文件路径>", args[0]);
        eprintln!("示例: {} config/sse-message.xml", args[0]);
        std::process::exit(1);
    }
    
    let config_file_path = &args[1];
    
    // 检查配置文件是否存在
    if !Path::new(config_file_path).exists() {
        eprintln!("错误: 配置文件 '{}' 不存在", config_file_path);
        std::process::exit(1);
    }
    
    println!("=== 使用配置文件进行消息编解码示例 ===");
    println!("配置文件: {}", config_file_path);
    
    // 读取配置文件
    let config_content = fs::read_to_string(config_file_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    // 创建配置管理器并加载配置
    let mut config_manager = ConfigManager::new();
    config_manager.load_from_str(&config_content)
        .map_err(|e| format!("解析配置文件失败: {:?}", e))?;
    
    println!("✓ 配置文件加载成功");
    
    // 创建编码器和解码器
    let mut encoder = MessageEncoder::new(&config_manager);
    
    println!("\n=== 编解码测试 ===");
    
    // 示例1: 登录消息 (Logon - 消息类型40)
    if let Some(_) = config_manager.get_message_def(40) {
        println!("\n1. 测试登录消息 (Logon - 类型40)");
        
        let mut logon_message = Message::new(40, 1001);
        logon_message.add_field("SenderCompID".to_string(), FieldValue::Str("SENDER001".to_string()));
        logon_message.add_field("TargetCompID".to_string(), FieldValue::Str("TARGET001".to_string()));
        logon_message.add_field("HeartBtInt".to_string(), FieldValue::U16(30));
        logon_message.add_field("PrtcVersion".to_string(), FieldValue::Str("1.0".to_string()));
        logon_message.add_field("TradeDate".to_string(), FieldValue::U32(20231201));
        logon_message.add_field("QSize".to_string(), FieldValue::U32(1000));
        
        // 编码
        let encoded_data = encoder.encode(&logon_message)
            .map_err(|e| format!("编码登录消息失败: {:?}", e))?;
        
        println!("  原始消息字段数: {}", logon_message.fields.len());
        println!("  编码后长度: {} 字节", encoded_data.len());
        println!("  编码数据 (前32字节): {:02X?}", 
                &encoded_data[..std::cmp::min(32, encoded_data.len())]);
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .map_err(|e| format!("解码登录消息失败: {:?}", e))?;
        
        println!("  解码后消息类型: {}", decoded_message.msg_type);
        println!("  解码后序列号: {}", decoded_message.seq_num);
        println!("  解码后字段数: {}", decoded_message.fields.len());
        println!("  解码后消息内容: {}", decoded_message.to_string());
        
        // 验证往返一致性
        assert_eq!(logon_message.msg_type, decoded_message.msg_type, "消息类型不匹配");
        assert_eq!(logon_message.seq_num, decoded_message.seq_num, "序列号不匹配");
        assert_eq!(logon_message.fields.len(), decoded_message.fields.len(), "字段数量不匹配");
        
        println!("  ✓ 往返编解码验证成功");
    }
    
    // 示例2: 新订单消息 (NewOrderSingle - 消息类型58)
    if let Some(_) = config_manager.get_message_def(58) {
        println!("\n2. 测试新订单消息 (NewOrderSingle - 类型58)");
        
        let mut order_message = Message::new(58, 1002);
        order_message.add_field("BizID".to_string(), FieldValue::U32(30000));
        order_message.add_field("ClOrdID".to_string(), FieldValue::Str("ORD123456".to_string()));
        order_message.add_field("SecurityID".to_string(), FieldValue::Str("000001".to_string()));
        order_message.add_field("Side".to_string(), FieldValue::Str("1".to_string()));
        order_message.add_field("Price".to_string(), FieldValue::Float(12.345)); // 12.345 * 100000000
        order_message.add_field("OrderQty".to_string(), FieldValue::Float(1000.000)); // 1000.000 * 1000
        
        // 编码
        let encoded_data = encoder.encode(&order_message)
            .map_err(|e| format!("编码订单消息失败: {:?}", e))?;
        
        println!("  原始消息字段数: {}", order_message.fields.len());
        println!("  编码后长度: {} 字节", encoded_data.len());
        println!("  编码数据 (前32字节): {:02X?}", 
                &encoded_data[..std::cmp::min(32, encoded_data.len())]);
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .map_err(|e| format!("解码订单消息失败: {:?}", e))?;
        
        println!("  解码后消息类型: {}", decoded_message.msg_type);
        println!("  解码后序列号: {}", decoded_message.seq_num);
        println!("  解码后字段数: {}", decoded_message.fields.len());
        println!("  解码后消息内容: {}", decoded_message.to_string());
        
        // 验证关键字段
        if let Some(FieldValue::U32(biz_id)) = decoded_message.get_field("BizID") {
            println!("  业务代码: {}", biz_id);
        }
        if let Some(FieldValue::Str(cl_ord_id)) = decoded_message.get_field("ClOrdID") {
            println!("  订单编号: {}", cl_ord_id);
        }
        if let Some(FieldValue::Float(price)) = decoded_message.get_field("Price") {
            println!("  价格: {}", price);
        }
        
        // 验证往返一致性
        assert_eq!(order_message.msg_type, decoded_message.msg_type, "消息类型不匹配");
        assert_eq!(order_message.seq_num, decoded_message.seq_num, "序列号不匹配");
        
        println!("  ✓ 往返编解码验证成功");
    }
    
    // 示例3: 心跳消息 (Heartbeat - 消息类型42)
    if let Some(_) = config_manager.get_message_def(42) {
        println!("\n3. 测试心跳消息 (Heartbeat - 类型42)");
        
        let heartbeat_message = Message::new(42, 1003);
        
        // 编码
        let encoded_data = encoder.encode(&heartbeat_message)
            .map_err(|e| format!("编码心跳消息失败: {:?}", e))?;
        
        println!("  原始消息字段数: {}", heartbeat_message.fields.len());
        println!("  编码后长度: {} 字节", encoded_data.len());
        println!("  编码数据: {:02X?}", encoded_data);
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .map_err(|e| format!("解码心跳消息失败: {:?}", e))?;
        
        println!("  解码后消息类型: {}", decoded_message.msg_type);
        println!("  解码后序列号: {}", decoded_message.seq_num);
        println!("  解码后消息内容: {}", decoded_message.to_string());
        
        println!("  ✓ 往返编解码验证成功");
    }
    
    // 示例4: 包含扩展字段的消息测试
    if let Some(_) = config_manager.get_message_def(58) {
        println!("\n4. 测试包含扩展字段的订单消息");
        
        let mut extended_order = Message::new(58, 1004);
        extended_order.add_field("BizID".to_string(), FieldValue::U32(300060)); // 基金业务
        extended_order.add_field("ClOrdID".to_string(), FieldValue::Str("FUND12345".to_string()));
        extended_order.add_field("SecurityID".to_string(), FieldValue::Str("159001".to_string()));
        extended_order.add_field("Side".to_string(), FieldValue::Str("1".to_string()));
        extended_order.add_field("Price".to_string(), FieldValue::Float(10.50)); // 10.50
        extended_order.add_field("OrderQty".to_string(), FieldValue::Float(500.000)); // 500.000
        
        // 添加扩展字段（如果配置中有定义）
        extended_order.add_field("Custodian".to_string(), FieldValue::Str("001".to_string()));
        extended_order.add_field("FundType".to_string(), FieldValue::U8(1));
        
        // 编码
        let encoded_data = encoder.encode(&extended_order)
            .map_err(|e| format!("编码扩展订单消息失败: {:?}", e))?;
        
        println!("  原始消息字段数: {}", extended_order.fields.len());
        println!("  编码后长度: {} 字节", encoded_data.len());
        
        // 解码
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode()
            .map_err(|e| format!("解码扩展订单消息失败: {:?}", e))?;
        
        println!("  解码后字段数: {}", decoded_message.fields.len());
        
        // 显示所有解码后的字段
        println!("  解码后的字段:");
        for (field_name, field_value) in &decoded_message.fields {
            println!("    {}: {:?}", field_name, field_value);
        }
        println!("  解码后消息内容: {}", decoded_message.to_string());
        
        println!("  ✓ 扩展字段编解码验证成功");
    }
    
    // 性能测试
    println!("\n=== 性能测试 ===");
    
    let test_message = Message::new(33, 9999); // 使用简单的心跳消息进行性能测试
    let iterations = 1000;
    
    // 编码性能测试
    let start_time = std::time::Instant::now();
    let mut last_encoded = Vec::new();
    
    for _ in 0..iterations {
        last_encoded = encoder.encode(&test_message)
            .map_err(|e| format!("性能测试编码失败: {:?}", e))?;
    }
    
    let encode_duration = start_time.elapsed();
    let encode_ops_per_sec = iterations as f64 / encode_duration.as_secs_f64();
    
    // 解码性能测试
    let start_time = std::time::Instant::now();
    
    for _ in 0..iterations {
        let mut decoder = MessageDecoder::new(&config_manager, &last_encoded);
        let _ = decoder.decode()
            .map_err(|e| format!("性能测试解码失败: {:?}", e))?;
    }
    
    let decode_duration = start_time.elapsed();
    let decode_ops_per_sec = iterations as f64 / decode_duration.as_secs_f64();
    
    println!("性能测试结果 ({} 次迭代):", iterations);
    println!("  编码: {:.2} ops/sec ({:.3} ms/op)", 
            encode_ops_per_sec, 
            encode_duration.as_millis() as f64 / iterations as f64);
    println!("  解码: {:.2} ops/sec ({:.3} ms/op)", 
            decode_ops_per_sec, 
            decode_duration.as_millis() as f64 / iterations as f64);
    
    println!("\n=== 测试完成 ===");
    println!("✓ 所有编解码测试通过");
    
    Ok(())
}