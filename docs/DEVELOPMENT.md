# 开发指南 (Development Guide)

本文档面向开发者，提供项目的技术架构、开发环境配置和贡献指南。

## 📋 目录

- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [开发环境要求](#开发环境要求)
- [快速开始](#快速开始)
- [可用脚本](#可用脚本)
- [代码规范](#代码规范)
- [架构说明](#架构说明)
- [贡献指南](#贡献指南)

## 🚀 技术栈

### 前端

- **React 19** - 组件化 UI 框架
- **TypeScript 5** - 类型安全
- **Vite 8** - 极速构建工具
- **Ant Design 6** - 企业级 UI 组件库
- **Redux Toolkit** - 状态管理
- **Slate.js** - 富文本编辑器框架
- **React Router** - 路由管理

### 后端

- **Tauri v2** - 轻量级桌面应用框架
- **Rust** - 高性能系统编程语言
- **Tokio** - 异步运行时
- **Serde** - 序列化/反序列化
- **Tracing** - 结构化日志
- **SQLx** - 异步 SQL 工具包
- **SQLite** - 嵌入式数据库

## 📦 项目结构

```
novel-assistant/
├── src-tauri/                 # Rust 后端
│   ├── src/
│   │   ├── ai/                # AI 服务模块
│   │   │   ├── actions/       # AI 动作实现
│   │   │   ├── prompts/       # AI 提示词模板
│   │   │   ├── service.rs     # AI 服务层
│   │   │   └── types.rs       # AI 相关类型定义
│   │   ├── commands/          # Tauri 命令
│   │   │   ├── action_commands.rs
│   │   │   ├── ai_commands.rs
│   │   │   ├── chapter_commands.rs
│   │   │   ├── creation_state_commands.rs
│   │   │   ├── model_commands.rs
│   │   │   ├── novel_commands.rs
│   │   │   ├── provider_commands.rs
│   │   │   └── tag_commands.rs
│   │   ├── config/            # 配置模块
│   │   │   ├── env.rs         # 环境变量
│   │   │   ├── mod.rs
│   │   │   └── paths.rs       # 路径管理
│   │   ├── database/          # 数据库模块
│   │   │   ├── models/        # 数据模型
│   │   │   ├── repositories/  # 数据访问层
│   │   │   ├── error.rs       # 错误处理
│   │   │   ├── mod.rs
│   │   │   └── pool.rs        # 连接池
│   │   ├── logging/           # 日志系统
│   │   │   ├── config.rs
│   │   │   ├── mod.rs
│   │   │   └── setup.rs
│   │   ├── utils/             # 工具函数
│   │   │   ├── macros.rs      # Rust 宏
│   │   │   ├── mod.rs
│   │   │   └── pagination.rs  # 分页工具
│   │   ├── lib.rs             # 库入口
│   │   └── main.rs            # 应用入口
│   ├── migrations/            # 数据库迁移
│   ├── templates/             # Tera 模板
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                       # React 前端
│   ├── components/            # 通用组件
│   │   ├── Editor/            # 富文本编辑器
│   │   ├── NovelBasicForm/    # 小说基础表单
│   │   ├── VersionDrawer/     # 版本抽屉
│   │   ├── layout/            # 布局组件
│   │   └── ...
│   ├── config/                # 配置文件
│   │   ├── index.ts
│   │   ├── menu.config.ts     # 菜单配置
│   │   └── routes.config.tsx  # 路由配置
│   ├── hooks/                 # 自定义 Hooks
│   │   ├── useAiAction.ts     # AI 动作 Hook
│   │   ├── useCreationState.ts
│   │   ├── useEditor.ts
│   │   └── ...
│   ├── pages/                 # 页面组件
│   │   ├── Novels/            # 作品管理页
│   │   ├── CreationDetail/    # 创作详情页
│   │   └── Settings/          # 设置页
│   ├── providers/             # Context Providers
│   │   ├── CreationStateProvider.tsx
│   │   └── EditorProvider.tsx
│   ├── services/              # API 服务层
│   │   ├── aiService.ts
│   │   ├── chapterService.ts
│   │   ├── creationStateService.ts
│   │   ├── modelService.ts
│   │   ├── novelService.ts
│   │   ├── providerService.ts
│   │   └── tagService.ts
│   ├── store/                 # Redux Store
│   │   ├── slices/
│   │   ├── hooks.ts
│   │   └── index.ts
│   ├── styles/                # 全局样式
│   ├── types/                 # TypeScript 类型
│   ├── utils/                 # 工具函数
│   ├── App.tsx                # 根组件
│   └── main.tsx               # 应用入口
├── .codebuddy/                # CodeBuddy 规则
├── .github/                   # GitHub 配置
├── scripts/                   # 构建脚本
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md                  # 产品介绍（主文档）
```

## 🛠️ 开发环境要求

- **Node.js** >= 22.19.0
- **pnpm** >= 10.33.0
- **Rust** >= 1.70.0
- **操作系统**:
  - Windows 10+
  - macOS 10.15+
  - Linux (带 WebKitGTK)

### 推荐工具

- **VS Code** - 推荐的代码编辑器
- **Rust Analyzer** - VS Code Rust 插件
- **ESLint** + **Prettier** - 代码检查和格式化
- **Tauri CLI** - Tauri 开发工具

## 📝 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/leexiaohui1997/novel-assistant.git
cd novel-assistant
```

### 2. 安装依赖

```bash
pnpm install
```

### 3. 配置环境变量（可选）

复制 `.env.example` 为 `.env` 并填写配置：

```bash
cp .env.example .env
```

### 4. 启动开发模式

```bash
pnpm run tauri dev
```

这将同时启动：

- Vite 开发服务器 (http://localhost:5173)
- Tauri 桌面应用窗口

### 5. 构建生产版本

```bash
pnpm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

## 🔧 可用脚本

### 前端脚本

- `pnpm run dev` - 启动 Vite 开发服务器
- `pnpm run build` - 构建前端静态资源
- `pnpm run preview` - 预览生产构建
- `pnpm run lint` - ESLint 代码检查
- `pnpm run lint:fix` - 自动修复 ESLint 问题
- `pnpm run format` - Prettier 代码格式化
- `pnpm run type-check` - TypeScript 类型检查

### Tauri 脚本

- `pnpm run tauri dev` - 启动 Tauri 开发模式
- `pnpm run tauri build` - 构建 Tauri 应用
- `pnpm run tauri icon` - 生成应用图标

### 其他脚本

- `pnpm run prepare` - 安装 Husky Git hooks
- `pnpm run release-tag` - 创建发布标签

## 📐 代码规范

### TypeScript 规范

- 使用严格的类型检查
- 避免使用 `any` 类型
- 为公共 API 提供完整的类型定义
- 使用接口（interface）定义对象结构

### Rust 规范

- 遵循 Rust 官方风格指南
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 为公共函数添加文档注释

### 提交规范

使用 Conventional Commits 格式：

```
<type>(<scope>): <subject>

<body>
```

**Type 类型：**

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档变更
- `style`: 代码格式
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程/辅助工具

**示例：**

```
feat(editor): 添加自动保存功能

- 实现 30 秒自动保存机制
- 添加保存状态指示器
- 支持手动保存快捷键
```

## 🏗️ 架构说明

### 前端架构

```
用户界面 (Components)
    ↓
状态管理 (Redux + Context)
    ↓
业务逻辑 (Hooks + Services)
    ↓
API 调用 (Tauri Invoke)
```

**核心概念：**

- **组件化** - 可复用的 UI 组件
- **状态管理** - Redux Toolkit 管理全局状态，Context 管理局部状态
- **服务层** - 封装 Tauri API 调用
- **Hooks** - 封装业务逻辑和副作用

### 后端架构

```
Tauri Commands
    ↓
Service Layer
    ↓
Repository Layer
    ↓
Database (SQLite)
```

**核心模块：**

- **Commands** - Tauri 命令入口，处理前端请求
- **Services** - 业务逻辑层
- **Repositories** - 数据访问层，封装数据库操作
- **Models** - 数据模型定义
- **AI Module** - AI 服务集成

### 数据流

1. **前端发起请求** → 调用 Service 层方法
2. **Service 层** → 通过 `invoke` 调用 Tauri Command
3. **Tauri Command** → 调用 Service/Repository 处理业务
4. **Repository** → 执行数据库操作
5. **返回结果** → 逐层返回到前端

## 🧪 测试

### 前端测试

```bash
pnpm run test
```

### 后端测试

```bash
cd src-tauri
cargo test
```

## 📚 相关文档

- [工具函数使用规范](src/utils/README.md)
- [Rust 工具函数文档](src-tauri/src/utils/README.md)
- [AI 服务调试指南](src/services/aiService.ts)

## 🤝 贡献指南

### 开发流程

1. **Fork 仓库** - 在 GitHub 上 Fork 本项目
2. **创建分支** - 从 `main` 分支创建功能分支
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **开发功能** - 实现您的功能或修复
4. **编写测试** - 确保代码有适当的测试覆盖
5. **提交代码** - 遵循提交规范
   ```bash
   git add .
   git commit -m "feat: your feature description"
   ```
6. **推送分支** - 推送到您的 Fork
   ```bash
   git push origin feature/your-feature-name
   ```
7. **提交 PR** - 在 GitHub 上创建 Pull Request

### PR 要求

- ✅ 代码符合项目规范
- ✅ 通过所有测试
- ✅ 更新相关文档
- ✅ 提交信息清晰明确
- ✅ 描述 PR 的目的和改动

### 代码审查要点

- 代码质量和可读性
- 性能影响
- 安全性考虑
- 向后兼容性
- 测试覆盖率

## ❓ 常见问题

### Q: 如何调试 Rust 代码？

A: 使用 `tracing` 日志系统，在代码中添加 `tracing::info!` 或 `tracing::debug!`，日志会输出到控制台。

### Q: 前端如何调用后端功能？

A: 使用 `@tauri-apps/api/core` 的 `invoke` 函数：

```typescript
import { invoke } from '@tauri-apps/api/core'
const result = await invoke('command_name', { param1, param2 })
```

### Q: 如何添加新的数据库表？

A:

1. 在 `src-tauri/migrations/` 创建迁移文件
2. 在 `src-tauri/src/database/models/` 定义模型
3. 在 `src-tauri/src/database/repositories/` 创建 Repository
4. 注册到 `AppState`

### Q: 如何添加新的 Tauri 命令？

A:

1. 在 `src-tauri/src/commands/` 创建命令文件
2. 使用 `#[tauri::command]` 装饰器
3. 在 `mod.rs` 中导出
4. 在 `lib.rs` 中注册到 `.invoke_handler()`

## 📞 技术支持

- **Issue** - 在 [GitHub Issues](https://github.com/leexiaohui1997/novel-assistant/issues) 报告问题
- **Discussion** - 参与技术讨论
- **Email** - 联系维护者

---

**注意：** 本文档面向开发者。如果您是普通用户，请查看 [主 README](../README.md) 了解产品功能和使用方法。
