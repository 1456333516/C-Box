## Why

C-Box 是一个面向 AI 时代的开发环境便携管理工具。当前 AI 工具链碎片化严重（Claude Code、Codex、Gemini CLI、MCP 服务等各自独立配置），每换一台设备就需重复安装运行时、配置技能、填入密钥，且配置散落在不同路径的不同文件中无法可视化管理。MVP 阶段首先解决最基础的问题：**自动检测缺失的开发环境依赖并提供一键安装**，为后续技能面板、凭证管理等模块奠定基础。

## What Changes

- 基于 Tauri 2 + Vue 3 + TypeScript + Rust 搭建完整的桌面应用骨架
- 实现 Rust 核心引擎：Pack 加载器、生命周期状态机、Orchestrator（类型化 RPC 编排器）、配置管理器
- 实现 TOML 声明式 Pack 规范解析（`manifest.toml`），支持检测命令、版本正则、多平台安装方式
- 实现 DAG 依赖拓扑排序，确保 Pack 按依赖顺序安装
- 实现环境锁文件（`environment.lock.toml`）自动生成，记录精确安装状态
- 实现 Shell 命令执行安全策略（统一脚本执行器 + 命令模板校验）
- 实现 Pack 生命周期状态机（未检测 → 检测中 → 未安装 → 下载中 → 安装中 → 已安装 → 已配置，含错误态和可重试机制）
- 状态通过 `@tauri-apps/plugin-store` 持久化，支持断点恢复
- 提供首批 6 个内置 Pack：Node.js、Python、Git、uv、Claude Code、npm
- 实现基础 Vue 3 可视化界面：Pack 列表展示、状态指示灯、一键检测、一键安装、进度条
- 平台抽象层（PAL）封装 Windows 特有逻辑（PATH 刷新 `WM_SETTINGCHANGE`、UAC 权限提升等）
- 处理边缘场景：检测命令 locale 差异（fallback_command）、多版本冲突警告、网络不可用提示、批量安装部分失败处理

## Capabilities

### New Capabilities

- `pack-loader`: TOML 声明式 Pack 加载器，负责扫描、解析 `manifest.toml`，构建依赖图并执行 DAG 拓扑排序
- `pack-state-machine`: Pack 生命周期状态机，管理检测/下载/安装/配置的完整状态流转，含错误态、可重试机制和状态持久化
- `environment-detector`: 环境检测引擎，通过 Shell 命令执行检测系统已安装工具及版本，支持 fallback_command 和 locale 容错
- `pack-installer`: Pack 安装执行器，支持 winget/scoop/url/script/brew 多种安装方式，含 UAC 提权、重启标记、checksum 校验
- `orchestrator`: 核心编排器，类型化 RPC 处理模块间状态变更请求，Tauri Event 处理 UI 进度通知
- `desktop-shell`: Tauri 2 + Vue 3 桌面应用骨架，含基础 UI（Pack 列表、状态灯、进度条、一键操作）

### Modified Capabilities

（无 — 这是全新项目，不存在已有 spec）

## Impact

- **技术栈引入**：Tauri 2.x、Vue 3、TypeScript、Rust，以及 Tauri 官方插件（shell、store、log、updater、fs）
- **系统权限**：部分 Pack 安装需要管理员权限（UAC），需在 manifest 中声明 `requires_admin`
- **文件系统**：创建 `packs/` 目录存放内置 Pack 的 `manifest.toml`；运行时生成 `environment.lock.toml`
- **网络依赖**：安装阶段依赖网络下载，需处理断网、代理、重试等场景
- **平台限制**：MVP 仅支持 Windows（P0），Linux/macOS 的 PAL 实现留待后续版本
