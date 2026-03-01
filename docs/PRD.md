# C-Box 产品需求文档 (PRD)

> **版本**: v0.1.0
> **日期**: 2026-03-01
> **状态**: 草案

---

## 1. 产品定位

C-Box 是一个面向 AI 时代的**开发环境便携管理工具**。

核心价值：将用户的整套 AI 开发环境（底层运行时、AI 工具技能、MCP 服务、工作流配置、API 凭证）统一收归到一个可视化桌面应用中，实现**一次配置，到处运行**。

## 2. 目标用户

| 用户类型 | 使用场景 |
|----------|----------|
| 个人开发者 | 换新电脑时快速恢复整套 AI 开发环境 |
| 团队 | 统一团队开发环境配置，新成员一键入职 |
| 非技术用户 | 通过引导式界面安装 AI 工具，零门槛上手 |

## 3. 核心问题

- AI 工具链碎片化：Claude Code、Codex、Gemini CLI、MCP 服务等各自独立配置
- 环境迁移成本高：每换一台设备，需重复安装运行时、配置技能、填入密钥
- 配置分散且不可视：技能文件、MCP 配置、API Key 散落在不同路径的不同文件中

## 4. 产品架构

### 术语约定

| 术语 | 含义 |
|------|------|
| **Tauri Plugin** | Tauri 框架级代码插件（如 shell、stronghold、store），Rust 实现 |
| **C-Box Pack** | C-Box 业务级工具包，TOML 声明式定义，由核心引擎解析执行 |
| **Adapter** | AI 工具适配器，实现统一 Trait 接口接入各 AI 工具 |
| **Module** | C-Box 顶层功能模块，通过事件总线通信，互不依赖 |

```
┌──────────────────────────────────────────────────────┐
│                    C-Box Desktop                      │
│                  (Tauri 2 + Vue 3)                    │
├──────────┬──────────┬──────────┬──────────────────────┤
│  环境部署  │ 技能面板  │ 凭证管理  │   未来模块预留位...    │
│   引擎    │          │   中心   │                      │
│  (Pack)  │(Adapter) │(Secure) │     (Module)         │
├──────────┴──────────┴──────────┴──────────────────────┤
│                    核心引擎 (Rust)                      │
│  Pack 加载器 | 状态机 | Orchestrator | 配置管理器 | 日志  │
├──────────────────────────────────────────────────────┤
│              Tauri Plugin 层 (框架能力)                 │
│   shell (命令执行) | stronghold (加密) | store (持久化)  │
│   updater (自动更新) | log (日志) | fs (文件系统)        │
├──────────────────────────────────────────────────────┤
│                   平台抽象层 (PAL)                      │
│         Windows / Linux / macOS 差异封装                │
└──────────────────────────────────────────────────────┘
```

### 4.1 架构原则

- **模块级低耦合**：各大功能模块（环境部署、技能面板、凭证管理）互为独立
- **适配器级低耦合**：每个 AI 工具（Claude Code、Codex、Gemini）实现统一 Trait 接口即可接入
- **Pack 级低耦合**：每个环境依赖（Node.js、Python）为独立 Pack 目录，TOML 声明式加载
- **显式扩展点**：通过 Trait 注册表 + manifest schema 版本控制提供扩展能力，新增 Pack/Adapter 不修改核心代码；新增 Module 可能需要在 Orchestrator 中注册路由

### 4.2 模块间通信策略

模块间通信采用**双通道分离**设计：

| 通道 | 机制 | 用途 | 示例 |
|------|------|------|------|
| **命令通道** | Orchestrator（类型化 RPC） | 状态变更操作 | 技能面板请求安装某工具 → `InstallRequest` |
| **事件通道** | Tauri Event | UI 通知 / 进度广播 | 安装进度 → 前端渲染进度条 |

> 原则：**状态变更走 Orchestrator，UI 通知走 Event**。避免用非类型安全的事件承担核心编排职责。

```rust
// Orchestrator 类型化请求/响应示例
enum OrchestratorRequest {
    InstallPack { pack_id: String },
    DetectAll,
    EnableSkill { adapter: String, skill_id: String },
}

enum OrchestratorResponse {
    Progress { pack_id: String, state: PackState, percent: f32 },
    Complete { pack_id: String, result: Result<(), InstallError> },
    DetectionReport(Vec<PackStatus>),
}
```

