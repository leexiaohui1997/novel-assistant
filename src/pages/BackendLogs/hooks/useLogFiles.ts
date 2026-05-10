import { useCallback, useEffect, useRef, useState } from 'react'

import { getLogFiles, type LogFileInfo } from '@/services/logService'
import { logger } from '@/utils/logger'

export interface UseLogFilesOptions {
  /** 自动刷新间隔（毫秒），设置为 false 禁用自动刷新 */
  autoRefreshInterval?: number | false
}

export interface UseLogFilesReturn {
  /** 日志文件列表 */
  logFiles: LogFileInfo[]
  /** 加载状态 */
  loading: boolean
  /** 错误信息 */
  error: string | null
  /** 手动刷新日志文件列表 */
  refreshLogFiles: () => Promise<void>
}

/**
 * 管理日志文件列表和自动刷新的 Hook
 *
 * @param options - 配置选项
 * @returns 日志文件列表状态和操作方法
 */
export function useLogFiles(options?: UseLogFilesOptions): UseLogFilesReturn {
  const [logFiles, setLogFiles] = useState<LogFileInfo[]>([])
  const [loading, setLoading] = useState(true) // 初始为 loading 状态
  const [error, setError] = useState<string | null>(null)
  const autoRefreshRef = useRef<NodeJS.Timeout>()
  const initializedRef = useRef(false)

  const autoRefreshInterval = options?.autoRefreshInterval ?? 30000

  // 刷新日志文件列表
  const refreshLogFiles = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const files = await getLogFiles()
      setLogFiles(files)
    } catch (err) {
      logger.error('刷新日志文件列表失败:', err)
      setError(err instanceof Error ? err.message : '获取日志文件列表失败')
    } finally {
      setLoading(false)
    }
  }, [])

  // 自动刷新和初始加载
  useEffect(() => {
    // 初始加载（仅执行一次）
    if (!initializedRef.current) {
      initializedRef.current = true
      void refreshLogFiles()
    }

    // 设置自动刷新
    if (autoRefreshInterval !== false) {
      autoRefreshRef.current = setInterval(() => {
        void refreshLogFiles()
      }, autoRefreshInterval)
    }

    return () => {
      if (autoRefreshRef.current) {
        clearInterval(autoRefreshRef.current)
      }
    }
  }, [refreshLogFiles, autoRefreshInterval])

  return {
    logFiles,
    loading,
    error,
    refreshLogFiles,
  }
}
