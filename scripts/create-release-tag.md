# Release Tag 管理脚本

## 功能说明

这个脚本用于自动化管理项目的 release tag，支持 CLI 参数和交互式操作，主要功能包括：

1. ✅ 检查工作区是否干净（有未提交更改会提示并退出）
2. 🔄 同步本地和远程 tags
3. 🔍 自动查找最大的版本号
4. 💬 支持 CLI 参数和 inquirer 交互式输入
5. 🚀 创建并推送新的 release tag

## 使用方法

### 方式一：使用 npm script（推荐）

```bash
# 交互式模式（默认）
pnpm release

# 指定更新方式
pnpm release -- --mode minor

# 指定为 beta 版本
pnpm release -- --mode patch --beta

# 静默模式（跳过交互确认）
pnpm release -- --mode patch --quiet
```

### 方式二：直接运行脚本

```bash
# 交互式模式
node scripts/create-release-tag.js

# 使用 CLI 参数
node scripts/create-release-tag.js --mode minor --beta
node scripts/create-release-tag.js -m patch -q
```

### 方式三：直接执行（需要执行权限）

```bash
./scripts/create-release-tag.js --mode major
./scripts/create-release-tag.js -m minor -b -q
```

## CLI 参数说明

| 参数            | 简写 | 说明                             | 默认值  |
| --------------- | ---- | -------------------------------- | ------- |
| `--mode <type>` | `-m` | 版本更新方式 (major/minor/patch) | `patch` |
| `--beta`        | `-b` | 是否为 beta 版本                 | `false` |
| `--quiet`       | `-q` | 静默模式，跳过交互确认           | `false` |

**示例：**

```bash
# 补丁更新，非 beta，需要确认
node scripts/create-release-tag.js

# 小版本更新，beta 版，静默模式
node scripts/create-release-tag.js -m minor -b -q

# 大版本更新，非 beta，需要确认
node scripts/create-release-tag.js --mode major
```

## 交互流程

### 交互式模式（默认）

运行脚本后，会按以下步骤进行：

#### 1. 自动检查

- 检查工作区是否有未提交的更改
- 同步本地和远程的 tags

#### 2. 版本信息展示

显示当前最大版本号，例如：

```
找到最大版本: release-v0.0.1-beta
```

#### 3. 选择更新方式（如果未指定 --mode）

使用 inquirer 提供交互式选择：

```
? 请选择版本更新方式 (Use arrow keys)
❯ 大版本 (major) - 不兼容的 API 修改
  小版本 (minor) - 向下兼容的功能性新增
  补丁 (patch) - 向下兼容的问题修正
```

**版本号递进规则：**

- **大版本 (major)**: `0.0.1` → `1.0.0`
- **小版本 (minor)**: `0.0.1` → `0.1.0`
- **补丁 (patch)**: `0.0.1` → `0.0.2`

#### 4. 选择是否为 Beta 版本（如果未指定 --beta）

```
? 是否为 Beta 版本？ (y/N)
```

#### 5. 显示版本信息并确认（如果未指定 --quiet）

```
📋 即将创建的新版本信息:
标签名: release-v0.1.0-beta
版本号: v0.1.0-beta

? 确认创建此版本？ (y/N)
```

#### 6. 创建并推送

确认后会自动：

- 创建 annotated tag
- 推送到远程仓库
- 触发 GitHub Actions 构建流程

### 静默模式（--quiet）

使用 `-q` 或 `--quiet` 参数时，脚本会：

- 跳过所有交互确认
- 使用 CLI 参数或默认值
- 直接创建并推送 tag

```bash
# 快速创建补丁版本
node scripts/create-release-tag.js -q

# 指定参数后静默执行
node scripts/create-release-tag.js -m minor -b -q
```

## Tag 格式说明

Tag 命名格式：`release-vX.Y.Z[-beta]`

- `X.Y.Z` - 语义化版本号（major.minor.patch）
- `-beta` - 可选，表示测试版本

**示例：**

- `release-v1.0.0` - 正式版本
- `release-v1.0.0-beta` - Beta 版本
- `release-v0.2.3` - 补丁版本
- `release-v2.0.0` - 大版本更新

## 注意事项

⚠️ **使用前请确保：**

1. 工作区是干净的（所有更改已提交）
2. 有权限推送到远程仓库
3. 网络连接正常（需要同步远程 tags）

💡 **最佳实践：**

- Beta 版本用于测试阶段
- 正式发布前移除 `-beta` 后缀
- 遵循语义化版本规范
- 重要更新记得更新 CHANGELOG

## 故障排除

### 问题：工作区有未提交的更改

**解决：** 先提交或暂存所有更改

```bash
git add .
git commit -m "your message"
```

### 问题：Tag 已存在

**解决：** 删除本地和远程的重复 tag

```bash
git tag -d release-vX.Y.Z
git push origin :refs/tags/release-vX.Y.Z
```

### 问题：推送失败

**解决：** 检查网络连接和权限，然后手动推送

```bash
git push origin release-vX.Y.Z
```

## GitHub Actions 集成

推送 tag 后会自动触发 `.github/workflows/build.yml` 中配置的构建流程，无需手动操作。
