## 📘 TDGW 消息格式 XML 配置设计文档

### 一、🧱 总体结构

最外层元素为 `<messages>`，内部由多个 `<message>` 节点组成，每个 `<message>` 定义一个消息类型。

```xml
<messages>
  <message type="..." name="...">
    <!-- 字段定义（field） -->
    ...
  </message>
</messages>
```

---

### 二、📦 `<message>` 节点

| 属性     | 类型     | 说明            |
| ------ | ------ | ------------- |
| `type` | u16    | 消息类型（MsgType） |
| `name` | string | 消息名称          |

#### 子节点：

* `<field>`：普通字段或数组字段
* `<extension>`（可选）：按 BizID 扩展字段，仅用于部分业务型消息

---

### 三、🧩 `<field>` 节点

普通字段使用 `type=char/u8/u16/u32/u64/price/quantity/amount/date/ntime` 等基本类型。
数组字段使用 `type="array"`，配合 `<length_field>` 与 `<struct>` 定义结构体数组。

#### 1. 普通字段格式：

```xml
<field name="ClOrdID" type="char" length="10" desc="客户订单编号"/>
```

| 属性       | 类型     | 说明                   |
| -------- | ------ | -------------------- |
| `name`   | string | 字段名                  |
| `type`   | enum   | 字段类型，如 `char`, `u32` |
| `length` | int    | （可选）char 类型的字节数      |
| `desc`   | string | （可选）字段说明             |

#### 2. 数组字段格式：

```xml
<field name="ExecList" type="array" desc="执行项数组">
  <length_field name="NoGroups" type="u16" desc="数组长度"/>
  <struct>
    <field name="ExecID" type="char" length="10" desc="执行编号"/>
    <field name="Price" type="price" desc="价格"/>
  </struct>
</field>
```

* `name`：数组字段名
* `type="array"`：表明这是结构体数组
* `<length_field>`：表示数组项个数，通常紧邻数组前
* `<struct>`：数组项结构，内嵌多个 `<field>`

---

### 四、📚 字段类型枚举（type）

| 类型名         | 说明               |
| ----------- | ---------------- |
| `char`      | 字符串，需指定 `length` |
| `u8`\~`u64` | 无符号整数            |
| `price`     | 价格（定点数）          |
| `quantity`  | 数量               |
| `amount`    | 金额               |
| `date`      | YYYYMMDD 格式日期    |
| `ntime`     | 纳秒级时间戳           |

---

### 五、🔧 `<extension>` 节点（可选）

支持按 `BizID` 扩展字段。

```xml
<extension biz_id="300060">
  <field name="Custodian" type="char" length="3" desc="托管机构"/>
</extension>
```

| 属性           | 说明           |
| ------------ | ------------ |
| `biz_id`     | 对应业务标识 BizID |
| `parent`（可选） | 对应的消息名称      |

---

### 六、📌 示例：含普通字段 + 数组 + 扩展字段

```xml
<message type="58" name="NewOrderSingle">
  <field name="BizID" type="u32" desc="业务编号"/>
  <field name="ClOrdID" type="char" length="10" desc="客户订单编号"/>

  <field name="ExecList" type="array" desc="执行信息列表">
    <length_field name="NoGroups" type="u16" desc="执行项个数"/>
    <struct>
      <field name="ExecID" type="char" length="10" desc="执行编号"/>
      <field name="Price" type="price" desc="成交价格"/>
    </struct>
  </field>

  <extension biz_id="300060">
    <field name="Custodian" type="char" length="3" desc="托管机构"/>
  </extension>
</message>
```

---

### 七、🔄 使用建议

* **代码生成**：可自动生成 Rust、C、Go 等结构体定义；
* **配置校验**：根据字段描述可辅助协议测试工具校验合法性；
* **灵活扩展**：支持将后续新增字段、数组结构统一进配置；
