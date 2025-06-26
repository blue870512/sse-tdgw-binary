# 编解码器测试套件

本目录包含了对SSE TDGW二进制消息编解码器的全面测试套件，涵盖了功能测试、性能测试和错误处理测试。

## 测试文件结构

### 1. `codec_integration_test.rs` - 基础集成测试

**目的**: 测试所有基础字段类型的编解码功能，确保往返一致性。

**覆盖的字段类型**:
- `U8`, `U16`, `U32`, `U64` - 无符号整数类型
- `I64` - 有符号64位整数
- `Char` - 固定长度字符串
- `Price` - N13(5)精度价格
- `Quantity` - N15(3)精度数量
- `Amount` - N18(5)金额
- `Date` - YYYYMMDD格式日期
- `NTime` - HHMMSSsssnnnn纳秒时间戳

**测试用例**:
- `test_encode_decode_roundtrip()` - 基本往返测试
- `test_boundary_values()` - 边界值测试（最小值）
- `test_maximum_values()` - 最大值测试
- `test_negative_values()` - 负数值测试
- `test_special_string_values()` - 特殊字符串值测试
- `test_performance()` - 基础性能测试（1000次迭代）

### 2. `array_codec_test.rs` - 数组类型测试

**目的**: 专门测试数组字段的编解码功能，包括嵌套数组。

**测试场景**:
- 简单数组（包含基础类型字段的结构体数组）
- 嵌套数组（数组中包含子数组）
- 空数组处理
- 大数组处理（100个元素）
- 数组字段类型一致性

**测试用例**:
- `test_simple_array_encode_decode()` - 简单数组测试
- `test_nested_array_encode_decode()` - 嵌套数组测试
- `test_empty_array_encode_decode()` - 空数组测试
- `test_large_array_encode_decode()` - 大数组测试
- `test_array_field_type_consistency()` - 数组字段类型一致性测试

### 3. `error_handling_test.rs` - 错误处理测试

**目的**: 测试编解码器在各种异常情况下的错误处理能力。

**错误场景**:
- 未知消息类型
- 字段类型不匹配
- 无效的日期格式
- 无效的时间格式
- 价格超出范围
- 字符串长度超出限制
- 损坏的二进制数据
- 空二进制数据
- 校验和不匹配
- 消息体长度不匹配
- 缺失必需字段

**测试用例**:
- `test_unknown_message_type()` - 未知消息类型错误
- `test_field_type_mismatch()` - 字段类型不匹配错误
- `test_invalid_date_format()` - 无效日期格式错误
- `test_invalid_ntime_format()` - 无效时间格式错误
- `test_price_out_of_range()` - 价格超出范围错误
- `test_string_length_exceeded()` - 字符串长度超出错误
- `test_corrupted_binary_data()` - 损坏数据错误
- `test_empty_binary_data()` - 空数据错误
- `test_checksum_mismatch()` - 校验和不匹配错误
- `test_body_length_mismatch()` - 消息体长度不匹配错误
- `test_missing_required_fields()` - 缺失字段错误

### 4. `performance_benchmark.rs` - 性能基准测试

**目的**: 测试编解码器在不同场景下的性能表现，提供性能基准。

**测试消息类型**:
- **小消息**: 2个字段（ID + 值）
- **中等消息**: 10个字段（包含各种类型）
- **大消息**: 25个字段（模拟真实的市场数据消息）
- **数组消息**: 包含可变大小数组的消息

**性能测试**:
- `test_small_message_encode_performance()` - 小消息编码性能（>1000 ops/sec）
- `test_small_message_decode_performance()` - 小消息解码性能（>1000 ops/sec）
- `test_medium_message_roundtrip_performance()` - 中等消息往返性能（>500 ops/sec）
- `test_large_message_roundtrip_performance()` - 大消息往返性能（>100 ops/sec）
- `test_small_array_message_performance()` - 小数组消息性能（>200 ops/sec）
- `test_large_array_message_performance()` - 大数组消息性能（>10 ops/sec）
- `test_batch_message_processing_performance()` - 批量消息处理性能
- `test_memory_efficiency()` - 内存使用效率测试
- `test_comprehensive_performance_report()` - 综合性能报告

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定测试文件
```bash
# 基础集成测试
cargo test --test codec_integration_test

# 数组测试
cargo test --test array_codec_test

# 错误处理测试
cargo test --test error_handling_test

# 性能基准测试
cargo test --test performance_benchmark
```

### 运行特定测试用例
```bash
# 运行往返测试
cargo test test_encode_decode_roundtrip

# 运行性能测试
cargo test test_performance

# 运行错误处理测试
cargo test test_unknown_message_type
```

### 运行性能测试（发布模式）
```bash
# 性能测试建议在发布模式下运行以获得准确结果
cargo test --release --test performance_benchmark
```

