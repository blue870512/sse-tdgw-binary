## 📘 TDGW Binary 编解码技术设计文档

版本：v1.0
作者：系统生成
时间：2025年6月

---

### 一、🎯 项目目标

构建一个稳定、可扩展的 TDGW Binary 协议编解码框架，支持：

* 动态读取消息结构配置（XML）
* 支持普通字段、结构体数组、扩展字段
* 基于 MsgType 进行消息识别与路由
* 支持类型安全的结构解析和序列化
* 适配 Rust/Go/C++ 等强类型语言

---

### 二、📦 系统架构概览

```
+---------------------+
|     XML 配置加载     |
+---------+-----------+
          |
          v
+---------------------+
|   消息结构注册表     | <-- HashMap<MsgType, MessageDef>
+---------------------+
          |
    编码器 | 解码器
          v
+---------------------+
|   二进制编解码逻辑   |
+---------------------+
          |
        用户接口层
```

---

### 三、🧱 模块结构设计

```text
tdgw_codec/
├── config/         # XML 配置加载模块
│   └── xml_parser.rs
├── types/          # 协议类型定义
│   └── field.rs
│   └── message.rs
├── codec/          # 编码/解码器实现
│   └── decoder.rs
│   └── encoder.rs
├── utils/          # 字节读取工具/错误处理
│   └── bytes.rs
│   └── error.rs
├── main.rs         # 示例调用或 CLI 工具
```

### 四、🧩 消息结构配置（XML）

参考 [message-xml-format.md](./message-xml-format.md)

---

### 五、🔧 数据结构设计（Rust示意）

```rust
enum FieldType {
    Char { length: u32 },
    U8,
    U16,
    U32,
    U64,
    I64,
    Price,
    Quantity,
    Amount,
    Date,
    NTime,
    Array { length_field: String, structure: Vec<FieldDef> },
}

struct FieldDef {
    name: String,
    field_type: FieldType,
    desc: Option<String>,
}

struct MessageDef {
    msg_type: u16,
    name: String,
    fields: Vec<FieldDef>,
    extensions: HashMap<u32, Vec<FieldDef>>, // BizID → Fields
}
```

---

### 六、📥 解码流程（示意）

1. 读取 MsgType（u16） → 查找 MessageDef
2. 按顺序逐字段解码：

   * 基础类型：按类型和长度读取
   * 数组类型：

     * 读取 length\_field 字段值
     * 依次解析 struct 项
3. 若存在 BizID 扩展：

   * 读取 BizID 值
   * 按配置附加解码扩展字段

---

### 七、📤 编码流程

1. 获取 MessageDef
2. 遍历字段列表，将结构体字段按顺序写入二进制缓冲区：

   * 基本类型：to\_le\_bytes()
   * Char：定长填充
   * 数组：先写 length\_field，然后迭代 struct 项
3. 若启用扩展字段：附加对应 BizID 的扩展内容

---

### 八、✅ 支持的字段类型

| 类型名         | 说明               |
| ----------- | ---------------- |
| `char`      | 字符串，需指定 `length` |
| `u8`\~`u64` | 无符号整数            |
| `i64`       | 有符号整数            |
| `price`     | 定点价格             |
| `quantity`  | 交易数量             |
| `amount`    | 金额               |
| `date`      | 交易日期，格式 YYYYMMDD |
| `ntime`     | 纳秒时间戳            |
| `array`     | 结构体数组类型          |

---

### 九、📚 配置加载器设计

* 使用 XML parser（如 Rust 的 `quick-xml`、Go 的 `encoding/xml`）反序列化为 `MessageDef`
* 将所有 `message` 存入 `HashMap<MsgType, MessageDef>`
* 支持运行时热加载或版本切换

---

### 十、🧪 测试策略

* 编解码一致性测试：结构 ↔ 二进制 往返一致
* 配置有效性测试：缺字段、类型不匹配报警
* 回归测试：对照真实生产报文验证解码结果

---

### 十一、📈 性能建议

* 避免动态反射，预编译结构定义为静态结构体
* 结构体数组建议按批次预分配空间（Vec::with\_capacity）
* 若性能要求高，可使用自定义 ByteReader 读写器按偏移解码

---

### 十二、📌 示例用途

* 证券交易网关报文解码
* 监管系统异构数据接入
* 协议兼容性回溯测试
* 自动生成接口文档 / WireLog 校验工具

