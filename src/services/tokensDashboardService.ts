import { invoke } from '@tauri-apps/api/core'

import type { PaginatedResult } from '@/types/common'

import { logger } from '@/utils/logger'

/** 看板汇总（总请求数 / 输入 tokens / 输出 tokens） */
export interface TokensSummary {
  totalRequests: number
  inputTokens: number
  outputTokens: number
}

/** 模型用量聚合行 */
export interface ModelUsageRow {
  providerId: string
  modelId: string
  providerName: string
  modelName: string
  requestCount: number
  inputTokens: number
  outputTokens: number
  totalTokens: number
}

/** AI 调用记录条目 */
export interface AiCallLogItem {
  id: string
  providerId: string
  modelId: string
  modelName: string
  providerName: string
  inputTokens: number
  outputTokens: number
  totalTokens: number
  durationMs: number
  message: string
  response: string | null
  thinkingContent: string | null
  status: string
  errorMessage: string | null
  callTime: string
  createdAt: string
}

/** 时间范围参数（毫秒时间戳） */
export interface TokensRangeParams {
  startMs: number
  endMs: number
}

/** 获取 Tokens 看板汇总 */
export async function fetchTokensSummary(range: TokensRangeParams): Promise<TokensSummary> {
  try {
    return await invoke<TokensSummary>('get_tokens_summary', {
      startMs: range.startMs,
      endMs: range.endMs,
    })
  } catch (error) {
    logger.error('获取 Tokens 汇总失败:', error)
    throw error
  }
}

/** 获取模型使用汇总 */
export async function fetchTokensModelUsage(range: TokensRangeParams): Promise<ModelUsageRow[]> {
  try {
    return await invoke<ModelUsageRow[]>('get_tokens_model_usage', {
      startMs: range.startMs,
      endMs: range.endMs,
    })
  } catch (error) {
    logger.error('获取模型使用汇总失败:', error)
    throw error
  }
}

/** 分页获取 AI 调用记录 */
export async function fetchAiCallLogs(
  range: TokensRangeParams,
  page: number,
  pageSize: number,
): Promise<PaginatedResult<AiCallLogItem>> {
  try {
    return await invoke<PaginatedResult<AiCallLogItem>>('list_ai_call_logs', {
      startMs: range.startMs,
      endMs: range.endMs,
      page,
      pageSize,
    })
  } catch (error) {
    logger.error('获取 AI 调用记录失败:', error)
    throw error
  }
}
