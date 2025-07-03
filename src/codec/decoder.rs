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

/// 消息解码器，用于将二进制数据解析为 Message 对象
pub struct MessageDecoder<'a> {
    /// 配置管理器，用于获取消息定义
    config_manager: &'a ConfigManager,
    /// 二进制数据
    buffer: &'a [u8],
    /// 当前解析位置
    position: usize,
}

impl<'a> MessageDecoder<'a> {
    /// 创建一个新的消息解码器
    pub fn new(config_manager: &'a ConfigManager, buffer: &'a [u8]) -> Self {
        Self {
            config_manager,
            buffer,
            position: 0,
        }
    }

    /// 解码消息
    pub fn decode(&mut self) -> MessageResult<Message> {
        // 解析消息头部
        if self.buffer.len() < MessageHeader::SIZE {
            return Err(MessageError::HeaderTooShort);
        }

        // 解码消息类型
        let msg_type = BigEndian::read_u32(&self.buffer[0..4]);
        
        // 解码序列号
        let seq_num = BigEndian::read_u32(&self.buffer[4..8]);
        
        // 解码消息体长度
        let body_length = BigEndian::read_u32(&self.buffer[8..12]);
        
        // 更新位置到消息体开始处
        self.position = MessageHeader::SIZE;

        // 验证校验和
        let body_end = MessageHeader::SIZE + body_length as usize;
        self.verify_checksum(body_end)?;

        // 获取消息定义
        let message_def = self.config_manager.get_message_def(msg_type)
            .ok_or_else(|| MessageError::UnknownMessageType(msg_type))?;

        // 创建消息对象
        let mut message = Message::new(msg_type, seq_num);

        // 解析消息字段
        for field_def in &message_def.fields {
            let field_value = self.decode_field(&field_def.base, Some(field_def))?;
            message.add_field(field_def.base.name.clone(), field_value);
        }

        // 解析扩展字段
        if message.has_field("BizID") && message_def.extensions.len() > 0 {
            let biz_id = message.get_field("BizID").unwrap().as_u32().unwrap();
            let biz_extension = self.config_manager.get_extension(msg_type, biz_id);

            if let Some(biz_extension) = biz_extension {
                for field_def in &biz_extension.fields {
                    let field_value = self.decode_field(&field_def, None)?;
                    message.add_field(field_def.name.clone(), field_value);
                }
            }
        }

        Ok(message)
    }

    /// 验证校验和
    fn verify_checksum(&self, body_end: usize) -> MessageResult<()> {
        if self.buffer.len() < body_end + 4 {
            return Err(MessageError::BodyTooShort);
        }
        
        // 计算校验和 - 使用 uint8 累加然后转换为 uint32
        let mut checksum: u8 = 0;
        for byte in &self.buffer[0..body_end] {
            checksum = checksum.wrapping_add(*byte);
        }
        
        // 读取消息中的校验和
        let message_checksum = BigEndian::read_u32(&self.buffer[body_end..body_end+4]);
        
        // 比较校验和 - 将 u8 转换为 u32 后比较
        if (checksum as u32) != message_checksum {
            return Err(MessageError::ChecksumError);
        }
        
        Ok(())
    }

