import { DefaultOptionType } from 'antd/es/select'

/**
 * 角色性别
 */
export enum CharacterGender {
  Male = 'male',
  Female = 'female',
  Other = 'other',
  Unknown = 'unknown',
}

/**
 * 角色性别标签
 */
export const CharacterGenderLabels: Record<CharacterGender, string> = {
  [CharacterGender.Male]: '男',
  [CharacterGender.Female]: '女',
  [CharacterGender.Other]: '其他',
  [CharacterGender.Unknown]: '未知',
}

/**
 * 角色选项
 */
export const CharacterGenderOptions: DefaultOptionType[] = [
  { value: CharacterGender.Male, label: CharacterGenderLabels[CharacterGender.Male] },
  { value: CharacterGender.Female, label: CharacterGenderLabels[CharacterGender.Female] },
  { value: CharacterGender.Other, label: CharacterGenderLabels[CharacterGender.Other] },
  { value: CharacterGender.Unknown, label: CharacterGenderLabels[CharacterGender.Unknown] },
]

/**
 * 角色类型
 */
export interface Character {
  id: string
  name: string
  gender: CharacterGender
  novelId: string
  background?: string
  appearance?: string
  personality?: string
  additionalInfo?: string
  createdAt: string
  updatedAt: string
}