### 4.3 Shell 命令执行安全策略

Tauri 2 的 Shell 插件采用严格的命令白名单机制，但 C-Box 需要执行多种安装命令（winget、scoop、brew、自定义脚本），无法穷举所有命令。

**解决方案**：在 Tauri `capabilities` 配置中，仅授权统一的**脚本执行器**（Windows: PowerShell / Linux+macOS: bash）。由 C-Box 核心引擎构造脚本内容并通过执行器运行，而非直接暴露任意命令。

```json
{
  "permissions": [
    "shell:allow-execute",
    {
      "identifier": "shell:allow-spawn",
      "allow": [
        { "name": "powershell", "cmd": "powershell", "args": ["-NoProfile", "-Command"] },
        { "name": "bash", "cmd": "bash", "args": ["-c"] }
      ]
    }
  ]
}
```

**安全加固**：
- 核心引擎仅执行来自**已注册 Pack manifest** 中声明的命令模板，拒绝前端直接传入的任意脚本字符串
- 锁定 Tauri 及 Shell 插件版本，持续监控安全公告（[参考 GHSA-93mr-g7q3-6r3q](https://github.com/tauri-apps/tauri/security/advisories/GHSA-93mr-g7q3-6r3q)）
- 所有实际执行的命令记入日志，便于安全审计

## 5. 功能模块详细设计

### 5.1 模块一：环境部署引擎 (MVP)

**职责**：检测当前系统缺失的开发环境依赖，提供一键自动安装。

#### Pack 声明规范 (manifest.toml)

```toml
[pack]
id = "nodejs"
name = "Node.js"
description = "JavaScript runtime environment"
category = "runtime"                    # runtime | ai-tool | mcp | package-manager
version_requirement = ">=18.0.0"
platforms = ["windows", "linux", "macos"]
dependencies = []                       # 依赖的其他 Pack id
schema_version = "1.0"                  # manifest 规范版本，用于前向兼容

[detect]
command = "node --version"
version_regex = 'v(\d+\.\d+\.\d+)'
fallback_command = "node -v"            # 备用检测命令（应对 locale 差异）

[install.windows]
method = "winget"                       # winget | scoop | url | script
package = "OpenJS.NodeJS.LTS"
checksum = ""                           # 安装包 SHA-256（url/script 方式时必填）
requires_admin = false                  # 是否需要管理员权限
requires_reboot = false                 # 安装后是否需要重启

[install.linux]
method = "script"
script = "install.sh"
checksum = ""

[install.macos]
method = "brew"
package = "node@lts"

[uninstall]                             # 卸载/回滚钩子
command = ""                            # 可选，卸载命令
```

#### 环境锁文件 (environment.lock.toml)

与 `manifest.toml`（期望态）配合，`environment.lock.toml` 记录**实际解析后的精确状态**：

```toml
[[resolved]]
pack_id = "nodejs"
installed_version = "22.14.0"
installed_at = "2026-03-01T10:30:00Z"
install_method = "winget"
checksum = "sha256:abc123..."
```

> 此文件在检测/安装完成后自动生成，用于导出 .cbox 包时精确还原环境。

#### Pack 生命周期状态机

```
未检测 → 检测中 → 未安装 → 下载中 → 安装中 → 已安装 → 已配置
                     ↓         ↓         ↓
                  检测失败   下载失败   安装失败
                     ↓         ↓         ↓
                  (可重试)   (可重试)   (可重试)
```

- 状态通过 `@tauri-apps/plugin-store` 持久化到本地，支持断点恢复
- 前端通过 Tauri Event 监听状态变化，渲染进度条和状态指示灯

#### 依赖拓扑排序

Pack 之间存在 `dependencies` 依赖关系（如 npm 依赖 Node.js），安装时核心引擎需执行 DAG 拓扑排序，确保被依赖项优先安装。若检测到循环依赖，拒绝执行并报错。

#### MVP v1 首批支持工具

| 工具 | 类别 | 检测方式 |
|------|------|----------|
| Node.js (LTS) | runtime | `node --version` |
| Python 3.x | runtime | `python --version` |
| Git | tool | `git --version` |
| uv | package-manager | `uv --version` |
| Claude Code | ai-tool | `claude --version` |
| npm | package-manager | `npm --version` |

### 5.2 模块二：AI 技能控制面板 (v2)

**职责**：可视化管理各 AI 工具的技能、MCP 服务、工作流配置，支持即时启用/停用。

#### AiToolAdapter Trait 标准接口

```rust
trait AiToolAdapter {
    fn detect(&self) -> DetectionResult;          // 检测 AI 工具是否已安装
    fn list_skills(&self) -> Vec<SkillInfo>;      // 列出可用技能
    fn enable_skill(&self, id: &str) -> Result;   // 启用技能
    fn disable_skill(&self, id: &str) -> Result;  // 停用技能
    fn sync_config(&self) -> Result;              // 同步配置到本地文件
}
```

#### 模块间联动

当 `detect()` 发现 AI 工具未安装时，通过 Orchestrator 发送 `InstallRequest` 给环境部署引擎触发安装，安装进度通过 Tauri Event 广播至前端。模块间无直接调用关系。

#### 首批适配器

| AI 工具 | 配置文件路径 | 管理内容 |
|---------|------------|----------|
| Claude Code | `~/.claude/` 相关文件 | 技能、MCP 服务、工作流 |
| Codex | 待定 | 待定 |
| Gemini CLI | 待定 | 待定 |

### 5.3 模块三：AI 凭证管理中心 (v2)

**职责**：安全存储和管理各 AI 工具的 API Key、订阅信息等敏感凭证。

#### 安全方案

- **本地存储**：统一使用 `tauri-plugin-stronghold`（基于 IOTA Stronghold 加密引擎）
  - 凭证存储于加密的 vault.hold 文件中，永不明文落盘
  - 跨平台一致性：Windows / Linux / macOS 行为完全相同，无需适配各平台原生密钥库
- **导出加密**：vault 文件天然加密，导出时再以用户自设主密码二次保护
- **导入流程**：新机器导入 .cbox 包 → 输入主密码解密 → 凭证写入新的 vault.hold

#### 凭证分组

| 分组 | 管理字段 |
|------|----------|
| Claude Code | API Key、订阅状态、Base URL |
| Codex | API Key、配置项 |
| Gemini | API Key、配置项 |
| 自定义 | 用户自定义的 Key-Value 凭证对 |

### 5.4 配置导出/导入

#### .cbox 包格式

```
my-config.cbox
├── manifest.json          # 包元信息：版本、创建时间、平台
├── environment.toml       # 环境依赖清单
├── skills/                # 技能配置快照
├── credentials.enc        # 主密码加密的凭证数据
└── cache/ (可选)           # 离线安装包缓存
```

- 导出时用户可选择是否包含离线安装包
- 凭证部分始终加密，导入时需输入主密码
- .cbox 包内含 `schema_version` 字段，导入时校验版本兼容性

#### 导入冲突策略

| 场景 | 策略 |
|------|------|
| 目标机器已有相同工具但版本不同 | 提示用户选择：保留现有 / 覆盖安装 / 跳过 |
| 目标机器已有凭证配置 | 提示用户选择：合并（不覆盖已有 Key） / 全量覆盖 |
| .cbox 包版本高于当前 C-Box 版本 | 拒绝导入，提示用户升级 C-Box |

## 6. 边缘场景与故障处理

| 场景 | 处理策略 |
|------|----------|
| **Windows UAC 权限提升** | `requires_admin = true` 的 Pack 触发 UAC 弹窗，用户拒绝则标记为安装失败（可重试） |
| **安装后需要重启** | `requires_reboot = true` 的 Pack 安装完成后提示用户重启，状态标记为「已安装-待重启」 |
| **PATH 环境变量刷新** | 安装完成后主动刷新 PATH（Windows: 广播 `WM_SETTINGCHANGE` / Unix: 重新加载 shell profile），而非要求用户重启终端 |
| **批量安装部分失败** | 失败项标记错误状态，已成功项保留，不做全量回滚；用户可逐个重试失败项 |
| **多版本冲突** | 检测时识别 `python` vs `python3`、架构不匹配等情况，UI 上给出明确警告和处理建议 |
| **检测命令 locale 差异** | manifest 支持 `fallback_command` 备用检测，核心引擎对输出做 trim + 正则匹配，忽略 locale 前缀 |
| **Vault 文件损坏** | 启动时校验 vault 完整性，损坏时提示用户从 .cbox 备份恢复或重新配置凭证 |
| **网络不可用** | 检测阶段不受影响（本地命令），安装阶段明确提示离线状态，引导用户使用离线 .cbox 包 |

## 7. Pack 生态

| 来源 | 说明 |
|------|------|
| 内置 Pack | 官方维护的核心工具包，随 C-Box 分发 |
| 用户自定义 | 用户按 manifest.toml 规范自行编写 Pack |
| 社区市场 | 社区贡献和分享 Pack（远期规划） |

### Pack 信任模型（社区市场阶段）

| 层级 | 机制 |
|------|------|
| **内置 Pack** | 随应用签名分发，天然可信 |
| **社区 Pack** | 必须包含 manifest 签名 + 发布者身份标识，C-Box 验证签名后方可加载 |
| **用户自定义 Pack** | 本地加载，跳过签名验证，UI 上标记为「未验证」 |

## 8. 技术选型

| 层级 | 技术 | 理由 |
|------|------|------|
| 桌面框架 | Tauri 2.x | 轻量、跨平台、Rust 后端适合系统级操作 |
| 前端 | Vue 3 + TypeScript | 轻量、生态成熟 |
| 后端 | Rust | 系统级操作、安全性、性能 |
| 安全存储 | tauri-plugin-stronghold | IOTA Stronghold 加密引擎，跨平台一致 |
| 状态持久化 | @tauri-apps/plugin-store | 持久化 KV 存储，支持 auto-save 和 change listeners |
| 命令执行 | @tauri-apps/plugin-shell | 系统命令执行 + 实时流式输出 |
| 自动更新 | @tauri-apps/plugin-updater | C-Box 自身的版本更新 |
| 日志 | @tauri-apps/plugin-log | 安装过程日志记录，便于排错 |
| Pack 格式 | TOML 声明式 | 可读性强、易于编写 |

## 9. 平台支持

| 平台 | 优先级 | 状态 |
|------|--------|------|
| Windows | P0 | MVP 首发 |
| Linux | P1 | 后续版本 |
| macOS | P1 | 后续版本 |

## 10. 分发方式

- **安装版**：提供 .msi / .exe 安装器
- **便携版**：解压即用，无需安装

## 11. 开源策略

- **协议**：MIT（待最终确认）
- **策略**：完全开源，社区自由贡献

## 12. 版本路线图

| 版本 | 里程碑 | 核心内容 |
|------|--------|----------|
| v0.1 | MVP | 环境部署引擎 + 首批 AI 开发工具 Pack + 基础 UI |
| v0.2 | 技能面板 | AI 技能控制面板（Claude Code 适配器优先） |
| v0.3 | 凭证管理 | AI 凭证管理中心 + 安全存储 |
| v0.4 | 便携配置 | .cbox 导出/导入 + 离线包支持 |
| v0.5 | Pack 市场 | 社区 Pack 市场基础设施 |
| v1.0 | 正式版 | 多平台支持 + 功能完善 + 稳定性保障 |

## 13. 非功能需求

| 维度 | 要求 |
|------|------|
| 启动速度 | < 2s 冷启动 |
| 包体积 | 安装包 < 20MB（不含离线缓存） |
| 内存占用 | 空闲时 < 100MB |
| 安全性 | 凭证全链路加密，永不明文落盘 |
| 可扩展性 | 新增 Pack / Adapter 无需修改核心代码；新增 Module 仅需在 Orchestrator 注册路由 |
| 国际化 | 首版中文，预留 i18n 架构 |
| 日志 | 通过 `@tauri-apps/plugin-log` 记录安装过程完整日志，便于排错 |
| 自动更新 | 通过 `@tauri-apps/plugin-updater` 支持 C-Box 自身版本检查和更新 |
| 网络容错 | 下载失败自动重试（最多 3 次）、支持系统代理配置（企业内网场景） |
