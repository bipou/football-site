## Git 提交信息规范

### 语言
中文

### 格式
<类型>: <简短描述>

## 编码规范

### 规则
1. 遵循 Leptos 规范和实践，同构模式，cargo-leptos 工具
2. view! 中用 t! 宏，避免 t_string!
3. view! 分支或栈溢：Either 是首选，拆分为独立组件是根本，标准库也好，避免 into_any
4. 样式须级联，禁并列组件重复样式
5. 遵循 SurrealDB 规范和实践
