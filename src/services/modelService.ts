import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/utils/logger'

/** 模型信息实体 */
export interface ModelInfo {
  /** 模型 ID（如 gpt-4o, claude-3-opus 等） */
  modelId: string
  /** 模型名称（与 modelId 相同） */
  modelName: string
}

/**
 * 从指定供应商拉取可用模型列表
 *
 * @param providerId - 供应商 ID
 * @returns 模型信息列表
 *
 * @example
 * ```typescript
 * const models = await fetchProviderModels('provider-id-123')
 * console.log(models) // [{ modelId: 'gpt-4o', modelName: 'gpt-4o' }, ...]
 * ```
 */
export async function fetchProviderModels(providerId: string): Promise<ModelInfo[]> {
  try {
    logger.debug('调用拉取供应商模型 API:', { providerId })
    const result = await invoke<ModelInfo[]>('fetch_provider_models', { providerId })
    logger.debug('拉取到模型列表:', result.length, '条')
    return result
  } catch (error) {
    logger.error('拉取供应商模型失败:', error)
    throw error
  }
}