### 显示测试输出
```bash
# 显示测试过程中的println!输出
cargo test -- --nocapture

# 显示特定测试的输出
cargo test test_comprehensive_performance_report -- --nocapture
```

## 测试数据说明

### 字段值示例

**基础类型**:
- `U8`: 0 ~ 255
- `U16`: 0 ~ 65535
- `U32`: 0 ~ 4294967295
- `U64`: 0 ~ 18446744073709551615
- `I64`: -9223372036854775808 ~ 9223372036854775807

**特殊类型**:
- `Char`: 固定长度ASCII字符串，如"HELLO"
- `Price`: 价格值，精度5位小数，如12345000表示1234.50000
- `Quantity`: 数量值，精度3位小数，如1234567表示1234.567
- `Amount`: 金额值，精度5位小数，如1234567890表示12345.67890
- `Date`: YYYYMMDD格式，如20231225表示2023年12月25日
- `NTime`: HHMMSSsssnnnn格式，如12345678901234表示12:34:56.789.1234

### 数组数据结构

数组消息包含长度字段和数组数据：
```rust
// 简单数组示例
array_count: 3,
simple_array: [
    [1001, "ITEM001", 12345000],  // ID, 名称, 价格
    [1002, "ITEM002", 67890000],
    [1003, "ITEM003", 99999000],
]

// 嵌套数组示例
nested_count: 2,
nested_array: [
    [100, 2, [[2001, 1111], [2002, 2222]]],  // 组ID, 子项数量, 子项数组
    [200, 3, [[3001, 3333], [3002, 4444], [3003, 5555]]],
]
```

## 性能基准

以下是在典型开发机器上的性能基准（仅供参考）：

| 消息类型 | 编码性能 | 解码性能 | 消息大小 |
|---------|---------|---------|----------|
| 小消息   | >1000 ops/sec | >1000 ops/sec | ~20 bytes |
| 中等消息 | >500 ops/sec  | >500 ops/sec  | ~100 bytes |
| 大消息   | >100 ops/sec  | >100 ops/sec  | ~300 bytes |
| 数组消息(10) | >200 ops/sec | >200 ops/sec | ~150 bytes |
| 数组消息(100) | >10 ops/sec | >10 ops/sec | ~1200 bytes |

**注意**: 实际性能可能因硬件配置、系统负载等因素而有所不同。

## 测试覆盖率

本测试套件旨在提供全面的测试覆盖：

- ✅ **功能覆盖**: 所有字段类型和数组类型
- ✅ **边界测试**: 最小值、最大值、边界条件
- ✅ **错误处理**: 各种异常情况和错误场景
- ✅ **性能测试**: 不同消息大小和复杂度的性能基准
- ✅ **往返一致性**: 编码后解码的数据完整性
- ✅ **内存效率**: 编码后数据大小的合理性

## 添加新测试

如果需要添加新的测试用例，请遵循以下原则：

1. **功能测试**: 添加到相应的测试文件中
2. **命名规范**: 使用描述性的测试函数名
3. **文档注释**: 为测试函数添加清晰的注释
4. **断言消息**: 提供有意义的断言失败消息
5. **性能测试**: 包含合理的性能期望值

### 示例测试函数

```rust
/// 测试特定场景的编解码
#[test]
fn test_specific_scenario() {
    let config_manager = create_test_config_manager();
    let message = create_test_message();
    
    // 编码
    let mut encoder = MessageEncoder::new(&config_manager);
    let encoded_data = encoder.encode(&message)
        .expect("Failed to encode message");
    
    // 解码
    let mut decoder = MessageDecoder::new(&config_manager, &encoded_data);
    let decoded_message = decoder.decode()
        .expect("Failed to decode message");
    
    // 验证
    assert_eq!(decoded_message.msg_type, message.msg_type, "Message type mismatch");
    // 更多断言...
    
    println!("✓ Specific scenario test passed");
}
```

## 故障排除

### 常见问题

1. **编译错误**: 确保所有依赖项都已正确添加到`Cargo.toml`
2. **测试失败**: 检查错误消息，可能是数据格式或类型不匹配
3. **性能测试失败**: 性能基准可能因系统负载而变化，可以调整期望值
4. **内存问题**: 大数组测试可能消耗较多内存，确保系统有足够资源

### 调试技巧

1. 使用`-- --nocapture`查看测试输出
2. 添加更多`println!`语句进行调试
3. 使用`hex::encode()`查看二进制数据的十六进制表示
4. 检查配置XML是否正确加载

## 贡献指南

欢迎为测试套件做出贡献：

1. 确保新测试遵循现有的代码风格
2. 添加适当的文档和注释
3. 验证测试在不同环境下都能通过
4. 更新本README文件以反映新增的测试