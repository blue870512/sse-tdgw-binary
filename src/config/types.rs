use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "messages")]
pub struct MessageConfig {
    #[serde(rename = "message", default)]
    pub messages: Vec<MessageDef>,
}

// 消息定义结构
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "message")]
pub struct MessageDef {
    #[serde(rename = "@type")]
    pub msg_type: u32,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "field", default)]
    pub fields: Vec<FieldDef>,
    #[serde(rename = "extension", default)]
    pub extensions: Vec<BizExtension>,
}

// 字段类型枚举
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    I64,
    Char,      // 固定长度 ASCII 字符串
    Price,     // N13(5) 精度价格
    Quantity,  // N15(3) 精度数量
    Amount,    // N18(5) 金额
    Date,      // YYYYMMDD 格式日期
    NTime,     // HHMMSSsssnnnn 纳秒时间戳
    Array,     // 数组类型
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaseFieldDef {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@type")]
    pub r#type: FieldType,
    #[serde(rename = "@length", skip_serializing_if = "Option::is_none", default)]
    #[serde(deserialize_with = "deserialize_length")]
    pub length: Option<usize>, // for Char
    #[serde(rename = "@desc")]
    pub desc: Option<String>,  // 字段描述
}

// 自定义反序列化函数，用于处理字符串形式的length属性
fn deserialize_length<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrUsize;

    impl<'de> serde::de::Visitor<'de> for StringOrUsize {
        type Value = Option<usize>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or usize")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match value.parse::<usize>() {
                Ok(v) => Ok(Some(v)),
                Err(_) => Err(E::custom(format!("failed to parse {} as usize", value))),
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value as usize))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }
    }

    deserializer.deserialize_option(StringOrUsize)
}

// 字段定义结构
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FieldDef {
    #[serde(flatten)]
    pub base: BaseFieldDef,

    #[serde(rename = "length_field", skip_serializing_if = "Option::is_none")]
    pub length_field: Option<BaseFieldDef>,
    #[serde(rename = "struct", skip_serializing_if = "Option::is_none")]
    pub r#struct: Option<StructDef>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StructDef {
    #[serde(rename = "field")]
    pub fields: Vec<BaseFieldDef>,
}

// 业务扩展结构
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BizExtension {
    #[serde(rename = "@biz_id")]
    pub biz_id: u32,
    #[serde(rename = "field")]
    pub fields: Vec<BaseFieldDef>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let xml = r#"
        <message type="58" name="NewOrderSingle">
            <field name="BizID" type="u32" desc="业务ID" />
            <field name="ClOrdID" type="char" length="10" desc="客户订单ID" />
            <field name="Price" type="price" desc="订单价格" />
            <field name="OrderQty" type="quantity" desc="订单数量" />
            <field name="SyncResponses" type="array" desc="同步响应项数组">
                <length_field name="NoGroups" type="u16" desc="同步响应项个数"/>
                <struct>
                    <field name="Pbu" type="char" length="8" desc="登录或订阅用 PBU"/>
                    <field name="SetID" type="u32" desc="平台内分区号"/>
                    <field name="BeginReportIndex" type="u64" desc="分区回报序号起点"/>
                    <field name="EndReportIndex" type="u64" desc="分区最大回报序号"/>
                    <field name="RejReason" type="u32" desc="拒绝码"/>
                    <field name="Text" type="char" length="64" desc="描述信息"/>
                </struct>
            </field>
            <extension biz_id="300060">
                <field name="Custodian" type="char" length="3" desc="放式基金转托管的目标方代理人。对方的销售人代码000-999，不足3位左侧补 0."/>
            </extension>
            <extension biz_id="300070">
                <field name="DividendSelect" type="char" length="1" desc="分红方式：U=红利转投, C=现金分红"/>
            </extension>
        </message>
        "#;

        // 需要先在 Cargo.toml 中添加 serde_xml_rs 依赖
        let message: MessageDef = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(message.msg_type, 58);
        assert_eq!(message.name, "NewOrderSingle");
        assert_eq!(message.fields.len(), 5);
        assert_eq!(message.fields[4].base.name, "SyncResponses");
        assert_eq!(message.fields[4].base.r#type, FieldType::Array);
        assert_eq!(message.fields[4].length_field.as_ref().unwrap().name, "NoGroups");
        assert_eq!(message.fields[4].length_field.as_ref().unwrap().r#type, FieldType::U16);
        assert_eq!(message.fields[4].r#struct.as_ref().unwrap().fields.len(), 6);
        assert_eq!(message.extensions.len(), 2);
        assert_eq!(message.extensions[0].biz_id, 300060);
        assert_eq!(message.extensions[0].fields.len(), 1);
        assert_eq!(message.extensions[1].biz_id, 300070);
        assert_eq!(message.extensions[1].fields.len(), 1);
    }

    #[test]
    fn test_serialize() {
        let message = MessageDef {
            msg_type: 58,
            name: "NewOrderSingle".to_string(),
            fields: vec![
                FieldDef {
                    base: BaseFieldDef {
                        name: "BizID".to_string(),
                        r#type: FieldType::U32,
                        length: None,
                        desc: Some("业务ID".to_string()),
                    },
                    length_field: None,
                    r#struct: None,
                },
                FieldDef {
                    base: BaseFieldDef {
                        name: "ClOrdID".to_string(),
                        r#type: FieldType::Char,
                        length: Some(10),
                        desc: Some("客户订单ID".to_string()),
                    },
                    length_field: None,
                    r#struct: None,
                },
                FieldDef {
                    base: BaseFieldDef {
                        name: "Price".to_string(),
                        r#type: FieldType::Price,
                        length: None,
                        desc: Some("订单价格".to_string()),
                    },
                    length_field: None,
                    r#struct: None,
                },
                FieldDef {
                    base: BaseFieldDef {
                        name: "OrderQty".to_string(),
                        r#type: FieldType::Quantity,
                        length: None,
                        desc: Some("订单数量".to_string()),
                    },
                    length_field: None,
                    r#struct: None,
                },
            ],
            extensions: vec![],
        };

        let s = quick_xml::se::to_string(&message).unwrap();
        println!("{}", s);
    }
}
