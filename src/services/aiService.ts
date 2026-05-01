import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/utils/logger'

/**
 * 测试模型是否可用
 * @param modelId 模型 ID
 */
export async function testModel(modelId: string): Promise<{
  success: boolean
  content: string
  usage?: Record<string, unknown>
}> {
  try {
    const result = await invoke<{
      success: boolean
      content: string
      usage?: Record<string, unknown>
    }>('test_model', { modelId })

    logger.info('模型测试成功', result)
    return result
  } catch (error) {
    logger.error('模型测试失败', error)
    throw error
  }
}
