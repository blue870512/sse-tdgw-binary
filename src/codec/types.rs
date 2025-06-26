use crate::util::error::CodecResult;

pub type Result<T> = CodecResult<T>;

pub(super) const TYPE_PRICE_MAX: i64 = 9_999_999_999_999;
pub(super) const TYPE_PRICE_MIN: i64 = -9_999_999_999_999;
pub(super) const TYPE_QUANTITY_MAX: i64 = 999_999_999_999_999;
pub(super) const TYPE_QUANTITY_MIN: i64 = -999_999_999_999_999;
pub(super) const TYPE_AMOUNT_MAX: i64 = 999_999_999_999_999_999;
pub(super) const TYPE_AMOUNT_MIN: i64 = -999_999_999_999_999_999;

pub(super) const TYPE_PRICE_SCALE: f64 = 1e5;
pub(super) const TYPE_QUANTITY_SCALE: f64 = 1e3;
pub(super) const TYPE_AMOUNT_SCALE: f64 = 1e5;

pub(super) fn validate_price(value: i64) -> bool {
    value >= TYPE_PRICE_MIN && value <= TYPE_PRICE_MAX
}

pub(super) fn validate_quantity(value: i64) -> bool {
    value >= TYPE_QUANTITY_MIN && value <= TYPE_QUANTITY_MAX
}

pub(super) fn validate_amount(value: i64) -> bool {
    value >= TYPE_AMOUNT_MIN && value <= TYPE_AMOUNT_MAX
}

/// 验证NTime格式是否为有效的HHMMSSsssnnnn格式
/// HH: 小时范围 00-23
/// MM: 分钟范围 00-59
/// SS: 秒范围 00-59
/// sss: 毫秒范围 000-999
/// nnnn: 百纳秒范围 0000-9999
pub(super) fn validate_ntime_format(ntime_value: u64) -> bool {
    // 提取各个时间组件
    let hour = (ntime_value / 100000000000) % 100;
    let minute = (ntime_value / 1000000000) % 100;
    let second = (ntime_value / 10000000) % 100;
    let millisecond = (ntime_value / 10000) % 1000;
    let hundred_nanosecond = ntime_value % 10000;
    
    // 验证小时范围 00-23
    if hour > 23 {
        return false;
    }
    
    // 验证分钟范围 00-59
    if minute > 59 {
        return false;
    }
    
    // 验证秒范围 00-59
    if second > 59 {
        return false;
    }
    
    // 验证毫秒范围 000-999
    if millisecond > 999 {
        return false;
    }
    
    // 验证百纳秒范围 0000-9999
    if hundred_nanosecond > 9999 {
        return false;
    }
    
    true
}

/// 验证Date格式是否为有效的YYYYMMDD格式
/// YYYY: 年份范围 0000-9999
/// MM: 月份范围 01-12
/// DD: 日期范围 01-31
pub(super) fn validate_date_format(date_value: u32) -> bool {
    // 提取年、月、日
    let year = date_value / 10000;
    let month = (date_value % 10000) / 100;
    let day = date_value % 100;
    
    // 验证年份范围 0000-9999
    if year > 9999 {
        return false;
    }
    
    // 验证月份范围 01-12
    if month < 1 || month > 12 {
        return false;
    }
    
    // 验证日期范围 01-31
    if day < 1 || day > 31 {
        return false;
    }
    
    // 进一步验证每月的天数
    match month {
        2 => {
            // 2月份，考虑闰年
            let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if is_leap_year {
                day <= 29
            } else {
                day <= 28
            }
        },
        4 | 6 | 9 | 11 => {
            // 4、6、9、11月只有30天
            day <= 30
        },
        _ => {
            // 其他月份有31天
            day <= 31
        }
    }
}

// 消息头部结构
#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub msg_type: u32,  // 消息类型
    pub seq_num: u32,   // 序列号
    pub body_length: u32, // 消息体长度
}

impl MessageHeader {
    pub fn new(msg_type: u32, seq_num: u32, body_length: u32) -> Self {
        Self {
            msg_type,
            seq_num,
            body_length,
        }
    }
    