    /// 解码字段
    /// 
    /// * `base_field_def` - 基本字段定义
    /// * `field_def` - 完整字段定义，用于数组类型
    fn decode_field(&mut self, base_field_def: &BaseFieldDef, field_def: Option<&FieldDef>) -> MessageResult<FieldValue> {
        match base_field_def.r#type {
            FieldType::U8 => {
                if self.position + 1 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = self.buffer[self.position];
                self.position += 1;
                Ok(FieldValue::U8(value))
            },
            FieldType::U16 => {
                if self.position + 2 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_u16(&self.buffer[self.position..]);
                self.position += 2;
                Ok(FieldValue::U16(value))
            },
            FieldType::U32 => {
                if self.position + 4 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_u32(&self.buffer[self.position..]);
                self.position += 4;
                Ok(FieldValue::U32(value))
            },
            FieldType::U64 => {
                if self.position + 8 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_u64(&self.buffer[self.position..]);
                self.position += 8;
                Ok(FieldValue::U64(value))
            },
            FieldType::I64 => {
                if self.position + 8 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_i64(&self.buffer[self.position..]);
                self.position += 8;
                Ok(FieldValue::I64(value))
            },
            FieldType::Char => {
                let length = base_field_def.length.ok_or_else(|| {
                    MessageError::FieldDecodeError(format!("Char field {} missing length", base_field_def.name))
                })?;
                
                if self.position + length > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                
                let s = std::str::from_utf8(&self.buffer[self.position..self.position + length])
                    .map_err(|e| MessageError::FieldDecodeError(format!("UTF-8 error: {}", e)))?
                    .trim()
                    .to_string();
                    
                self.position += length;
                Ok(FieldValue::Str(s))
            },
            FieldType::Price => {
                if self.position + 8 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_i64(&self.buffer[self.position..]);
                self.position += 8;
                if !validate_price(value) {
                    return Err(MessageError::ValueExceedsRange(format!("Price value {} exceeds maximum limit", value)));
                }
                // Price类型：先解析为i64，然后除以100000转成float
                let float_value = value as f64 / TYPE_PRICE_SCALE;
                Ok(FieldValue::Float(float_value))
            },
            FieldType::Quantity => {
                if self.position + 8 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_i64(&self.buffer[self.position..]);
                self.position += 8;
                if !validate_quantity(value) {
                    return Err(MessageError::ValueExceedsRange(format!("Quantity value {} exceeds maximum limit", value)));
                }
                let float_value = value as f64 / TYPE_QUANTITY_SCALE;
                Ok(FieldValue::Float(float_value))
            },
            FieldType::Amount => {
                if self.position + 8 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_i64(&self.buffer[self.position..]);
                self.position += 8;
                // Amount类型：先解析为i64，验证小于999999999999999，然后除以TYPE_AMOUNT_SCALE转成float
                if !validate_amount(value) {
                    return Err(MessageError::ValueExceedsRange(format!("Amount value {} exceeds maximum limit", value)));
                }
                let float_value = value as f64 / TYPE_AMOUNT_SCALE;
                Ok(FieldValue::Float(float_value))
            },
            FieldType::Date => {
                if self.position + 4 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_u32(&self.buffer[self.position..]);
                self.position += 4;
                
                // 验证Date格式 YYYYMMDD
                if !validate_date_format(value) {
                    return Err(MessageError::InvalidFieldValue(format!(
                        "Invalid date format: {}. Expected YYYYMMDD format with valid year (0000-9999), month (01-12), and day (01-31)", 
                        value
                    )));
                }
                
                Ok(FieldValue::U32(value))
            },
            FieldType::NTime => {
                if self.position + 8 > self.buffer.len() {
                    return Err(MessageError::BodyTooShort);
                }
                let value = BigEndian::read_u64(&self.buffer[self.position..]);
                self.position += 8;
                
                // 验证NTime格式 HHMMSSsssnnnn
                if !validate_ntime_format(value) {
                    return Err(MessageError::InvalidFieldValue(format!(
                        "Invalid ntime format: {}. Expected HHMMSSsssnnnn format with valid hour (00-23), minute (00-59), second (00-59), millisecond (000-999), and hundred nanosecond (0000-9999)", 
                        value
                    )));
                }
                
                Ok(FieldValue::U64(value))
            },
            FieldType::Array => {
                // 如果是数组类型，需要完整的字段定义
                let field_def = field_def.ok_or_else(|| {
                    MessageError::ArrayElementDecodeError(format!("Array field {} missing field definition", base_field_def.name))
                })?;
                self.decode_array(field_def)
            },
        }
    }

    /// 解码数组字段
    fn decode_array(&mut self, field_def: &FieldDef) -> MessageResult<FieldValue> {
        // 获取数组长度字段定义
        let length_field_def = field_def.length_field.as_ref().ok_or_else(|| {
            MessageError::ArrayCountDecodeError(format!("Array field {} missing length field", field_def.base.name))
        })?;
        
        // 解码数组长度
        let length_value = self.decode_field(length_field_def, None)?;
        
        // 获取数组长度值
        let length = match length_value {
            FieldValue::U8(v) => v as usize,
            FieldValue::U16(v) => v as usize,
            FieldValue::U32(v) => v as usize,
            _ => return Err(MessageError::InvalidArrayCountType),
        };
        
        // 获取数组元素结构定义
        let struct_def = field_def.r#struct.as_ref().ok_or_else(|| {
            MessageError::ArrayElementDecodeError(format!("Array field {} missing struct definition", field_def.base.name))
        })?;
        
        // 解码数组元素
        let mut array_elements = Vec::with_capacity(length);
        for _ in 0..length {
            let mut element = Vec::with_capacity(struct_def.fields.len());
            
            // 解码每个元素的字段
            for field in &struct_def.fields {
                let field_value = self.decode_field(field, None)?;
                element.push(field_value);
            }
            
            array_elements.push(element);
        }
        
        Ok(FieldValue::Array(array_elements))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::manager::ConfigManager;
    use crate::codec::encoder::MessageEncoder;
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

