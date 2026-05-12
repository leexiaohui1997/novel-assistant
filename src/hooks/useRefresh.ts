import { useCallback, useEffect, useState } from 'react'

import { logger } from '@/utils/logger'

export type UseRefreshProps<T = unknown> = {
  actionName?: string
  initialData: T
  refreshFn: () => Promise<T>
  onSuccess?: (data: T) => void
  onError?: (error: unknown) => void
}

export function useRefresh<T = unknown>({
  refreshFn,
  onSuccess,
  onError,
  actionName,
  initialData,
}: UseRefreshProps<T>) {
  const [data, setData] = useState(initialData)
  const [loading, setLoading] = useState(true)
  const [refreshPromise, setRefreshPromise] = useState<{
    resolve: (value: T) => void
    reject: (reason?: unknown) => void
  }>()

  const refresh = useCallback(() => {
    const promise = new Promise<T>((resolve, reject) => {
      setRefreshPromise({
        resolve,
        reject,
      })
    })
    return promise
  }, [])

  useEffect(() => {
    let cancelled = false

    // eslint-disable-next-line complexity
    const loader = async () => {
      try {
        setLoading(true)
        const data = await refreshFn()
        if (!cancelled) {
          setData(data)
          onSuccess?.(data)
          refreshPromise?.resolve(data)
          if (actionName) {
            logger.info(`${actionName}成功`, data)
          }
        }
      } catch (err) {
        if (!cancelled) {
          onError?.(err)
          refreshPromise?.reject(err)
          if (actionName) {
            logger.error(`${actionName}失败`, err)
          }
        }
      } finally {
        if (!cancelled) {
          setLoading(false)
        }
      }
    }

    void loader()

    return () => {
      cancelled = true
    }
  }, [refreshFn, onSuccess, onError, refreshPromise, actionName])

  return {
    data,
    loading,
    refresh,
    setData,
  }
}