    // 头部固定长度为12字节
    pub const SIZE: usize = 12;
}

// 编解码模块内部使用的字段类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum FieldTypeCode {
    U8,
    U16,
    U32,
    U64,
    I64,
    Char(usize),  // 包含长度信息
    Price,
    Quantity,
    Amount,
    Date,
    NTime,
    Array,  // 数组类型
}

// 编解码模块内部使用的字段值枚举
#[derive(Debug, Clone)]
pub enum FieldValueData {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I64(i64),
    Str(String),
    Array(Vec<FieldValueData>),  // 数组类型
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_date_format() {
        // 测试有效的Date格式
        assert!(validate_date_format(20231201)); // 2023-12-01
        assert!(validate_date_format(20240229)); // 2024-02-29 (闰年)
        assert!(validate_date_format(20000101)); // 2000-01-01
        assert!(validate_date_format(99991231)); // 9999-12-31
        assert!(validate_date_format(10101)); // 0001-01-01
        assert!(validate_date_format(20230630)); // 2023-06-30
        assert!(validate_date_format(20230430)); // 2023-04-30
        
        // 测试边界值
        assert!(validate_date_format(20230101)); // 年初
        assert!(validate_date_format(20231231)); // 年末
        assert!(validate_date_format(20230131)); // 1月31日
        assert!(validate_date_format(20230228)); // 平年2月28日
        assert!(validate_date_format(20240229)); // 闰年2月29日
        assert!(validate_date_format(20230331)); // 3月31日
        assert!(validate_date_format(20230430)); // 4月30日
        assert!(validate_date_format(20230531)); // 5月31日
        assert!(validate_date_format(20230630)); // 6月30日
        assert!(validate_date_format(20230731)); // 7月31日
        assert!(validate_date_format(20230831)); // 8月31日
        assert!(validate_date_format(20230930)); // 9月30日
        assert!(validate_date_format(20231031)); // 10月31日
        assert!(validate_date_format(20231130)); // 11月30日
        assert!(validate_date_format(20231231)); // 12月31日
        
        // 测试无效的年份 (>9999)
        assert!(!validate_date_format(100000101)); // 10000-01-01
        assert!(!validate_date_format(123450101)); // 12345-01-01
        
        // 测试无效的月份 (<1 或 >12)
        assert!(!validate_date_format(20230001)); // 2023-00-01
        assert!(!validate_date_format(20231301)); // 2023-13-01
        assert!(!validate_date_format(20231401)); // 2023-14-01
        assert!(!validate_date_format(20239901)); // 2023-99-01
        
        // 测试无效的日期 (<1 或 >31)
        assert!(!validate_date_format(20230100)); // 2023-01-00
        assert!(!validate_date_format(20230132)); // 2023-01-32
        assert!(!validate_date_format(20230199)); // 2023-01-99
        
        // 测试2月份的特殊情况
        // 平年2月29日无效
        assert!(!validate_date_format(20230229)); // 2023-02-29 (平年)
        assert!(!validate_date_format(21000229)); // 2100-02-29 (非闰年，能被100整除但不能被400整除)
        assert!(!validate_date_format(19000229)); // 1900-02-29 (非闰年，能被100整除但不能被400整除)
        
        // 2月30日和31日始终无效
        assert!(!validate_date_format(20230230)); // 2023-02-30
        assert!(!validate_date_format(20240230)); // 2024-02-30 (即使闰年)
        assert!(!validate_date_format(20230231)); // 2023-02-31
        assert!(!validate_date_format(20240231)); // 2024-02-31 (即使闰年)
        
        // 测试4、6、9、11月的31日无效
        assert!(!validate_date_format(20230431)); // 2023-04-31
        assert!(!validate_date_format(20230631)); // 2023-06-31
        assert!(!validate_date_format(20230931)); // 2023-09-31
        assert!(!validate_date_format(20231131)); // 2023-11-31
        
        // 测试闰年规则
        // 能被4整除但不能被100整除的年份是闰年
        assert!(validate_date_format(20240229)); // 2024是闰年
        assert!(validate_date_format(20280229)); // 2028是闰年
        
        // 能被100整除但不能被400整除的年份不是闰年
        assert!(!validate_date_format(21000229)); // 2100不是闰年
        assert!(!validate_date_format(22000229)); // 2200不是闰年
        assert!(!validate_date_format(23000229)); // 2300不是闰年
        
        // 能被400整除的年份是闰年
        assert!(validate_date_format(20000229)); // 2000是闰年
        assert!(validate_date_format(24000229)); // 2400是闰年
        
        // 测试特殊边界情况
        assert!(!validate_date_format(0)); // 0000-00-00 会失败，因为月份和日期都是0
        assert!(validate_date_format(10101)); // 0001-01-01 最小有效日期
    }

