import { invoke } from '@tauri-apps/api/core'

import { Character } from '@/types/character'
import { logger } from '@/utils/logger'

/**
 * 创建角色参数
 */
export interface CreateCharacterParams {
  novelId: string
  name: string
  gender: string
  background?: string
  appearance?: string
  personality?: string
  additionalInfo?: string
}

/**
 * 更新角色参数
 */
export interface UpdateCharacterParams {
  id: string
  name: string
  gender: string
  background?: string
  appearance?: string
  personality?: string
  additionalInfo?: string
}

/**
 * 创建角色
 */
export async function createCharacter(params: CreateCharacterParams): Promise<Character> {
  try {
    logger.debug('调用创建角色 API:', params)

    const result = await invoke<Character>('create_character', {
      novelId: params.novelId,
      name: params.name,
      gender: params.gender,
      background: params.background,
      appearance: params.appearance,
      personality: params.personality,
      additionalInfo: params.additionalInfo,
    })

    logger.debug('角色创建成功:', result)
    return result
  } catch (error) {
    logger.error('创建角色失败:', error)
    throw error
  }
}

/**
 * 根据小说 ID 获取所有角色
 */
export async function getCharactersByNovel(novelId: string): Promise<Character[]> {
  try {
    logger.debug('调用获取角色列表 API:', novelId)

    const result = await invoke<Character[]>('get_characters_by_novel', {
      novelId,
    })

    logger.debug('获取到角色列表:', result.length, '个')
    return result
  } catch (error) {
    logger.error('获取角色列表失败:', error)
    throw error
  }
}

/**
 * 根据 ID 获取角色
 */
export async function getCharacterById(id: string): Promise<Character> {
  try {
    logger.debug('调用获取角色详情 API:', id)

    const result = await invoke<Character>('get_character_by_id', {
      id,
    })

    logger.debug('获取到角色详情:', result)
    return result
  } catch (error) {
    logger.error('获取角色详情失败:', error)
    throw error
  }
}

/**
 * 更新角色
 */
export async function updateCharacter(params: UpdateCharacterParams): Promise<Character> {
  try {
    logger.debug('调用更新角色 API:', params)

    const result = await invoke<Character>('update_character', {
      id: params.id,
      name: params.name,
      gender: params.gender,
      background: params.background,
      appearance: params.appearance,
      personality: params.personality,
      additionalInfo: params.additionalInfo,
    })

    logger.debug('角色更新成功:', result)
    return result
  } catch (error) {
    logger.error('更新角色失败:', error)
    throw error
  }
}

/**
 * 删除角色
 */
export async function deleteCharacter(id: string): Promise<void> {
  try {
    logger.debug('调用删除角色 API:', id)

    await invoke<void>('delete_character', {
      id,
    })

    logger.debug('角色删除成功')
  } catch (error) {
    logger.error('删除角色失败:', error)
    throw error
  }
}
