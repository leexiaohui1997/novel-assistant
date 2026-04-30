import { invoke } from '@tauri-apps/api/core'

import { PaginatedResult } from '@/types/common'
import { logger } from '@/utils/logger'

/** 模型信息实体（从供应商拉取到的原始数据） */
export interface ModelInfo {
  /** 模型 ID（如 gpt-4o, claude-3-opus 等） */
  modelId: string
  /** 模型名称（可编辑） */
  modelName: string
}

/** 数据库中的模型实体（带供应商名称） */
export interface Model {
  /** 模型记录 ID */
  id: string
  /** 所属供应商 ID */
  providerId: string
  /** 供应商名称（JOIN 带出） */
  providerName: string
  /** 模型标识（如 gpt-4o） */
  modelId: string
  /** 模型别名（展示用名称） */
  alias: string
  /** 是否默认模型 */
  isDefault: boolean
  /** 是否启用 */
  isEnabled: boolean
  /** 创建时间 */
  createdAt: string
  /** 更新时间 */
  updatedAt: string
}

/** 新增模型项 */
export interface NewModelItem {
  modelId: string
  alias: string
}

/** 批量添加模型请求参数 */
export interface BatchAddModelsParams {
  providerId: string
  models: NewModelItem[]
}

/**
 * 从指定供应商拉取可用模型列表
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

/**
 * 批量添加模型
 */
export async function addModels(payload: BatchAddModelsParams): Promise<Model[]> {
  try {
    logger.debug('调用批量添加模型 API:', payload)
    const result = await invoke<Model[]>('add_models', { payload })
    logger.debug('模型批量添加成功:', result.length, '条')
    return result
  } catch (error) {
    logger.error('批量添加模型失败:', error)
    throw error
  }
}

/**
 * 分页获取模型列表（带供应商名称）
 */
export async function getModelsWithPagination(
  page: number,
  pageSize: number,
): Promise<PaginatedResult<Model>> {
  try {
    logger.debug('调用分页获取模型列表 API:', { page, pageSize })
    const result = await invoke<PaginatedResult<Model>>('get_models_with_pagination', {
      page,
      pageSize,
    })
    logger.debug('获取到模型列表:', result.data.length, '条，总数:', result.total)
    return result
  } catch (error) {
    logger.error('分页获取模型列表失败:', error)
    throw error
  }
}

/**
 * 删除模型
 */
export async function deleteModel(id: string): Promise<void> {
  try {
    logger.debug('调用删除模型 API:', id)
    await invoke('delete_model', { id })
    logger.debug('模型删除成功:', id)
  } catch (error) {
    logger.error('删除模型失败:', error)
    throw error
  }
}

/**
 * 切换模型启用状态
 */
export async function toggleModelEnabled(id: string, isEnabled: boolean): Promise<Model> {
  try {
    logger.debug('调用切换模型启用状态 API:', { id, isEnabled })
    const result = await invoke<Model>('toggle_model_enabled', { id, isEnabled })
    logger.debug('模型启用状态更新成功:', result)
    return result
  } catch (error) {
    logger.error('切换模型启用状态失败:', error)
    throw error
  }
}

/**
 * 更新模型别名
 */
export async function updateModelAlias(id: string, alias: string): Promise<Model> {
  try {
    logger.debug('调用更新模型别名 API:', { id, alias })
    const result = await invoke<Model>('update_model_alias', { id, alias })
    logger.debug('模型别名更新成功:', result)
    return result
  } catch (error) {
    logger.error('更新模型别名失败:', error)
    throw error
  }
}
