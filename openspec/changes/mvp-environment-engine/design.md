## Context

C-Box MVP 是一个基于 Tauri 2 + Vue 3 + Rust 的桌面应用，核心功能是环境部署引擎。项目从零开始，无历史代码包袱。PRD（`docs/PRD.md`）已经过 Context7 文档验证和 Codex 安全审查，技术选型和架构设计已确认可行。

当前状态：纯绿地项目，仅有 PRD 文档。MVP 阶段聚焦 Windows 平台，实现 Pack 声明式环境检测与一键安装。

关键约束：
- Tauri 2 Shell 插件采用命令白名单机制，需通过统一脚本执行器绕过
- `@tauri-apps/plugin-store` 用于状态持久化，需显式 flush 策略保障崩溃一致性
- Pack 间存在 DAG 依赖关系，安装顺序必须经过拓扑排序

## Goals / Non-Goals

**Goals:**
- 搭建可运行的 Tauri 2 + Vue 3 桌面应用骨架
- 实现完整的 Pack 生命周期：加载 → 检测 → 安装 → 状态持久化
- 提供 6 个内置 Pack（Node.js、Python、Git、uv、Claude Code、npm）
- 提供基础 UI：Pack 列表、状态灯、进度条、一键操作
- 处理 Windows 特有场景（UAC、PATH 刷新、winget 集成）

**Non-Goals:**
- AI 技能控制面板（v0.2）
- 凭证管理中心（v0.3）
- .cbox 导出/导入（v0.4）
- 社区 Pack 市场（v0.5）
- Linux / macOS 平台支持（v1.0）
- Pack 卸载功能（仅预留接口，不实现逻辑）
- 离线安装包缓存

## Decisions

### D1: 项目结构

```
C-Box/
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── orchestrator/       # Orchestrator 编排器
│   │   ├── pack/               # Pack 加载、状态机、安装
│   │   ├── detector/           # 环境检测引擎
│   │   └── platform/           # PAL 平台抽象层
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                        # Vue 3 前端
│   ├── App.vue
│   ├── components/
│   ├── composables/
│   ├── stores/                 # Pinia 状态管理
│   └── types/
├── packs/                      # 内置 Pack 定义
│   ├── nodejs/manifest.toml
│   ├── python/manifest.toml
│   ├── git/manifest.toml
│   ├── uv/manifest.toml
│   ├── claude-code/manifest.toml
│   └── npm/manifest.toml
├── docs/PRD.md
└── openspec/
```

**理由**：遵循 Tauri 2 标准项目结构。Rust 后端按职责划分为 orchestrator/pack/detector/platform 四个模块，前端使用 Pinia 管理状态。内置 Pack 定义放在项目根目录 `packs/` 下，与源代码分离。

### D2: 双通道通信架构

- **Orchestrator（命令通道）**：Rust 侧实现，处理所有状态变更操作。前端通过 `invoke()` 调用 Tauri Command，Command 内部委托 Orchestrator 处理。类型安全，请求/响应明确。
- **Tauri Event（事件通道）**：仅用于后端向前端广播非请求性通知（安装进度、状态变更）。前端通过 `listen()` 订阅。

**替代方案**：仅用 Tauri Event 做所有通信 → 放弃，因为 Event 非类型安全，不适合承担状态变更编排。

### D3: Pack 检测策略

检测逻辑在 Rust 侧通过 `@tauri-apps/plugin-shell` 执行系统命令（如 `node --version`），解析 stdout 提取版本号。

- 优先使用 `detect.command`，失败时回退到 `detect.fallback_command`
- **编码处理**：所有命令前置 `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; `，解决中文 Windows（GBK 编码）下乱码问题；Rust 侧以 `String::from_utf8_lossy` 解码，规范化 CRLF→LF 后再做 trim + 正则匹配
- 命令超时设为 10 秒，超时视为检测失败

**替代方案**：通过文件系统探测（检查安装路径是否存在）→ 放弃，因为不同用户安装路径不同，不可靠。

### D4: Pack 安装策略

MVP 支持的安装方式：

| 方式 | 实现 | Windows 场景 |
|------|------|-------------|
| `winget` | 调用 `winget install --id <package> --accept-source-agreements --accept-package-agreements` | 主要方式 |
| `scoop` | 调用 `scoop install <package>` | 备选 |
| `script` | 执行 Pack 目录下的 `.ps1` 脚本 | 自定义安装 |
| `url` | 下载安装包后执行（预留，MVP 不实现） | — |

所有命令通过统一脚本执行器（`powershell.exe -NoProfile -NonInteractive -Command`）执行，Rust 侧构造命令字符串，仅允许 manifest 中声明的命令模板。

**Tauri 2 Capabilities 约束**：`tauri-plugin-shell` 实行严格沙箱白名单，必须在 `src-tauri/capabilities/default.json` 的 `shell:allow-execute.allow` 数组中显式声明 `powershell`（cmd: `powershell.exe`）和 `pwsh` 两个条目，`-Command` 参数位置使用 `{ "validator": ".+" }`。Rust 代码中通过声明的 `name` 字段引用命令（`Command::new("powershell")`），使用可执行文件名将被沙箱静默拦截。

### D5: 状态持久化与崩溃恢复

使用 `@tauri-apps/plugin-store` 存储 Pack 状态，配置 `autoSave: 500`（500ms 防抖）。

关键状态变更节点（进入下载中、安装中、安装完成、安装失败）强制调用 `store.save()` 立即 flush，不依赖 autoSave。

崩溃恢复：启动时读取持久化状态，若发现处于「下载中」或「安装中」的 Pack，自动回退到「未安装」状态并标记需重试。

### D6: 前端技术方案

- Vue 3 Composition API + TypeScript
- Pinia 管理前端状态（与 Rust 侧 plugin-store 分工：Pinia 管理 UI 状态，plugin-store 管理持久化状态）
- 组件库：不使用第三方 UI 库，自定义轻量组件保持包体积 < 20MB
- i18n：使用 vue-i18n，首版仅中文，预留多语言架构

## Risks / Trade-offs

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| winget 在部分 Windows 版本不可用 | 无法安装 Pack | 检测时优先确认 winget 可用性，不可用时提示用户安装或切换到 scoop |
| Shell 命令执行安全漏洞 | 任意代码执行 | 仅执行 manifest 声明的命令模板 + 锁定插件版本 + 日志审计 |
| plugin-store 崩溃一致性 | 状态丢失 | 关键节点强制 flush + 启动时状态回退 |
| 首批 Pack 版本检测正则可能不兼容 | 检测失败 | fallback_command + 宽松正则 + 用户可手动跳过 |
| Tauri 2 在 Windows 上的 WebView2 依赖 | 部分旧系统无 WebView2 | 打包时内嵌 WebView2 bootstrapper |
