use indexmap::IndexMap;
use crate::message::FieldValue;

/// 解析后的消息结构，使用 IndexMap 保持字段的插入顺序
#[derive(Debug, Clone)]
pub struct Message {
    /// 消息类型
    pub msg_type: u32,
    /// 序列号
    pub seq_num: u32,
    /// 消息字段，使用 IndexMap 保持字段的插入顺序
    pub fields: IndexMap<String, FieldValue>,
}

impl Message {
    /// 创建一个新的消息实例
    pub fn new(msg_type: u32, seq_num: u32) -> Self {
        Self {
            msg_type,
            seq_num,
            fields: IndexMap::new(),
        }
    }

    /// 添加字段到消息中
    pub fn add_field(&mut self, name: String, value: FieldValue) {
        self.fields.insert(name, value);
    }

    /// 获取字段值
    pub fn get_field(&self, name: &str) -> Option<&FieldValue> {
        self.fields.get(name)
    }

    /// 获取字段数量
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// 检查是否包含指定字段
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }
}