<message type="41" name="Logout">
  <field name="SessionStatus" type="u32" desc="会话状态代码"/>
  <field name="Text" type="char" length="64" desc="文本信息"/>
</message>

<message type="33" name="Heartbeat">
</message>

<message type="58" name="NewOrderSingle">
  <field name="BizID" type="u32" desc="业务代码"/>
  <field name="BizPbu" type="char" length="8" desc="业务PBU编号，前5位有"/>
  <field name="ClOrdID" type="char" length="10" desc="会员内部订单编号"/>
  <field name="SecurityID" type="char" length="12" desc="证券代码，前6位有效"/>
  <field name="Account" type="char" length="13" desc="证券账户，前10位有效"/>
  <field name="OwnerType" type="u8" desc="订单所有者类型，暂不启"/>
  <field name="Side" type="char" length="1" desc="买卖方向：1=买, 2=卖"/>
  <field name="Price" type="price" desc="申报价格"/>
  <field name="OrderQty" type="quantity" desc="申报数量"/>
  <field name="OrdType" type="char" length="1" desc="订单类型：1=市转撤, 2=限价, 3=市转限, 4=本方最优, 5=对手方最优"/>
  <field name="TimeInForce" type="char" length="1" desc="订单有效时间类型：0=当日有效"/>
  <field name="TransactTime" type="ntime" desc="申报时间"/>
  <field name="CreditTag" type="char" length="2" desc="信用标签，用于现货竞价交易业务的信用交易，取值：XY=担保品买卖, RZ=融资交易, RQ=融券交易, PC=平仓交易, 其他业务填写默认值，无意义。"/>
  <field name="ClearingFirm" type="char" length="8" desc="结算会员代码，前5位有效"/>
  <field name="BranchID" type="char" length="8" desc="营业部代码，前5位有效"/>
  <field name="UserInfo" type="char" length="32" desc="用户私有信息，前12位有效"/>

  <extension biz_id="300060">
    <field name="Custodian" type="char" length="3" desc="放式基金转托管的目标方代理人。对方的销售人代码000-999，不足3位左侧补 0."/>
  </extension>
  <extension biz_id="300070">
    <field name="DividendSelect" type="char" length="1" desc="分红方式：U=红利转投, C=现金分红"/>
  </extension>
  <extension biz_id="300080">
    <field name="DestSecurity" type="char" length="12" desc="转换的目标基金代码，前6位有效"/>
  </extension>
  <extension biz_id="300090">
    <field name="DestSecurity" type="char" length="12" desc="被划转的目标证券代码，前6位有效"/>
  </extension>
  <extension biz_id="300091">
    <field name="DestSecurity" type="char" length="12" desc="被划转的目标证券代码，前6位有效"/>
  </extension>
  <extension biz_id="300092">
    <field name="DestSecurity" type="char" length="12" desc="被划转的目标证券代码，前6位有效"/>
  </extension>
  <extension biz_id="300093">
    <field name="DestSecurity" type="char" length="12" desc="被划转的目标证券代码，前6位有效"/>
  </extension>
  <extension biz_id="300094">
    <field name="DestSecurity" type="char" length="12" desc="被划转的目标证券代码，前6位有效"/>
  </extension>
  <extension biz_id="300095">
    <field name="DestSecurity" type="char" length="12" desc="被划转的目标证券代码，前6位有效"/>
  </extension>
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

