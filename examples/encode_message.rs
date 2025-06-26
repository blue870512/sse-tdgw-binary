use sse_tdgw_binary::codec::MessageEncoder;
use sse_tdgw_binary::config::ConfigManager;
use sse_tdgw_binary::message::{Message, FieldValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置字符串
    let config_str = r#"<messages>
<message type="40" name="Logon">
  <field name="SenderCompID" type="char" length="32" desc="发送方代码"/>
  <field name="TargetCompID" type="char" length="32" desc="接收方代码"/>
  <field name="HeartBtInt" type="u16" desc="心跳间隔（秒）"/>
  <field name="PrtcVersion" type="char" length="8" desc="协议版本"/>
  <field name="TradeDate" type="date" desc="交易日期（YYYYMMDD）"/>
  <field name="QSize" type="u32" desc="客户端最大队列长度"/>
</message>

<message type="58" name="NewOrderSingle">
  <field name="BizID" type="u32" desc="业务代码"/>
  <field name="ClOrdID" type="char" length="10" desc="会员内部订单编号"/>
  <field name="SecurityID" type="char" length="12" desc="证券代码，前6位有效"/>
  <field name="Side" type="char" length="1" desc="买卖方向：1=买, 2=卖"/>
  <field name="Price" type="price" desc="申报价格"/>
  <field name="OrderQty" type="quantity" desc="申报数量"/>
</message>

<message type="206" name="ExecRptSync">
  <field name="SyncRequests" type="array" desc="同步请求项数组">
    <length_field name="NoGroups" type="u16" desc="同步请求项个数"/>
    <struct>
      <field name="Pbu" type="char" length="8" desc="登录或订阅用PBU"/>
      <field name="SetID" type="u32" desc="平台内分区号"/>
      <field name="BeginReportIndex" type="u64" desc="分区预期回报序号，不支持超过2^32"/>
    </struct>
  </field>
</message>
</messages>"#;

    // 创建配置管理器并加载配置
    let mut config_manager = ConfigManager::new();
    config_manager.load_from_str(config_str)?;
    
    // 创建编码器
    let mut encoder = MessageEncoder::new(&config_manager);
    
    println!("=== 消息编码示例 ===");
    
    // 示例1: 编码登录消息
    println!("\n1. 编码登录消息 (Logon)");
    let mut logon_message = Message::new(40, 1);
    logon_message.add_field("SenderCompID".to_string(), FieldValue::Str("SENDER123".to_string()));
    logon_message.add_field("TargetCompID".to_string(), FieldValue::Str("TARGET456".to_string()));
    logon_message.add_field("HeartBtInt".to_string(), FieldValue::U16(30));
    logon_message.add_field("PrtcVersion".to_string(), FieldValue::Str("1.0".to_string()));
    logon_message.add_field("TradeDate".to_string(), FieldValue::U32(20231201));
    logon_message.add_field("QSize".to_string(), FieldValue::U32(1000));
    
    let encoded_logon = encoder.encode(&logon_message)?;
    println!("编码后长度: {} 字节", encoded_logon.len());
    println!("编码数据: {:02X?}", &encoded_logon[..std::cmp::min(32, encoded_logon.len())]);
    if encoded_logon.len() > 32 {
        println!("... (显示前32字节)");
    }
    
    // 示例2: 编码订单消息（部分字段使用默认值）
    println!("\n2. 编码订单消息 (NewOrderSingle) - 部分字段使用默认值");
    let mut order_message = Message::new(58, 2);
    order_message.add_field("BizID".to_string(), FieldValue::U32(300001));
    order_message.add_field("ClOrdID".to_string(), FieldValue::Str("ORD001".to_string()));
    order_message.add_field("SecurityID".to_string(), FieldValue::Str("000001".to_string()));
    order_message.add_field("Side".to_string(), FieldValue::Str("1".to_string()));
    order_message.add_field("Price".to_string(), FieldValue::Float(12.345)); // Price字段使用Float类型
    order_message.add_field("OrderQty".to_string(), FieldValue::Float(1000.0)); // OrderQty使用Float类型（Quantity类型）
    order_message.add_field("Amount".to_string(), FieldValue::Float(12345.6789)); // Amount使用Float类型（Amount类型）
    
    let encoded_order = encoder.encode(&order_message)?;
    println!("编码后长度: {} 字节", encoded_order.len());
    println!("编码数据: {:02X?}", &encoded_order[..std::cmp::min(32, encoded_order.len())]);
    if encoded_order.len() > 32 {
        println!("... (显示前32字节)");
    }
    
    // 示例3: 编码包含数组的消息
    println!("\n3. 编码包含数组的消息 (ExecRptSync)");
    let mut sync_message = Message::new(206, 3);
    
    // 创建数组数据
    let sync_requests = vec![
        vec![
            FieldValue::Str("PBU00001".to_string()),  // Pbu
            FieldValue::U32(1),                        // SetID
            FieldValue::U64(100),                      // BeginReportIndex
        ],
        vec![
            FieldValue::Str("PBU00002".to_string()),  // Pbu
            FieldValue::U32(2),                        // SetID
            FieldValue::U64(200),                      // BeginReportIndex
        ],
    ];
    
    sync_message.add_field("SyncRequests".to_string(), FieldValue::Array(sync_requests));
    
    let encoded_sync = encoder.encode(&sync_message)?;
    println!("编码后长度: {} 字节", encoded_sync.len());
    println!("编码数据: {:02X?}", &encoded_sync[..std::cmp::min(48, encoded_sync.len())]);
    if encoded_sync.len() > 48 {
        println!("... (显示前48字节)");
    }
    
    // 示例4: 编码空数组消息
    println!("\n4. 编码空数组消息 (ExecRptSync - 空数组)");
    let mut empty_sync_message = Message::new(206, 4);
    empty_sync_message.add_field("SyncRequests".to_string(), FieldValue::Array(Vec::new()));
    
    let encoded_empty_sync = encoder.encode(&empty_sync_message)?;
    println!("编码后长度: {} 字节", encoded_empty_sync.len());
    println!("编码数据: {:02X?}", encoded_empty_sync);
    
    println!("\n=== 编码完成 ===");
    
    Ok(())
}