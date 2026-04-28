import { invoke } from '@tauri-apps/api/core'

import { PaginatedResult } from '@/types/common'
import { logger } from '@/utils/logger'

/** 供应商实体 */
export interface Provider {
  id: string
  name: string
  baseUrl: string
  apiKey?: string
  isEnabled: boolean
  createdAt: string
  updatedAt: string
}

/** 创建供应商请求参数 */
export interface CreateProviderParams {
  name: string
  baseUrl: string
  apiKey?: string
  isEnabled?: boolean
}

/** 更新供应商请求参数 */
export interface UpdateProviderParams {
  name: string
  baseUrl: string
  apiKey?: string
  isEnabled?: boolean
}

/**
 * 创建新供应商
 */
export async function createProvider(params: CreateProviderParams): Promise<Provider> {
  try {
    logger.debug('调用创建供应商 API:', params)
    const result = await invoke<Provider>('create_provider', { provider: params })
    logger.debug('供应商创建成功:', result)
    return result
  } catch (error) {
    logger.error('创建供应商失败:', error)
    throw error
  }
}

/**
 * 分页获取供应商列表
 */
export async function getProvidersWithPagination(
  page: number,
  pageSize: number,
): Promise<PaginatedResult<Provider>> {
  try {
    logger.debug('调用分页获取供应商列表 API:', { page, pageSize })
    const result = await invoke<PaginatedResult<Provider>>('get_providers_with_pagination', {
      page,
      pageSize,
    })
    logger.debug('获取到供应商列表:', result.data.length, '条，总数:', result.total)
    return result
  } catch (error) {
    logger.error('分页获取供应商列表失败:', error)
    throw error
  }
}

/**
 * 更新供应商信息
 */
export async function updateProvider(id: string, params: UpdateProviderParams): Promise<Provider> {
  try {
    logger.debug('调用更新供应商 API:', id, params)
    const result = await invoke<Provider>('update_provider', { id, provider: params })
    logger.debug('供应商更新成功:', result)
    return result
  } catch (error) {
    logger.error('更新供应商失败:', error)
    throw error
  }
}

/**
 * 删除供应商
 */
export async function deleteProvider(id: string): Promise<void> {
  try {
    logger.debug('调用删除供应商 API:', id)
    await invoke('delete_provider', { id })
    logger.debug('供应商删除成功:', id)
  } catch (error) {
    logger.error('删除供应商失败:', error)
    throw error
  }
}
