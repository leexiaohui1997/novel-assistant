import { useCallback, useRef, useState } from 'react'

export function useLoading<U = unknown, A = unknown>(fn: (...args: A[]) => Promise<U>) {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const [result, setResult] = useState<U | null>(null)
  const executeId = useRef(0)

  const execute = useCallback(
    async (...args: A[]) => {
      executeId.current += 1
      const currentId = executeId.current
      try {
        setLoading(true)
        const result = await fn(...args)
        if (currentId === executeId.current) {
          setResult(result)
          setError(null)
        }
        return result
      } catch (error) {
        if (currentId === executeId.current) {
          setError(error as Error)
        }
        throw error
      } finally {
        if (currentId === executeId.current) {
          setLoading(false)
        }
      }
    },
    [fn],
  )

  return {
    execute,
    loading,
    result,
    error,
  }
}
