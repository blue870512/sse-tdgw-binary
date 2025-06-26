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
</messages>"#;

    const LOGIN_MESSAGE: [u8; 98] = [
        0, 0, 0, 40, 0, 0, 0, 1, 0, 0, 0, 82, 83, 69, 78, 68, 
        69, 82, 49, 50, 51, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 
        32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 84, 65, 82, 71, 
        69, 84, 52, 53, 54, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 
        32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 30, 49, 46, 
        48, 32, 32, 32, 32, 32, 1, 52, 180, 33, 0, 0, 3, 232, 0, 0, 0, 58
    ];
    
    #[test]
    fn test_decode() {
        let mut config_manager = ConfigManager::new();
        config_manager.load_from_str(CONFIG_STR).unwrap();

        let mut decoder = MessageDecoder::new(&config_manager, &LOGIN_MESSAGE);
        let message = decoder.decode().unwrap();

        assert_eq!(message.msg_type, 40);
        assert_eq!(message.seq_num, 1);
        assert_eq!(message.get_field("SenderCompID").unwrap().to_string().trim(), "SENDER123");
        let trade_date: u32 = message.get_field("TradeDate").unwrap().as_u32().unwrap();
        assert_eq!(trade_date, 20231201u32);
    }
}