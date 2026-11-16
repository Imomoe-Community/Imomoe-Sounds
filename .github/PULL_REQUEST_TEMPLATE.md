## 变更说明
简要概括本次 Pull Request 解决的问题或实现的功能。

## 变更类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 配置或文档优化
- [ ] 重构或性能改进
- [ ] CI / 工具链维护

## 验证清单
在提交前请确认已在本地完成以下检查：
- [ ] `cargo fmt --all -- --check` 格式化检查通过
- [ ] `cargo clippy --all-targets --locked -- -D warnings` 静态分析无告警
- [ ] `cargo test --locked` 单元测试全部通过
- [ ] `cargo build --release --locked` 构建成功
- [ ] （如涉及行为变更）已完成服务端或真人验收测试

## 关联 Issue
Closes #
