import { useCallback, useState } from 'react'

import { getLogFileContent } from '@/services/logService'
import { logger } from '@/utils/logger'

export interface UseLogContentReturn {
  /** 日志文件内容 */
  content: string | null
  /** 加载状态 */
  loading: boolean
  /** 错误信息 */
  error: string | null
  /** 加载指定文件的内容 */
  loadContent: (filename: string) => Promise<void>
  /** 清除当前内容 */
  clearContent: () => void
}

/**
 * 管理单个日志文件内容加载的 Hook
 *
 * @returns 日志内容状态和操作方法
 */
export function useLogContent(): UseLogContentReturn {
  const [content, setContent] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // 加载指定文件的内容
  const loadContent = useCallback(async (filename?: string) => {
    if (!filename) {
      setContent(null)
      return
    }

    setLoading(true)
    setError(null)
    try {
      const fileContent = await getLogFileContent(filename)
      setContent(fileContent)
    } catch (err) {
      logger.error('读取日志文件失败:', err)
      setError(err instanceof Error ? err.message : '读取日志文件失败')
      setContent(null)
    } finally {
      setLoading(false)
    }
  }, [])

  // 清除当前内容
  const clearContent = useCallback(() => {
    setContent(null)
    setError(null)
  }, [])

  return {
    content,
    loading,
    error,
    loadContent,
    clearContent,
  }
}
