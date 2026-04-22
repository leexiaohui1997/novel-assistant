#!/usr/bin/env node

/**
 * Release Tag 管理脚本
 *
 * 功能：
 * 1. 检查工作区是否干净
 * 2. 同步本地和远程 tags
 * 3. 查找最大版本号
 * 4. 支持 CLI 参数和交互式输入
 * 5. 创建并推送新的 release tag
 */

import { execSync } from 'child_process'

import { select, confirm } from '@inquirer/prompts'
import { Command } from 'commander'

// 颜色输出
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
}

function log(message, color = colors.reset) {
  console.log(`${color}${message}${colors.reset}`)
}

function error(message) {
  log(`❌ ${message}`, colors.red)
}

function success(message) {
  log(`✅ ${message}`, colors.green)
}

function info(message) {
  log(`ℹ️  ${message}`, colors.blue)
}

function warning(message) {
  log(`⚠️  ${message}`, colors.yellow)
}

// 执行 shell 命令
function exec(command, options = {}) {
  try {
    return execSync(command, {
      stdio: 'pipe',
      encoding: 'utf-8',
      ...options,
    }).trim()
  } catch (err) {
    if (options.ignoreError) {
      return ''
    }
    throw err
  }
}

// 检查是否有未提交的更改
function checkWorkingDirectory() {
  info('检查工作区状态...')

  const status = exec('git status --porcelain', { ignoreError: true })

  if (status) {
    error('工作区有未提交的更改！')
    log('\n未提交的文件：', colors.yellow)
    log(status)
    log('\n请先提交或暂存这些更改后再运行此脚本。', colors.yellow)
    process.exit(1)
  }

  success('工作区干净')
}

// 同步本地和远程 tags
function syncTags() {
  info('同步 tags...')

  // 获取远程 tags
  const remoteTags = exec('git ls-remote --tags origin', { ignoreError: true })
    .split('\n')
    .filter((line) => line.includes('refs/tags/release-v'))
    .map((line) => line.split('refs/tags/')[1])
    .filter(Boolean)

  // 获取本地 tags
  const localTags = exec("git tag --list 'release-v*'", { ignoreError: true })
    .split('\n')
    .filter(Boolean)

  // 找出需要删除的本地 tags（不在远程的）
  const toDelete = localTags.filter((tag) => !remoteTags.includes(tag))

  // 找出需要同步的远程 tags（不在本地的）
  const toFetch = remoteTags.filter((tag) => !localTags.includes(tag))

  if (toDelete.length > 0 || toFetch.length > 0) {
    info(`发现 ${toDelete.length} 个需要删除的本地 tags，${toFetch.length} 个需要同步的远程 tags`)

    // 删除不在远程的本地 tags
    for (const tag of toDelete) {
      info(`删除本地 tag: ${tag}`)
      exec(`git tag -d ${tag}`)
    }

    // 从远程获取 tags
    if (toFetch.length > 0) {
      info('从远程获取 tags...')
      exec('git fetch --tags --force')
    }

    success('Tags 同步完成')
  } else {
    success('本地和远程 tags 已同步')
  }
}

// 解析版本号
function parseVersion(tag) {
  const match = tag.match(/^release-v(\d+)\.(\d+)\.(\d+)(?:-(beta))?$/)
  if (!match) return null

  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    isBeta: !!match[4],
    full: tag,
  }
}

// 查找最大的版本号
function findMaxVersion() {
  info('查找最大版本号...')

  const tags = exec("git tag --list 'release-v*'", { ignoreError: true })
    .split('\n')
    .filter(Boolean)

  let maxVersion = null

  for (const tag of tags) {
    const version = parseVersion(tag)
    if (!version) continue

    if (!maxVersion) {
      maxVersion = version
      continue
    }

    // 比较版本号
    if (version.major > maxVersion.major) {
      maxVersion = version
    } else if (version.major === maxVersion.major) {
      if (version.minor > maxVersion.minor) {
        maxVersion = version
      } else if (version.minor === maxVersion.minor) {
        if (version.patch > maxVersion.patch) {
          maxVersion = version
        }
      }
    }
  }

  if (maxVersion) {
    success(`找到最大版本: ${maxVersion.full}`)
    return maxVersion
  } else {
    warning('没有找到任何 release tag，将从 v0.0.0 开始')
    return { major: 0, minor: 0, patch: 0, isBeta: false, full: 'release-v0.0.0' }
  }
}

