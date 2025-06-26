# SSE TDGW Binary 示例程序

本目录包含了 SSE TDGW Binary 库的示例程序，展示如何使用该库进行消息的编解码操作。

## 示例程序列表

### 1. encode_message.rs
基础的消息编码示例，使用硬编码的配置字符串。

**功能特点：**
- 演示基本的消息编码流程
- 包含登录、订单、数组消息等多种类型
- 使用内嵌的XML配置

**运行方法：**
```bash
cargo run --example encode_message
```

### 2. config_file_example.rs
完整的消息编解码示例，支持从外部配置文件读取消息定义。

**功能特点：**
- 从外部XML配置文件读取消息定义
- 完整的编码和解码流程演示
- 往返一致性验证
- 扩展字段支持测试
- 性能基准测试
- 错误处理和参数验证

**运行方法：**
```bash
# 使用项目中的配置文件
cargo run --example config_file_example config/sse-message.xml

# 或使用自定义配置文件
cargo run --example config_file_example /path/to/your/config.xml
```

## 配置文件格式

配置文件使用XML格式定义消息结构，例如：

```xml
<messages>
  <message type="40" name="Logon">
    <field name="SenderCompID" type="char" length="32" desc="发送方代码"/>
    <field name="TargetCompID" type="char" length="32" desc="接收方代码"/>
    <field name="HeartBtInt" type="u16" desc="心跳间隔（秒）"/>
    <!-- 更多字段... -->
  </message>
  
  <message type="58" name="NewOrderSingle">
    <field name="BizID" type="u32" desc="业务代码"/>
    <field name="ClOrdID" type="char" length="10" desc="订单编号"/>
    <!-- 扩展字段支持 -->
    <extension biz_id="300060">
      <field name="Custodian" type="char" length="3" desc="托管方代码"/>
      <field name="FundType" type="u8" desc="基金类型"/>
    </extension>
  </message>
</messages>
```

## 支持的字段类型

| 类型 | 描述 | 示例 |
|------|------|------|
| `u8` | 8位无符号整数 | `255` |
| `u16` | 16位无符号整数 | `65535` |
| `u32` | 32位无符号整数 | `4294967295` |
| `u64` | 64位无符号整数 | `18446744073709551615` |
| `i8` | 8位有符号整数 | `-128` |
| `i16` | 16位有符号整数 | `-32768` |
| `i32` | 32位有符号整数 | `-2147483648` |
| `i64` | 64位有符号整数 | `-9223372036854775808` |
| `char` | 字符串 | `"HELLO"` (需要指定length) |
| `price` | 价格类型 | `12.345` (内部存储为i64) |
| `quantity` | 数量类型 | `1000.000` (内部存储为i64) |
| `amount` | 金额类型 | `12345.67890` (内部存储为i64) |
| `date` | 日期类型 | `20231201` (YYYYMMDD格式) |
| `ntime` | 时间类型 | `12345678901234` (纳秒时间戳) |
| `array` | 数组类型 | 包含length_field和struct定义 |

## 扩展字段 (Extension)

扩展字段允许根据业务类型(biz_id)动态添加字段：

```xml
<extension biz_id="300060">
  <field name="Custodian" type="char" length="3" desc="托管方代码"/>
  <field name="FundType" type="u8" desc="基金类型"/>
</extension>
```

在代码中使用：
```rust
// 设置业务类型
message.add_field("BizID".to_string(), FieldValue::U32(300060));

// 添加对应的扩展字段
message.add_field("Custodian".to_string(), FieldValue::Str("001".to_string()));
message.add_field("FundType".to_string(), FieldValue::U8(1));
```

## 数组字段

数组字段支持复杂的嵌套结构：

```xml
<field name="SyncRequests" type="array" desc="同步请求项数组">
  <length_field name="NoGroups" type="u16" desc="数组长度"/>
  <struct>
    <field name="Pbu" type="char" length="8" desc="PBU代码"/>
    <field name="SetID" type="u32" desc="分区号"/>
    <field name="BeginReportIndex" type="u64" desc="起始序号"/>
  </struct>
</field>
```

在代码中使用：
```rust
let array_data = vec![
    vec![
        FieldValue::Str("PBU00001".to_string()),
        FieldValue::U32(1),
        FieldValue::U64(100),
    ],
    vec![
        FieldValue::Str("PBU00002".to_string()),
        FieldValue::U32(2),
        FieldValue::U64(200),
    ],
];
message.add_field("SyncRequests".to_string(), FieldValue::Array(array_data));
```

## 性能特点

- **高效编码**: 支持每秒数千次编码操作
- **快速解码**: 支持每秒数千次解码操作
- **内存优化**: 最小化内存分配和拷贝
- **零拷贝**: 解码过程中尽可能避免数据拷贝

## 错误处理

示例程序包含完整的错误处理：

- 配置文件不存在检查
- XML解析错误处理
- 编码/解码错误处理
- 数据一致性验证

## 调试技巧

1. **查看编码数据**: 使用十六进制格式输出编码后的字节数据
2. **字段验证**: 比较编码前后的字段值确保一致性
3. **性能分析**: 使用内置的性能测试功能
4. **日志输出**: 详细的步骤日志帮助定位问题

## 扩展开发

基于这些示例，你可以：

1. 创建自定义的消息类型
2. 实现特定业务逻辑的编解码器
3. 集成到现有的交易系统中
4. 添加网络传输功能
5. 实现消息持久化存储

## 注意事项

- 确保配置文件路径正确
- 字段类型必须与配置文件中定义的类型匹配
- 字符串字段需要指定正确的长度
- 数组字段的结构必须与配置中的struct定义一致
- 扩展字段只有在对应的biz_id匹配时才会被处理