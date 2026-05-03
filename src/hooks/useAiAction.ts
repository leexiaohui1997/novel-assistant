import { invoke } from '@tauri-apps/api/core'
import { useCallback, useState } from 'react'

import { logger } from '@/utils/logger'

export type UseAiActionProps = {
  /** Action 名称 */
  actionName: string
  /** 获取参数的函数 */
  getParams: () => Record<string, unknown>
}

/**
 * 使用 AI Action 的 Hook
 *
 * @example
 * ```tsx
 * const { execute, loading, result } = useAiAction({
 *   actionName: 'recommend_tags',
 *   getParams: () => ({
 *     title: form.getFieldValue('title'),
 *     channel: form.getFieldValue('targetReader'),
 *   }),
 * })
 *
 * // 在按钮点击时调用
 * <Button onClick={execute} loading={loading}>AI 推荐</Button>
 * ```
 */
export function useAiAction<T = unknown>(options: UseAiActionProps) {
  const { actionName, getParams } = options

  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<T | null>(null)
  const [error, setError] = useState<Error | null>(null)

  /**
   * 执行 AI Action
   *
   * @returns Promise<T> Action 返回结果
   */
  const execute = useCallback(async (): Promise<T> => {
    try {
      setLoading(true)
      setError(null)

      const params = getParams()
      logger.debug(`调用 AI Action: ${actionName}`, params)

      const response = await invoke<T>('execute_action', {
        actionName,
        actionParams: params,
      })

      logger.debug(`AI Action 返回结果: ${actionName}`, response)
      setResult(response)
      return response
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err))
      logger.error(`AI Action 执行失败: ${actionName}`, error)
      setError(error)
      throw err
    } finally {
      setLoading(false)
    }
  }, [actionName, getParams])

  return {
    /** 执行函数 */
    execute,
    /** 加载状态 */
    loading,
    /** 执行结果 */
    result,
    /** 错误信息 */
    error,
  }
}
