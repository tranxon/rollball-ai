# Embedding 模块代码审查报告

**审查日期**: 2026-06-06
**审查范围**: 本地未提交的所有更改（41 个文件，+1,948/-244 行）
**主要模块**: `acowork-embed`（新增）、Gateway Embedding API、Runtime Embedding Provider、Grafeo 记忆 consolidation、前端 Embedding 设置 UI
**审查视角**: Completeness + Correctness

---

## 验证结果总览

| # | 级别 | Finding | 状态 | 说明 |
|---|------|---------|------|------|
| 1 | Critical | Gateway 未启动 embed 进程 | ✅ 确认 | grep 确认 `spawn_embed_process` 仅定义，无调用点 |
| 2 | Critical | gRPC 协议桥丢弃 embed 字段 | ✅ 确认 | proto 缺字段，bridge 显式 `_` 丢弃，Runtime 硬编码 None |
| 3 | Critical | 热更新 embed 配置是 stub | ✅ 确认 | L946-949 明确 TODO，仅日志无实际重建 |
| 4 | Critical | ONNX 推理阻塞 async 线程 | ✅ 确认 | L175 `tokio::sync::Mutex` + 直接 `session.run()` |
| 5 | Critical | 每 embedding 新建线程+Runtime | ✅ 确认 | L147-164 `std::thread::spawn` + `new_current_thread()` |
| 6 | Critical | UTF-8 字节切片 panic | ✅ 确认 | L332-333 `&summary_value[..200]` 按字节切 |
| 7 | Warning | agents.rs 多处 embed_config_json=None | ✅ 确认 | L1018/1415/1601 三处硬编码 None |
| 8 | Warning | 自动下载后立即 drop 模型 | ✅ 确认 | L109-115 `drop(model)` |
| 9 | Warning | Gateway 关闭未清理 embed 进程 | ✅ 确认 | `kill_embed_process` 仅定义，shutdown 路径未调用 |
| 10 | Warning | 读锁跨越 await 阻塞写者 | ⚠️ 部分确认 | 持锁期间仅做轻量 HTTP 状态查询，实际阻塞时间很短 |
| 11 | Warning | 下载全量载入内存 | ✅ 确认 | L282 `response.bytes().await?` |
| 12 | Warning | 取消标志永为 false | ✅ 确认 | L532 `AtomicBool::new(false)` 局部变量 |
| 13 | Suggestion | 默认 embedding_models.json 缺失 | ✅ 确认 | 首次部署返回空列表 |
| 14 | Suggestion | proto 缺 embed 字段 | 🔄 与 #2 合并 | 同一问题的协议层面表述 |
| 15 | Suggestion | is_downloaded 语义不一致 | ✅ 确认 | registry 仅查目录，downloader 查具体文件 |
| 16 | Suggestion | Unix 信号重复注册 | ✅ 确认 | `flag::register` 的 AtomicBool 从未被读取 |
| 17 | Suggestion | 全零 mask 兜底行为 | ✅ 确认 | L71-78 `unwrap_or(0)` 返回 hidden_state[0] |

---

## Critical Issues (MUST FIX) — 需要修复

### 1. Gateway 从未启动 acowork-embed 子进程 ✅确认
**位置**: `core/acowork-gateway/src/lifecycle/embed.rs#L35`

**问题**: `spawn_embed_process` 已实现，但 Gateway 的 `run()` 启动流程和 HTTP API 中均无调用点。`GatewayState.embed_process` 永远为 `None`，本地 ONNX Embedding 服务实际上不可用。

**修复**: 在 Gateway `run()` 初始化阶段调用 `spawn_embed_process()`，或在首个 Embedding API 请求时惰性启动。

---

### 2. gRPC 协议桥丢弃 AgentHelloResult 的 embed 字段 ✅确认
**位置**:
- `core/acowork-core/src/proto_bridge.rs#L525-L526`
- `core/acowork-runtime/src/grpc/client.rs#L512-L519`
- `core/acowork-core/proto/gateway_ipc.proto`

**问题**:
- `proto_bridge.rs` L525-526 显式 `let _ = (..., embed_endpoint, embed_model_id, embed_dimension)` 丢弃
- `gateway_ipc.proto` 的 `AgentHelloResult` message 缺少这三个字段
- Runtime gRPC 客户端 L517-519 硬编码 `embed_endpoint: None`，注释明确说 TODO

这导致通过 gRPC 连接的 Runtime 永远收不到本地 embed 服务端点，ONNX 一级 Provider 无法加入 Fallback 链。

**修复**:
1. 在 `gateway_ipc.proto` 的 `AgentHelloResult` 中新增 `string embed_endpoint`、`string embed_model_id`、`uint64 embed_dimension`
2. 在 `proto_bridge.rs` 的 `to_proto` / `from_proto` 中正确传递
3. 在 Runtime `grpc/client.rs` 中从 proto result 读取并填充到 `AgentHelloConfig`

---