    #[test]
    fn test_validate_ntime_format() {
        // 测试有效的NTime格式
        assert!(validate_ntime_format(0)); // 00:00:00.000.0000
        assert!(validate_ntime_format(120000000000)); // 12:00:00.000.0000
        assert!(validate_ntime_format(2359599999999)); // 23:59:59.999.9999
        assert!(validate_ntime_format(930451234560)); // 09:30:45.123.4560
        assert!(validate_ntime_format(1430225007500)); // 14:30:22.500.7500
        
        // 测试边界值
        assert!(validate_ntime_format(2300000000000)); // 23:00:00.000.0000 (最大小时)
        assert!(validate_ntime_format(590000000)); // 00:59:00.000.0000 (最大分钟)
        assert!(validate_ntime_format(59000000000)); // 00:00:59.000.0000 (最大秒)
        assert!(validate_ntime_format(9990000)); // 00:00:00.999.0000 (最大毫秒)
        assert!(validate_ntime_format(9999)); // 00:00:00.000.9999 (最大百纳秒)
        
        // 测试无效的小时 (>23)
        assert!(!validate_ntime_format(2400000000000)); // 24:00:00.000.0000
        assert!(!validate_ntime_format(2500000000000)); // 25:00:00.000.0000
        assert!(!validate_ntime_format(9900000000000)); // 99:00:00.000.0000
        
        // 测试无效的分钟 (>59)
        assert!(!validate_ntime_format(60000000000)); // 00:60:00.000.0000
        assert!(!validate_ntime_format(61000000000)); // 00:61:00.000.0000
        assert!(!validate_ntime_format(99000000000)); // 00:99:00.000.0000
        
        // 测试无效的秒 (>59)
        assert!(!validate_ntime_format(600000000)); // 00:00:60.000.0000
        assert!(!validate_ntime_format(610000000)); // 00:00:61.000.0000
        assert!(!validate_ntime_format(990000000)); // 00:00:99.000.0000
        
        // 测试组合无效情况
        assert!(!validate_ntime_format(2461999999999)); // 24:61:99.999.9999 (多个字段都无效)
        assert!(!validate_ntime_format(2562999999999)); // 25:62:99.999.9999 (多个字段都无效)
        
        // 测试特殊值
        assert!(validate_ntime_format(1)); // 00:00:00.000.0001
        assert!(validate_ntime_format(10)); // 00:00:00.000.0010
        assert!(validate_ntime_format(100)); // 00:00:00.000.0100
        assert!(validate_ntime_format(1000)); // 00:00:00.000.1000
        assert!(validate_ntime_format(10000)); // 00:00:00.001.0000
        assert!(validate_ntime_format(100000)); // 00:00:00.010.0000
        assert!(validate_ntime_format(1000000)); // 00:00:01.000.0000
        assert!(validate_ntime_format(10000000)); // 00:00:10.000.0000
        assert!(validate_ntime_format(100000000)); // 00:01:00.000.0000
        assert!(validate_ntime_format(1000000000)); // 00:10:00.000.0000
        assert!(validate_ntime_format(10000000000)); // 01:00:00.000.0000
        assert!(validate_ntime_format(100000000000)); // 10:00:00.000.0000
    }
}