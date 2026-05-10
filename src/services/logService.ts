import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/utils/logger'

export interface LogFileInfo {
  filename: string
  size: number
  modified: number // Unix timestamp (milliseconds)
}

/**
 * 获取日志文件列表
 */
export async function getLogFiles(): Promise<LogFileInfo[]> {
  try {
    logger.debug('调用获取日志文件列表 API')
    const result = await invoke<LogFileInfo[]>('get_log_files')
    logger.debug('获取到日志文件列表:', result.length, '个文件')
    return result
  } catch (error) {
    logger.error('获取日志文件列表失败:', error)
    throw error
  }
}

/**
 * 获取指定日志文件的内容
 */
export async function getLogFileContent(filename: string): Promise<string> {
  try {
    logger.debug('调用获取日志文件内容 API:', filename)
    const content = await invoke<string>('get_log_file_content', { filename })
    logger.debug('成功读取日志文件内容，长度:', content.length)
    return content
  } catch (error) {
    logger.error('读取日志文件内容失败:', error)
    throw error
  }
}