### 3. Runtime 热更新 embedding 配置是未实现的 stub ✅确认
**位置**: `core/acowork-runtime/src/agent/session/session_manager.rs#L933-L949`

**问题**: `handle_embedding_config_update` 仅打印日志（L946-949 明确标注 TODO）。Gateway 推送新模型/维度变更时，运行中的 Agent 不会重建 `FallbackEmbeddingProvider` 链，也不会更新 Grafeo memory store 的维度。

**修复**: 在 `SessionManager` 中持有可替换的 Provider 句柄，收到更新后重建 Provider 链；维度变更时拒绝不兼容变更或触发迁移。

---

### 4. ONNX 阻塞推理卡住 tokio 工作线程 ✅确认
**位置**: `core/acowork-embed/src/model.rs#L174-L229`

**问题**: `embed_batch` L175 通过 `tokio::sync::Mutex` 获取 `Session` 后直接执行 `session.run()`。ONNX 推理是 CPU 密集型阻塞操作，会长时间占用 tokio worker 线程，高并发下 HTTP 服务器完全无响应。

**修复**: 将 `tokio::sync::Mutex` 替换为 `std::sync::Mutex`，把 `session.run()` 及张量构造包进 `tokio::task::spawn_blocking`。

```rust
// 建议修改示意
let session = self.session.clone(); // Arc<std::sync::Mutex<Session>>
let result = tokio::task::spawn_blocking(move || {
    let mut session = session.lock().unwrap();
    // ... 构造 tensor、执行 session.run、提取数据 ...
}).await.map_err(|e| ModelError::Inference(...))?;
```

---

### 5. 每次 embedding 都新建 OS 线程 + tokio Runtime ✅确认
**位置**: `core/acowork-runtime/src/memory/consolidation_bg.rs#L144-L165`

**问题**: `embedding_fn` L147-164 每调用一次（每个文本）都 `std::thread::spawn` + `tokio::runtime::Builder::new_current_thread().build()`。100 次 embedding = 100 个线程 + 100 个 Runtime，极易耗尽资源。

**修复**: 复用已有 Runtime 的 `Handle` 执行异步 embedding，或将 consolidation 中的同步闭包改为异步接口。

```rust
// 建议：避免新建线程/Runtime
let handle = tokio::runtime::Handle::current();
handle.block_on(provider.embed(text)) // 仅在非异步上下文使用
// 更优方案：将 consolidation 中的同步闭包改为异步接口，彻底避免 block_on
```

---

### 6. UTF-8 字节切片越界 panic ✅确认
**位置**: `core/acowork-grafeo/src/consolidation/offline.rs#L332-L333`

**问题**: `&summary_value[..200]` 按字节切片。中文等多字节 UTF-8 字符可能被切在字符中间，导致运行时 panic。

**修复**: 按字符截断。

```rust
let truncated = if summary_value.chars().count() > 200 {
    format!("{}…", summary_value.chars().take(200).collect::<String>())
} else {
    summary_value
};
```

---

## Warnings (SHOULD FIX) — 建议修复

### 7. Gateway 启动 Agent 时 RuntimeConfigUpdate 永远不带 embed_config_json ✅确认
**位置**: `core/acowork-gateway/src/http/agents.rs` L1018/L1415/L1601

**问题**: 三处推送 `RuntimeConfigUpdate` 的代码均硬编码 `embed_config_json: None`。Agent 首次启动时收不到 embedding 配置，只能依赖环境变量或回退到 Ollama/Remote。而 `global_push.rs` L409-L433 在模型切换时已正确构造 `embed_config_json`。

**修复**: 构造 `RuntimeConfigUpdate` 时，若 `embed_process` 已存在且包含活跃模型信息，将配置序列化填入，与 `global_push.rs` 保持一致。

---

### 8. acowork-embed 首次启动自动下载后未保留模型 ✅确认
**位置**: `core/acowork-embed/src/main.rs#L109-L115`

**问题**: 自动下载成功后 L109-115 调用 `try_load_model`，但紧接着 `drop(model)` 且未赋给 `initial_model`。`AppState.model` 最终为 `None`，服务启动后仍需手动调用 `POST /models/{id}/load`。

**修复**: 将 `try_load_model` 结果赋给 `initial_model`，使首次启动后即可用。

---

### 9. Gateway 关闭时未清理 acowork-embed 子进程 ✅确认
**位置**: `core/acowork-gateway/src/lifecycle/embed.rs#L132`

**问题**: `kill_embed_process` 已实现，但 Gateway shutdown 路径从未调用。Gateway 退出后 `acowork-embed` 会成为孤儿进程。

**修复**: 在 Gateway `run()` 的 shutdown 尾部读取 `gw.embed_process` 并调用 `kill_embed_process`。

---

### 10. 异步读锁跨越 await 点阻塞写者 ⚠️部分确认
**位置**: `core/acowork-gateway/src/http/embedding_api.rs#L98-L150`

**问题**: `list_embedding_models` 在持有 `tokio::sync::RwLockReadGuard` 的状态下 await HTTP 请求（查询 embed 服务模型状态）。读锁长时间不释放，写者被阻塞。

