# 小说创作助手 (Novel Assistant)

一个基于 Tauri v2 + React + TypeScript 的企业级桌面应用框架。

## 🚀 技术栈

### 前端

- **React 19** - 组件化 UI 框架
- **TypeScript 5** - 类型安全
- **Vite 8** - 极速构建工具
- **Ant Design 6** - 企业级 UI 组件库
- **Redux Toolkit** - 状态管理

### 后端

- **Tauri v2** - 轻量级桌面应用框架
- **Rust** - 高性能系统编程语言
- **Tokio** - 异步运行时
- **Serde** - 序列化/反序列化
- **Tracing** - 结构化日志

## 📦 项目结构

```
novel-assistant/
├── src-tauri/                 # Rust 后端
│   ├── src/
│   │   ├── lib.rs             # 库入口
│   │   └── main.rs            # 应用入口
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                       # React 前端
│   ├── store/                 # Redux Store
│   ├── utils/                 # 工具函数
│   ├── styles/                # 全局样式
│   ├── main.tsx               # 应用入口
│   └── vite-env.d.ts
├── package.json
├── tsconfig.json
└── vite.config.ts
```

## 🛠️ 开发环境要求

- **Node.js** >= 22.19.0
- **pnpm** >= 10.33.0
- **Rust** >= 1.70.0
- **操作系统**: Windows 10+, macOS 10.15+, Linux (带 WebKitGTK)

## 📝 快速开始

### 1. 安装依赖

```bash
pnpm install
```

### 2. 启动开发模式

```bash
pnpm run tauri dev
```

这将同时启动:

- Vite 开发服务器 (http://localhost:5173)
- Tauri 桌面应用窗口

### 3. 构建生产版本

```bash
pnpm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

## 🔧 可用脚本

- `pnpm run dev` - 启动 Vite 开发服务器
- `pnpm run build` - 构建前端静态资源
- `pnpm run preview` - 预览生产构建
- `pnpm run tauri dev` - 启动 Tauri 开发模式
- `pnpm run tauri build` - 构建 Tauri 应用
- `pnpm run lint` - ESLint 代码检查
- `pnpm run format` - Prettier 代码格式化
- `pnpm run type-check` - TypeScript 类型检查

## 📄 许可证

专有软件 - 保留所有权利。详见 [LICENSE](LICENSE) 文件。