// 构建新版本号
function buildNewVersion(currentVersion, updateType, isBeta) {
  let { major, minor, patch } = currentVersion

  switch (updateType) {
    case 'major':
      major += 1
      minor = 0
      patch = 0
      break
    case 'minor':
      minor += 1
      patch = 0
      break
    case 'patch':
      patch += 1
      break
  }

  const versionString = `v${major}.${minor}.${patch}${isBeta ? '-beta' : ''}`
  const tagName = `release-${versionString}`

  return {
    major,
    minor,
    patch,
    isBeta,
    versionString,
    tagName,
  }
}

// 创建并推送 tag
function createAndPushTag(tagName) {
  info(`创建 tag: ${tagName}...`)

  try {
    // 检查 tag 是否已存在
    const existingTag = exec(`git tag --list '${tagName}'`, { ignoreError: true })
    if (existingTag) {
      error(`Tag ${tagName} 已存在！`)
      process.exit(1)
    }

    // 创建 tag
    exec(`git tag -a ${tagName} -m "Release ${tagName}"`)
    success(`Tag ${tagName} 创建成功`)

    // 推送 tag
    info('推送 tag 到远程...')
    exec(`git push origin ${tagName}`)
    success(`Tag ${tagName} 推送成功`)

    log('\n🎉 发布流程完成！', colors.green)
    log(`GitHub Actions 将自动触发构建流程`, colors.cyan)
  } catch (err) {
    error('创建或推送 tag 失败')
    log(err.message, colors.red)

    // 如果创建成功但推送失败，提示用户手动处理
    const tagExists = exec(`git tag --list '${tagName}'`, { ignoreError: true })
    if (tagExists) {
      warning(`Tag ${tagName} 已在本地创建，但推送失败`)
      info(`可以手动执行: git push origin ${tagName}`)
    }

    process.exit(1)
  }
}

// 主函数
async function main(options) {
  const { mode, beta, quiet } = options

  log('\n🚀 Release Tag 管理工具', colors.cyan)
  log('═'.repeat(50), colors.cyan)

  try {
    // 1. 检查工作区
    checkWorkingDirectory()

    // 2. 同步 tags
    syncTags()

    // 3. 查找最大版本
    const currentVersion = findMaxVersion()

    // 4. 确定更新方式和 beta 状态
    let updateType = mode
    let isBeta = beta

    // 如果不是静默模式，使用 inquirer 进行交互
    if (!quiet) {
      // 如果没有指定 mode，询问用户
      updateType = await select({
        type: 'list',
        name: 'updateType',
        message: '请选择版本更新方式',
        choices: [
          { name: `大版本 (major) - 不兼容的 API 修改`, value: 'major' },
          { name: `小版本 (minor) - 向下兼容的功能性新增`, value: 'minor' },
          { name: `补丁 (patch) - 向下兼容的问题修正`, value: 'patch' },
        ],
        default: updateType,
      })

      // 如果没有指定 beta，询问用户
      isBeta = await confirm({
        type: 'confirm',
        name: 'isBeta',
        message: '是否为 Beta 版本？',
        default: isBeta,
      })
    } else {
      // 静默模式下使用默认值
      if (!updateType) {
        updateType = 'patch'
      }
      if (isBeta === undefined) {
        isBeta = false
      }
    }

    // 5. 构建新版本
    const newVersion = buildNewVersion(currentVersion, updateType, isBeta)

    // 6. 显示版本信息
    log('\n📋 即将创建的新版本信息:', colors.cyan)
    log(`标签名: ${newVersion.tagName}`, colors.blue)
    log(`版本号: ${newVersion.versionString}`, colors.blue)
    log('')

    // 7. 如果不是静默模式，确认操作
    if (!quiet) {
      const confirmed = await confirm({
        type: 'confirm',
        name: 'confirmed',
        message: '确认创建此版本？',
        default: false,
      })

      if (!confirmed) {
        info('操作已取消')
        process.exit(0)
      }
    }

    // 8. 创建并推送 tag
    createAndPushTag(newVersion.tagName)
  } catch (err) {
    error('脚本执行出错')
    log(err.message, colors.red)
    process.exit(1)
  }
}

// 配置 CLI
const program = new Command()

program
  .name('create-release-tag')
  .description('创建并发布新的 release tag')
  .version('1.0.0')
  .option('-m, --mode <type>', '版本更新方式 (major/minor/patch)', 'patch')
  .option('-b, --beta', '是否为 beta 版本', false)
  .option('-q, --quiet', '静默模式，跳过交互确认', false)
  .action((options) => {
    // 验证 mode 参数
    const validModes = ['major', 'minor', 'patch']
    if (!validModes.includes(options.mode)) {
      error(`无效的 mode 参数: ${options.mode}`)
      log(`有效的选项: ${validModes.join(', ')}`, colors.yellow)
      process.exit(1)
    }

    main(options)
  })

program.parse(process.argv)