<message type="100" name="TestMessage">
  <field name="TestU8" type="u8" desc="测试U8字段"/>
  <field name="TestU16" type="u16" desc="测试U16字段"/>
  <field name="TestU32" type="u32" desc="测试U32字段"/>
  <field name="TestU64" type="u64" desc="测试U64字段"/>
  <field name="TestI64" type="i64" desc="测试I64字段"/>
  <field name="TestChar" type="char" length="10" desc="测试字符串字段"/>
  <field name="TestPrice" type="price" desc="测试价格字段"/>
  <field name="TestQuantity" type="quantity" desc="测试数量字段"/>
  <field name="TestAmount" type="amount" desc="测试金额字段"/>
  <field name="TestDate" type="date" desc="测试日期字段"/>
  <field name="TestNTime" type="ntime" desc="测试时间字段"/>
</message>

</messages>"#;

const CONFIG_SUBSET_STR: &str = r#"<messages>
<message type="100" name="TestMessage">
  <field name="TestU8" type="u8" desc="测试U8字段"/>
  <field name="TestU16" type="u16" desc="测试U16字段"/>
  <field name="TestU32" type="u32" desc="测试U32字段"/>
  <field name="TestU64" type="u64" desc="测试U64字段"/>
  <field name="TestI64" type="i64" desc="测试I64字段"/>
  <field name="TestChar" type="char" length="10" desc="测试字符串字段"/>
  <field name="TestPrice" type="price" desc="测试价格字段"/>
  <field name="TestQuantity" type="quantity" desc="测试数量字段"/>
  <field name="TestAmount" type="amount" desc="测试金额字段"/>
  <field name="TestDate" type="date" desc="测试日期字段"/>
</message>

