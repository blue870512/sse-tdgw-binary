/// 消息头部结构
#[derive(Debug, Clone)]
pub struct MessageHeader {
    /// 消息类型
    pub msg_type: u32,
    /// 序列号
    pub seq_num: u32,
    /// 消息体长度
    pub body_length: u32,
}

impl MessageHeader {
    /// 创建一个新的消息头部
    pub fn new(msg_type: u32, seq_num: u32, body_length: u32) -> Self {
        Self {
            msg_type,
            seq_num,
            body_length,
        }
    }
    
    /// 头部固定长度为12字节
    pub const SIZE: usize = 12;
}