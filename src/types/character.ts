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
export enum CharacterType {
  /** 主角 */
  Protagonist = 'protagonist',
  /** 二号主角 */
  SecondProtagonist = 'second_protagonist',
  /** 三号主角 */
  ThirdProtagonist = 'third_protagonist',
  /** 配角 */
  Supporting = 'supporting',
  /** 次要配角 */
  MinorSupporting = 'minor_supporting',
  /** 重要敌对 */
  MajorAntagonist = 'major_antagonist',
  /** 次要敌对 */
  MinorAntagonist = 'minor_antagonist',
  /** 敌对阵营 */
  AntagonistFaction = 'antagonist_faction',
}

/**
 * 角色类型标签
 */
export const CharacterTypeLabels: Record<CharacterType, string> = {
  [CharacterType.Protagonist]: '主角',
  [CharacterType.SecondProtagonist]: '二号主角',
  [CharacterType.ThirdProtagonist]: '三号主角',
  [CharacterType.Supporting]: '配角',
  [CharacterType.MinorSupporting]: '次要配角',
  [CharacterType.MajorAntagonist]: '重要敌对',
  [CharacterType.MinorAntagonist]: '次要敌对',
  [CharacterType.AntagonistFaction]: '敌对阵营',
}

/**
 * 角色类型选项
 */
export const CharacterTypeOptions: DefaultOptionType[] = [
  { value: CharacterType.Protagonist, label: CharacterTypeLabels[CharacterType.Protagonist] },
  {
    value: CharacterType.SecondProtagonist,
    label: CharacterTypeLabels[CharacterType.SecondProtagonist],
  },
  {
    value: CharacterType.ThirdProtagonist,
    label: CharacterTypeLabels[CharacterType.ThirdProtagonist],
  },
  { value: CharacterType.Supporting, label: CharacterTypeLabels[CharacterType.Supporting] },
  {
    value: CharacterType.MinorSupporting,
    label: CharacterTypeLabels[CharacterType.MinorSupporting],
  },
  {
    value: CharacterType.MajorAntagonist,
    label: CharacterTypeLabels[CharacterType.MajorAntagonist],
  },
  {
    value: CharacterType.MinorAntagonist,
    label: CharacterTypeLabels[CharacterType.MinorAntagonist],
  },
  {
    value: CharacterType.AntagonistFaction,
    label: CharacterTypeLabels[CharacterType.AntagonistFaction],
  },
]

/**
 * 角色
 */
export interface Character {
  id: string
  name: string
  gender: CharacterGender
  characterType?: CharacterType | null
  novelId: string
  background?: string
  appearance?: string
  personality?: string
  additionalInfo?: string
  createdAt: string
  updatedAt: string
}
