import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { useCallback, useEffect, useRef } from 'react'

import { WithAiActionBase, WithAiActionBaseExecute, WithAiActionBaseProps } from './Base'

import { ConversationType } from '@/services/aiConversation'
import { logger } from '@/utils/logger'

export type WithAiActionProps<T = unknown> = Omit<WithAiActionBaseProps<T>, 'execute'> & {
  name?: string
  conversation?: {
    id?: string
    title?: string
    type?: ConversationType
  }
  getPrompt?: (feedback: string) => string
}

/**
 * `execute_ai` 命令派发的事件载荷
 *
 * - 成功：`{ status: 'ok', data: T }`
 * - 失败：`{ status: 'error', message: string }`
 */
export type AiServiceEventPayload<T> =
  | { status: 'ok'; data: T }
  | { status: 'error'; message: string }

/** `execute_ai` 命令入参（与后端 `ExecuteAiPayload` 对齐，camelCase） */
type ExecuteAiInvokeArgs = {
  eventName: string
  userPrompt: string
  modelId?: string
  /** 已有会话 ID；未传则后端新建会话 */
  conversationId?: string
  /** 会话类型（仅在新建会话时生效） */
  conversationType?: ConversationType
  /** 会话标题（仅在新建会话时生效） */
  title?: string
}

/**
 * 把事件载荷转换为 Promise resolve / reject
 *
 * 拆分为独立函数以保持外层 listener 的圈复杂度 < 5
 */
function settleByPayload<T>(
  payload: AiServiceEventPayload<T>,
  resolve: (data: T) => void,
  reject: (err: Error) => void,
) {
  if (payload.status === 'ok') {
    resolve(payload.data)
    return
  }
  reject(new Error(payload.message))
}

export function WithAiAction<T = unknown>({
  name = 'default',
  conversation,
  getPrompt,
  ...props
}: WithAiActionProps<T>) {
  /** 收集尚未触发的 unlisten，组件卸载时统一兑底清理，避免泄漏 */
  const pendingUnlistensRef = useRef<Set<UnlistenFn>>(new Set())

  const execute = useCallback<WithAiActionBaseExecute<T>>(
    async ({ userFeedback = '', modelId }) => {
      const eventName = `ai-action/${name}/${Date.now()}`
      const userPrompt = getPrompt?.(userFeedback) || userFeedback

      return new Promise<T>((resolve, reject) => {
        let unlisten: UnlistenFn | undefined

        const cleanup = () => {
          if (!unlisten) return
          pendingUnlistensRef.current.delete(unlisten)
          unlisten()
          unlisten = undefined
        }

        listen<AiServiceEventPayload<T>>(eventName, (event) => {
          logger.debug('[AiActionV2] event received', { eventName, payload: event.payload })
          cleanup()
          settleByPayload(event.payload, resolve as (data: T) => void, reject)
        })
          .then((fn) => {
            unlisten = fn
            pendingUnlistensRef.current.add(fn)

            const args: ExecuteAiInvokeArgs = {
              eventName,
              userPrompt,
              modelId,
              conversationId: conversation?.id,
              conversationType: conversation?.type,
              title: conversation?.title,
            }
            return invoke<string>('execute_ai', { payload: args })
          })
          .catch((err) => {
            cleanup()
            reject(err instanceof Error ? err : new Error(String(err)))
          })
      })
    },
    [getPrompt, name, conversation?.id, conversation?.type, conversation?.title],
  )
  // 组件卸载时兜底：清理所有还未触发的 listener
  useEffect(
    () => () => {
      pendingUnlistensRef.current.forEach((fn) => fn())
      pendingUnlistensRef.current.clear()
    },
    [],
  )

  return <WithAiActionBase {...props} execute={execute} />
}
