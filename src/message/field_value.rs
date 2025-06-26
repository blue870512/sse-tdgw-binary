use std::fmt;

/// 字段值枚举，表示消息中各种类型的字段值
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// 无符号8位整数
    U8(u8),
    /// 无符号16位整数
    U16(u16),
    /// 无符号32位整数
    U32(u32),
    /// 无符号64位整数
    U64(u64),
    /// 有符号64位整数
    I64(i64),
    /// 浮点数类型
    Float(f64),
    /// 字符串类型
    Str(String),
    /// 数组类型，表示嵌套的字段值数组
    Array(Vec<Vec<FieldValue>>),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::U8(v) => write!(f, "{}", v),
            FieldValue::U16(v) => write!(f, "{}", v),
            FieldValue::U32(v) => write!(f, "{}", v),
            FieldValue::U64(v) => write!(f, "{}", v),
            FieldValue::I64(v) => write!(f, "{}", v),
            FieldValue::Float(v) => write!(f, "{}", v),
            FieldValue::Str(v) => write!(f, "{}", v),
            FieldValue::Array(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    write!(f, "{{")?;
                    for (isub, sub_item) in item.iter().enumerate() {
                        if isub > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", sub_item)?;
                    }
                    write!(f, "}}")?;

                    if i < v.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            },
        }
    }
}

impl FieldValue {
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            FieldValue::U8(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            FieldValue::U16(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            FieldValue::U32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            FieldValue::U64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            FieldValue::I64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            FieldValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            FieldValue::Str(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Vec<FieldValue>>> {
        match self {
            FieldValue::Array(v) => Some(v),
            _ => None,
        }
    }
}

// 实现各种类型到 FieldValue 的转换
impl From<u8> for FieldValue {
    fn from(value: u8) -> Self {
        FieldValue::U8(value)
    }
}

impl From<u16> for FieldValue {
    fn from(value: u16) -> Self {
        FieldValue::U16(value)
    }
}

impl From<u32> for FieldValue {
    fn from(value: u32) -> Self {
        FieldValue::U32(value)
    }
}

impl From<u64> for FieldValue {
    fn from(value: u64) -> Self {
        FieldValue::U64(value)
    }
}

impl From<i64> for FieldValue {
    fn from(value: i64) -> Self {
        FieldValue::I64(value)
    }
}

impl From<f64> for FieldValue {
    fn from(value: f64) -> Self {
        FieldValue::Float(value)
    }
}

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        FieldValue::Str(value.to_string())
    }
}

impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        FieldValue::Str(value)
    }
}

impl Into<u8> for FieldValue {
    fn into(self) -> u8 {
        match self {
            FieldValue::U8(v) => v,
            _ => panic!("Cannot convert FieldValue to u8"),
        }
    }
}

impl Into<u16> for FieldValue {
    fn into(self) -> u16 {
        match self {
            FieldValue::U16(v) => v,
            _ => panic!("Cannot convert FieldValue to u16"),
        }
    }
}

impl Into<u32> for FieldValue {
    fn into(self) -> u32 {
        match self {
            FieldValue::U32(v) => v,
            _ => panic!("Cannot convert FieldValue to u32"),
        }
    }
}

impl Into<u64> for FieldValue {
    fn into(self) -> u64 {
        match self {
            FieldValue::U64(v) => v,
            _ => panic!("Cannot convert FieldValue to u64"),
        }
    }
}

impl Into<i64> for FieldValue {
    fn into(self) -> i64 {
        match self {
            FieldValue::I64(v) => v,
            _ => panic!("Cannot convert FieldValue to i64"),
        }
    }
}

impl Into<f64> for FieldValue {
    fn into(self) -> f64 {
        match self {
            FieldValue::Float(v) => v,
            _ => panic!("Cannot convert FieldValue to f64"),
        }
    }
}