**实际风险**: 持锁期间的 HTTP 请求是轻量级状态查询（timeout 2-5s），且模型列表通常很短（3-5 个），实际阻塞时间较短。但在 embed 服务不可达时，每个模型的状态查询都会 timeout，累积阻塞时间可能较长。

**修复**: 在发起外部 HTTP 请求前，先将需要的数据从锁内克隆出来并 `drop(gw)`，与 `download_model` / `select_model` 的处理方式保持一致。

---

### 11. 模型下载全量载入内存 ✅确认
**位置**: `core/acowork-embed/src/download.rs#L282`

**问题**: `download_file_inner` L282 使用 `response.bytes().await?` 将整个文件（可能是数百 MB 的 ONNX 模型）一次性读入内存再写入磁盘，易造成 OOM。

**修复**: 改用 `response.bytes_stream()` 流式写入，或 `tokio::io::copy` 从 response 流直接写入文件。

---

### 12. 下载接口的取消标志永为 false ✅确认
**位置**: `core/acowork-embed/src/server.rs#L532`

**问题**: `download_model` handler L532 每次新建局部 `AtomicBool::new(false)` 传给下载器。没有任何代码能将其设为 `true`，取消逻辑形同虚设。

**修复**: 若暂不支持取消，移除该参数；否则从 `AppState` 提供可被外部设置的共享取消标志。

---

## Suggestions (CONSIDER) — 可改进

### 13. Gateway data_dir 缺少默认 embedding_models.json ✅确认
**位置**: `core/acowork-gateway/src/resource_cache.rs`

若本地不存在 `embedding_models.json`，初始化为空列表。但 `acowork-embed/assets/embedding_models.json` 已包含内置注册表，可考虑首次部署时自动回退加载。

---

### 14. 与 #2 合并 — gRPC proto embed 字段缺失是同一问题的协议层面表述

---

### 15. `ModelRegistry::is_downloaded` 与 `Downloader::is_downloaded` 语义不一致 ✅确认
**位置**: `core/acowork-embed/src/registry.rs#L156-L159`

前者 L156-158 仅检查目录是否存在，后者检查 `model.onnx` 和 `tokenizer.json` 两个具体文件。建议统一判断逻辑，避免空目录被误判为已下载。

---

### 16. `shutdown.rs` 中 Unix 信号处理重复注册 ✅确认
**位置**: `core/acowork-embed/src/shutdown.rs#L47-L70`

L47 `flag::register(SIGTERM, AtomicBool::new(false))` 和 L61 `flag::register(SIGINT, AtomicBool::new(false))` 创建的 `AtomicBool` 从未被读取（实际信号处理由 L51-57/63-69 的 `Signals::new` 完成），逻辑冗余。

**修复**: 移除 `flag::register` 调用，只保留 `signal_hook::iterator::Signals` 方式。

---

### 17. `last_token_pooling` 对全零 mask 的兜底行为可文档化 ✅确认
**位置**: `core/acowork-embed/src/pool.rs#L71-L80`

L78 `unwrap_or(0)` 当 `attention_mask` 全为 0 时回退到返回 `hidden_state[0]`，建议在该函数文档中说明此兜底策略。

---

## 需要修复的问题清单（按优先级）

### 必须修复 (6)
1. **Gateway 启动 embed 进程** — 否则整个本地 embedding 功能不可用
2. **gRPC proto embed 字段** — 否则 Runtime 永远收不到 embed 端点
3. **热更新 embed 配置 stub** — 否则模型切换后旧链路继续使用
4. **ONNX 推理 spawn_blocking** — 否则并发下 HTTP 服务无响应
5. **consolidation_bg 线程/Runtime 池化** — 否则资源耗尽
6. **UTF-8 字节切片** — 否则中文内容 panic

### 建议修复 (6)
7. agents.rs embed_config_json 穿透
8. 自动下载后保留模型
9. Gateway shutdown 清理 embed 进程
10. embedding_api 读锁优化
11. 流式下载替代全量载入
12. 取消标志或移除

### 可改进 (4, #14 已合并)
13. 默认 embedding_models.json
15. is_downloaded 语义统一
16. 信号处理冗余清理
17. 全零 mask 兜底文档化

---

## Summary of Changes

- **新增 `acowork-embed` crate**: 完整的 ONNX 本地 Embedding 服务（HTTP API、模型下载、ONNX 推理、配置管理）
- **Gateway 集成**: Embedding HTTP API 路由、模型生命周期管理（但未实际启动进程）
- **Runtime 集成**: Embedding Provider 接口、Fallback 链、背景记忆 consolidation（但热更新为 stub）
- **Grafeo 集成**: 离线 consolidation 大幅扩展，支持 embedding 向量存储
- **前端集成**: Embedding 模型设置页、多语言 i18n、Gateway API 类型扩展
- **协议扩展**: `gateway_ipc.proto` 新增 `RuntimeConfigUpdate.embed_config_json`（但 `AgentHelloResult` 缺少对应字段）