</messages>
"#;

    const LOGIN_MESSAGE: [u8; 98] = [
        0, 0, 0, 40, 0, 0, 0, 1, 0, 0, 0, 82, 83, 69, 78, 68, 
        69, 82, 49, 50, 51, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 
        32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 84, 65, 82, 71, 
        69, 84, 52, 53, 54, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 
        32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 30, 49, 46, 
        48, 32, 32, 32, 32, 32, 1, 52, 180, 33, 0, 0, 3, 232, 0, 0, 0, 58
    ];
    
    /// 创建测试用的配置管理器
    fn create_test_config_manager() -> ConfigManager {
        let mut config_manager = ConfigManager::new();
        config_manager.load_from_str(CONFIG_STR).unwrap();
        config_manager
    }

    /// 创建测试用的配置管理器子集
    /// 只包含消息类型为100的消息定义
    fn create_test_config_manager_subset() -> ConfigManager {
        let mut config_manager = ConfigManager::new();
        config_manager.load_from_str(CONFIG_SUBSET_STR).unwrap();
        config_manager
    }

    /// 创建包含各种字段类型的测试消息
    fn create_test_message() -> Message {
        let mut message = Message::new(100, 12345);
        
        // 添加各种类型的字段
        message.add_field("TestU8".to_string(), FieldValue::U8(255));
        message.add_field("TestU16".to_string(), FieldValue::U16(65535));
        message.add_field("TestU32".to_string(), FieldValue::U32(4294967295));
        message.add_field("TestU64".to_string(), FieldValue::U64(18446744073709551615));
        message.add_field("TestI64".to_string(), FieldValue::I64(-9223372036854775808));
        message.add_field("TestChar".to_string(), FieldValue::Str("Hello".to_string()));
        message.add_field("TestPrice".to_string(), FieldValue::Float(123.45));
        message.add_field("TestQuantity".to_string(), FieldValue::Float(1000.123));
        message.add_field("TestAmount".to_string(), FieldValue::Float(50000.12345));
        message.add_field("TestDate".to_string(), FieldValue::U32(20231225));
        message.add_field("TestNTime".to_string(), FieldValue::U64(1234567890123));
        
        message
    }

    /// 创建包含扩展字段的测试消息
    fn create_extension_test_message() -> Message {
        let mut message = Message::new(58, 54321);
        
        // 基础字段
        message.add_field("BizID".to_string(), FieldValue::U32(300060));
        message.add_field("BizPbu".to_string(), FieldValue::Str("PBU001".to_string()));
        message.add_field("ClOrdID".to_string(), FieldValue::Str("ORD123".to_string()));
        message.add_field("SecurityID".to_string(), FieldValue::Str("000001.SZ".to_string()));
        message.add_field("Account".to_string(), FieldValue::Str("1234567890".to_string()));
        message.add_field("OwnerType".to_string(), FieldValue::U8(1));
        message.add_field("Side".to_string(), FieldValue::Str("1".to_string()));
        message.add_field("Price".to_string(), FieldValue::Float(10.50));
        message.add_field("OrderQty".to_string(), FieldValue::Float(1000.0));
        message.add_field("OrdType".to_string(), FieldValue::Str("2".to_string()));
        message.add_field("TimeInForce".to_string(), FieldValue::Str("0".to_string()));
        message.add_field("TransactTime".to_string(), FieldValue::U64(1234567890123));
        message.add_field("CreditTag".to_string(), FieldValue::Str("XY".to_string()));
        message.add_field("ClearingFirm".to_string(), FieldValue::Str("FIRM001".to_string()));
        message.add_field("BranchID".to_string(), FieldValue::Str("BR001".to_string()));
        message.add_field("UserInfo".to_string(), FieldValue::Str("USER123".to_string()));
        
        // 扩展字段 (300060)
        message.add_field("Custodian".to_string(), FieldValue::Str("001".to_string()));
        
        message
    }

    /// 创建包含数组字段的测试消息
    fn create_array_test_message() -> Message {
        let mut message = Message::new(206, 98765);
        
        // 创建数组元素
        let mut array_elements = Vec::new();
        
        // 第一个元素
        let element1 = vec![
            FieldValue::Str("PBU001".to_string()),
            FieldValue::U32(1),
            FieldValue::U64(100),
        ];
        array_elements.push(element1);
        
        // 第二个元素
        let element2 = vec![
            FieldValue::Str("PBU002".to_string()),
            FieldValue::U32(2),
            FieldValue::U64(200),
        ];
        array_elements.push(element2);
        
        message.add_field("SyncRequests".to_string(), FieldValue::Array(array_elements));
        
        message
    }

    #[test]
    fn test_decode_original() {
        let config_manager = create_test_config_manager();
        let mut decoder = MessageDecoder::new(&config_manager, &LOGIN_MESSAGE);
        let message = decoder.decode().unwrap();

        assert_eq!(message.msg_type, 40);
        assert_eq!(message.seq_num, 1);
        assert_eq!(message.get_field("SenderCompID").unwrap().to_string().trim(), "SENDER123");
        let trade_date: u32 = message.get_field("TradeDate").unwrap().as_u32().unwrap();
        assert_eq!(trade_date, 20231201u32);
    }

    #[test]
    fn test_encode_decode_roundtrip_basic_types() {
        let config_manager = create_test_config_manager();
        let original_message = create_test_message();
        
        // 编码消息
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message).unwrap();
        
        // 解码消息
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode().unwrap();
        
        // 验证消息头
        assert_eq!(decoded_message.msg_type, original_message.msg_type);
        assert_eq!(decoded_message.seq_num, original_message.seq_num);
        
        // 验证各种字段类型
        assert_eq!(decoded_message.get_field("TestU8").unwrap().as_u8().unwrap(), 255);
        assert_eq!(decoded_message.get_field("TestU16").unwrap().as_u16().unwrap(), 65535);
        assert_eq!(decoded_message.get_field("TestU32").unwrap().as_u32().unwrap(), 4294967295);
        assert_eq!(decoded_message.get_field("TestU64").unwrap().as_u64().unwrap(), 18446744073709551615);
        assert_eq!(decoded_message.get_field("TestI64").unwrap().as_i64().unwrap(), -9223372036854775808);
        assert_eq!(decoded_message.get_field("TestChar").unwrap().to_string().trim(), "Hello");
        
        // 验证浮点数字段（考虑精度）
        let price = decoded_message.get_field("TestPrice").unwrap().as_f64().unwrap();
        assert!((price - 123.45).abs() < 0.00001);
        
        let quantity = decoded_message.get_field("TestQuantity").unwrap().as_f64().unwrap();
        assert!((quantity - 1000.123).abs() < 0.001);
        
        let amount = decoded_message.get_field("TestAmount").unwrap().as_f64().unwrap();
        assert!((amount - 50000.12345).abs() < 0.00001);
        
        // 验证日期和时间字段
        assert_eq!(decoded_message.get_field("TestDate").unwrap().as_u32().unwrap(), 20231225);
        assert_eq!(decoded_message.get_field("TestNTime").unwrap().as_u64().unwrap(), 1234567890123);
    }

    #[test]
    fn test_encode_decode_roundtrip_with_extensions() {
        let config_manager = create_test_config_manager();
        let original_message = create_extension_test_message();
        
        // 编码消息
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message).unwrap();
        
        // 解码消息
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode().unwrap();
        
        // 验证消息头
        assert_eq!(decoded_message.msg_type, original_message.msg_type);
        assert_eq!(decoded_message.seq_num, original_message.seq_num);
        
        // 验证基础字段
        assert_eq!(decoded_message.get_field("BizID").unwrap().as_u32().unwrap(), 300060);
        assert_eq!(decoded_message.get_field("BizPbu").unwrap().to_string().trim(), "PBU001");
        assert_eq!(decoded_message.get_field("ClOrdID").unwrap().to_string().trim(), "ORD123");
        assert_eq!(decoded_message.get_field("SecurityID").unwrap().to_string().trim(), "000001.SZ");
        assert_eq!(decoded_message.get_field("Side").unwrap().to_string().trim(), "1");
        
        // 验证价格和数量字段
        let price = decoded_message.get_field("Price").unwrap().as_f64().unwrap();
        assert!((price - 10.50).abs() < 0.00001);
        
        let quantity = decoded_message.get_field("OrderQty").unwrap().as_f64().unwrap();
        assert!((quantity - 1000.0).abs() < 0.001);
        
        // 验证扩展字段
        assert_eq!(decoded_message.get_field("Custodian").unwrap().to_string().trim(), "001");
    }

    #[test]
    fn test_encode_decode_roundtrip_with_arrays() {
        let config_manager = create_test_config_manager();
        let original_message = create_array_test_message();
        
        // 编码消息
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message).unwrap();
        
        // 解码消息
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode().unwrap();
        
        // 验证消息头
        assert_eq!(decoded_message.msg_type, original_message.msg_type);
        assert_eq!(decoded_message.seq_num, original_message.seq_num);
        
        // 验证数组字段
        let array_field = decoded_message.get_field("SyncRequests").unwrap();
        if let FieldValue::Array(elements) = array_field {
            assert_eq!(elements.len(), 2);
            
            // 验证第一个元素
            let element1 = &elements[0];
            assert_eq!(element1[0].to_string().trim(), "PBU001");
            assert_eq!(element1[1].as_u32().unwrap(), 1);
            assert_eq!(element1[2].as_u64().unwrap(), 100);
            
            // 验证第二个元素
            let element2 = &elements[1];
            assert_eq!(element2[0].to_string().trim(), "PBU002");
            assert_eq!(element2[1].as_u32().unwrap(), 2);
            assert_eq!(element2[2].as_u64().unwrap(), 200);
        } else {
            panic!("Expected array field");
        }
    }

    #[test]
    fn test_decode_subset_fields() {
        let config_manager = create_test_config_manager();
        let config_subset_manager = create_test_config_manager_subset();
        let original_message = create_test_message();
        
        // 编码完整消息
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&original_message).unwrap();
        
        // 解码消息
        let mut decoder = MessageDecoder::new(&config_subset_manager, &encoded_data);
        let decoded_message = decoder.decode().unwrap();
        
        // 只验证字段子集
        let test_fields = vec![
            ("TestU8", FieldValue::U8(255)),
            ("TestU32", FieldValue::U32(4294967295)),
            ("TestChar", FieldValue::Str("Hello".to_string())),
            ("TestDate", FieldValue::U32(20231225)),
        ];
        
        for (field_name, expected_value) in test_fields {
            let decoded_value = decoded_message.get_field(field_name).unwrap();
            match (&expected_value, decoded_value) {
                (FieldValue::U8(expected), FieldValue::U8(actual)) => {
                    assert_eq!(expected, actual, "Field {} mismatch", field_name);
                },
                (FieldValue::U32(expected), FieldValue::U32(actual)) => {
                    assert_eq!(expected, actual, "Field {} mismatch", field_name);
                },
                (FieldValue::Str(expected), FieldValue::Str(actual)) => {
                    assert_eq!(expected.trim(), actual.trim(), "Field {} mismatch", field_name);
                },
                _ => panic!("Unexpected field type for {}", field_name),
            }
        }
    }

    #[test]
    fn test_decode_error_cases() {
        let config_manager = create_test_config_manager();
        
        // 测试消息头太短
        let short_header = [0u8; 8];
        let mut decoder = MessageDecoder::new(&config_manager, &short_header);
        assert!(matches!(decoder.decode(), Err(MessageError::HeaderTooShort)));
        
        // 测试未知消息类型
        let unknown_msg_type = [
            0, 0, 0, 99, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 100
        ];
        let mut decoder = MessageDecoder::new(&config_manager, &unknown_msg_type);
        assert!(matches!(decoder.decode(), Err(MessageError::UnknownMessageType(99))));
        
        // 测试消息体太短
        let short_body = [
            0, 0, 0, 40, 0, 0, 0, 1, 0, 0, 0, 10, // 声明body长度为10
            1, 2, 3, 4, 5, // 但实际只有5字节
        ];
        let mut decoder = MessageDecoder::new(&config_manager, &short_body);
        assert!(matches!(decoder.decode(), Err(MessageError::BodyTooShort)));
    }

    #[test]
    fn test_decode_edge_values() {
        let config_manager = create_test_config_manager();
        
        // 创建包含边界值的消息
        let mut message = Message::new(100, 0);
        message.add_field("TestU8".to_string(), FieldValue::U8(0));
        message.add_field("TestU16".to_string(), FieldValue::U16(0));
        message.add_field("TestU32".to_string(), FieldValue::U32(0));
        message.add_field("TestU64".to_string(), FieldValue::U64(0));
        message.add_field("TestI64".to_string(), FieldValue::I64(0));
        message.add_field("TestChar".to_string(), FieldValue::Str("".to_string()));
        message.add_field("TestPrice".to_string(), FieldValue::Float(0.0));
        message.add_field("TestQuantity".to_string(), FieldValue::Float(0.0));
        message.add_field("TestAmount".to_string(), FieldValue::Float(0.0));
        message.add_field("TestDate".to_string(), FieldValue::U32(20000101));
        message.add_field("TestNTime".to_string(), FieldValue::U64(0));
        
        // 编码和解码
        let mut encoder = MessageEncoder::new(&config_manager);
        let encoded_data = encoder.encode(&message).unwrap();
        
        let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
        let decoded_message = decoder.decode().unwrap();
        
        // 验证边界值
        assert_eq!(decoded_message.get_field("TestU8").unwrap().as_u8().unwrap(), 0);
        assert_eq!(decoded_message.get_field("TestU16").unwrap().as_u16().unwrap(), 0);
        assert_eq!(decoded_message.get_field("TestU32").unwrap().as_u32().unwrap(), 0);
        assert_eq!(decoded_message.get_field("TestU64").unwrap().as_u64().unwrap(), 0);
        assert_eq!(decoded_message.get_field("TestI64").unwrap().as_i64().unwrap(), 0);
        
        let price = decoded_message.get_field("TestPrice").unwrap().as_f64().unwrap();
        assert!((price - 0.0).abs() < 0.00001);
        
        assert_eq!(decoded_message.get_field("TestDate").unwrap().as_u32().unwrap(), 20000101);
        assert_eq!(decoded_message.get_field("TestNTime").unwrap().as_u64().unwrap(), 0);
    }
}