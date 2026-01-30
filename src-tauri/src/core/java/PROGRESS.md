# Java Provider / Per-provider Cache 工作进度与后续规划

概述
- 目标：支持可插拔的 Java provider（例如 Adoptium、Corretto 等），并为每个 provider 提供独立的 catalog 缓存以避免互相污染；同时为后续把 provider 注入到下载 / 恢复流程中做准备。
- 当前状态：PR1（per-provider cache）已在本地实现并通过单元测试与 `cargo test`（全部通过）；后续 PR（PR2/PR3/PR4）将在此基础上逐步实现 `PendingJavaDownload.provider_name`、恢复逻辑以及用户偏好持久化与 UI 支持。

已完成（摘要）
- 添加 provider 注册草案（可扩展为 `ProviderRegistry`）：
  - 文件：`src-tauri/src/core/java/providers/registry.rs`（包含 `register/get/default` 等方法与单元测试）
- 使 `JavaProvider` trait 可作为 trait-object：
  - 在 `src-tauri/Cargo.toml` 中加入 `async-trait`
  - 在 `src-tauri/src/core/java/provider.rs` 使用 `#[async_trait]` 标注 trait
  - 在 `src-tauri/src/core/java/providers/adoptium.rs` 的 impl 上加 `#[async_trait]`
- 实现 per-provider catalog cache（核心改动）：
  - 新增 API（保留 legacy 接口兼容）：
    - `get_catalog_cache_path_for_provider(...)`
    - `load_cached_catalog_for_provider(...)`
    - `save_catalog_cache_for_provider(...)`
    - `clear_catalog_cache_for_provider(...)`
    - 这些函数均在 `src-tauri/src/core/java/mod.rs` 中新增或封装
  - 修改 `AdoptiumProvider::fetch_catalog`：使用 `provider_key = "{provider}-{os}-{arch}"` 作为 cache key（`adoptium-linux-x64` 之类），先尝试 `load_cached_catalog_for_provider`（未过期则直接返回），在抓取成功后 `save_catalog_cache_for_provider`。
- 测试
  - `src-tauri/src/core/java/mod.rs`：添加了保存/加载 cache 的单元测试（写入 temp dir，验证读写与清理逻辑）
  - `src-tauri/src/core/java/providers/adoptium.rs`：添加了 provider key 生成测试
  - `cargo test`：所有单元测试通过（当前测试数：85，全部通过）
- 注册 ProviderRegistry 到应用（startup）
  - 在 `src-tauri/src/main.rs` 的 `setup()` 中 `app.manage(provider_registry)` 并 `register("adoptium", …, default=true)`，保证启动时有默认 provider

为什么这样做？
- per-provider cache：可以避免不同 provider（或不同平台/arch）的 catalog 相互覆盖，避免误用缓存数据。
- 把 provider 逻辑集中在库层（`core::java`）而不是分散到 UI/命令层：利于维护与测试、也方便后续注入/切换 provider。

已知问题与注意点（需要在后续 PR 中解决）
- `PendingJavaDownload` 当前还没有记录 `provider_name`（影响恢复流程），这将在 PR2 中修改。
- 缓存迁移：我们保留了 legacy `java_catalog_cache.json`（兼容旧安装），但后续可考虑迁移策略或清理策略。
- 并发安全：`ProviderRegistry` 采用 `RwLock` 做读写同步；当前采用 `.unwrap()` 在 lock 上（若 lock 被 poison 会 panic），若要更健壮可换 `parking_lot::RwLock` 或显式处理 poison。
- `async-trait` 引入：为实现 trait-object 的 async 方法做了权衡（简化实现、性能影响极小），若团队有 policy 可替代为 `BoxFuture` 的手写方案。

后续 PR 计划（优先级 & 任务清单）

PR 2 — `feat/java-pending-download-provider`（优先级：高）
- 为 `PendingJavaDownload` 增加字段 `provider_name: Option<String>`（并更新 `DownloadQueue` 的序列化/反序列化）
- 在 `download_and_install_java_with_provider` 中写入 `provider_name`（从 `ProviderRegistry` 或传入 provider 获取）
- 更新前端 TS 类型（`ts-rs` 生成的 binding 同步）
- 添加单元/集成测试，验证保存后能正确读取 provider 信息

PR 3 — `feat/java-resume-provider`（优先级：高）
- 修改 `resume_pending_downloads`：优先使用 pending 中记录的 `provider_name` 去解析 provider 并恢复下载
- 若 provider 不存在或不可用，按策略 fallback（记录错误并通知前端）
- 添加并发与失败场景测试（例如 provider 无响应、url 失效）

PR 4 — `feat/java-provider-pref`（优先级：中）
- 在 `src-tauri/src/core/java/persistence.rs::JavaConfig` 添加：`preferred_provider: Option<String>`
- 提供 get/set API：`set_preferred_java_provider` / `get_preferred_java_provider`
- 增加 Tauri commands：`list_java_providers`, `set_default_java_provider`, `get_default_java_provider`
- 在 app 启动时读取 persisted preference 并设置 `ProviderRegistry` 的 default
- 前端（Settings）加入 provider 选择 UI（可选：我也可以做前端改动）

验收标准（PR1 -> PR4）
- PR1：实现 per-provider cache，相关单元测试通过，编译通过，主流程无回归（已满足）
- PR2：`PendingJavaDownload` 序列化向后兼容，恢复下载能以正确 provider 恢复，相关测试通过
- PR3：`resume_pending_downloads` 能正确使用 `provider_name`，并在 provider 不可用时有明确降级策略
- PR4：用户可以在 UI 中选择 provider，选择会持久化并在下次启动时生效

回滚与兼容策略
- 所有对储存结构（如 `PendingJavaDownload`）的更改将尽量保持向后兼容（可以保留 `Option` 字段，并在读取旧数据时填充 `None`）
- 对 catalog cache 的改造保留了 legacy cache 读取路径（在找不到 provider-specific cache 时仍可读取老文件，以避免用户缓存立即失效）

我下一步要做的工作
- 根据你的确认：我会为 PR1 创建分支（建议分支名：`feat/java-per-provider-cache`），提交改动并打开 PR。PR 内容将包含：
  - `mod.rs` 的 cache API 与测试
  - `adoptium.rs` 的缓存 key 调整与测试
  - `Cargo.toml` 的 `async-trait`（如已加则列入 PR）
  - `main.rs` 在 setup 注册 `ProviderRegistry`（如已加则列入 PR）
  - PR 描述与 Checklist（包含 `cargo test` 和简短变更说明）
- 在 PR1 合并后我会按计划继续 PR2/PR3/PR4 的实现（每步单独 PR，便于 review）

需要你确认 / 给我指示的地方
- 分支命名与 PR 标题是否接受（建议：`feat/java-per-provider-cache`，PR 标题 `feat(java): per-provider catalog cache`）？
- PR 描述是否需要包含特定格式或额外的上下文（例如引用 issue、指定审阅者等）？
- 是否需要我顺便在 PR 中包含对前端的 placeholder（例如一个 `provider` 下拉在设置里但先不联动）？

---

如果你确认分支与 PR 标题无异议，我会把 PR1 做成一个干净、可下线回滚的提交并开启 PR。需要我现在就去创建 PR 吗？
