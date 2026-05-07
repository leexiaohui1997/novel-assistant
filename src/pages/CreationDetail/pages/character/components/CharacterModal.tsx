import { App, Form, FormInstance, Input, Modal, Select } from 'antd'
import { camelCase } from 'lodash-es'
import { useCallback, useMemo, useRef, useState } from 'react'

import { WithAiAction } from '@/components/WithAiAction'
import { createCharacter, updateCharacter } from '@/services/characterService'
import {
  Character,
  CharacterGender,
  CharacterGenderOptions,
  CharacterTypeOptions,
} from '@/types/character'
import { getErrorMsg } from '@/utils/error'
import { logger } from '@/utils/logger'

/**
 * AI 生成的角色数据结构
 */
interface GeneratedCharacter {
  name: string
  gender: CharacterGender
  background: string
  appearance?: string
  personality?: string
  additional_info?: string
}

/**
 * AI 优化结果数据结构（只包含优化的字段）
 */
interface OptimizedCharacter {
  name?: string
  gender?: CharacterGender
  background?: string
  appearance?: string
  personality?: string
  additional_info?: string
}

/**
 * CharacterModal 组件的 Props
 */
interface CharacterModalProps {
  /** 弹窗是否可见 */
  open: boolean
  /** 待编辑的角色数据；不传则为创建模式 */
  character?: Character
  /** 所属小说 ID */
  novelId: string
  /** 弹窗关闭回调（点击取消 / 提交成功后） */
  onClose?: () => void
  /** 创建 / 更新成功后的回调，通常用于外层刷新列表 */
  onSuccess?: () => void
  /** 弹窗完全关闭（动画结束）后的回调 */
  afterClose?: () => void
}

/**
 * 角色创建 / 编辑弹窗
 *
 * 功能：
 * 1. 创建模式：支持通过 AI 一键生成整份角色资料
 * 2. 编辑模式：加载已有角色数据进行修改
 * 3. 各字段支持 AI 单独优化（名称、性别、背景、外貌、性格、其它描述）
 */
export function CharacterModal({
  open,
  character,
  novelId,
  onClose,
  onSuccess,
  afterClose,
}: CharacterModalProps) {
  const { message } = App.useApp()
  const formRef = useRef<FormInstance>(null)
  const [loading, setLoading] = useState(false)

  /** AI 一键生成角色资料的结果回调，将结果回填到表单 */
  const handleAiGenerateResult = useCallback(
    (result: GeneratedCharacter) => {
      logger.debug('AI 生成角色结果:', result)
      formRef.current?.setFieldsValue({
        name: result.name,
        gender: result.gender,
        background: result.background,
        appearance: result.appearance,
        personality: result.personality,
        additionalInfo: result.additional_info,
      })
      message.success('AI 已生成角色建议，请确认后创建')
    },
    [message],
  )

  /** 是否为编辑模式 */
  const isEdit = useMemo(() => !!character, [character])

  /** 弹窗标题：编辑模式为纯文本，创建模式额外挂载 AI 生成入口 */
  const modalTitle = useMemo(
    () =>
      isEdit ? (
        '编辑角色'
      ) : (
        <WithAiAction
          tip="AI 创建角色"
          placement="rightTop"
          showFeedback
          classNames={{
            root: 'items-center!',
            left: '',
          }}
          aiAction={{
            actionName: 'generate_character',
            getParams: () => ({
              novel_id: novelId,
            }),
          }}
          onResult={handleAiGenerateResult}
        >
          <span>创建角色</span>
        </WithAiAction>
      ),
    [isEdit, novelId, handleAiGenerateResult],
  )

  /** 表单初始值：编辑模式取自 character，创建模式为空 */
  const initialValues = useMemo(
    () =>
      character
        ? {
            name: character.name,
            gender: character.gender,
            characterType: character.characterType ?? undefined,
            background: character.background,
            appearance: character.appearance,
            personality: character.personality,
            additionalInfo: character.additionalInfo,
          }
        : {},
    [character],
  )

  /** AI 单字段优化结果回调：若 AI 返回了对应字段则回填，否则提示无需优化 */
  const handleOptimizeResult = useCallback(
    (result: OptimizedCharacter, field: keyof OptimizedCharacter, fieldLabel: string) => {
      logger.debug(`AI 优化${fieldLabel}结果:`, result)
      if (result[field]) {
        formRef.current?.setFieldValue(camelCase(field), result[field])
        message.success(`已优化${fieldLabel}`)
      } else {
        message.info(`AI 认为${fieldLabel}无需优化`)
      }
    },
    [message],
  )

  /**
   * 为任意表单控件包裹一层 AI 优化能力
   * @param children 原始表单控件
   * @param props 提示文案、字段名、字段中文标签
   */
  const withOptimizeField = useCallback(
    (
      children: React.ReactNode,
      props: {
        tip: string
        field: keyof OptimizedCharacter
        fieldLabel: string
      },
    ) => (
      <WithAiAction
        tip={props.tip}
        showFeedback
        aiAction={{
          actionName: 'optimize_character',
          getParams: () => {
            const currentValues = formRef.current?.getFieldsValue()
            return {
              novel_id: novelId,
              character: {
                name: currentValues?.name || '',
                gender: currentValues?.gender || 'unknown',
                background: currentValues?.background,
                appearance: currentValues?.appearance,
                personality: currentValues?.personality,
                additional_info: currentValues?.additionalInfo,
              },
              optimize_fields: [props.field],
            }
          },
        }}
        onResult={(result: OptimizedCharacter) =>
          handleOptimizeResult(result, props.field, props.fieldLabel)
        }
      >
        {children}
      </WithAiAction>
    ),
    [novelId, handleOptimizeResult],
  )

  /** 将表单 values 转换为服务端接口所需的 payload */
  const buildPayload = useCallback(
    (values: Record<string, unknown>) => ({
      name: values.name as string,
      gender: values.gender as string,
      characterType: (values.characterType as Character['characterType']) ?? null,
      background: values.background as string | undefined,
      appearance: values.appearance as string | undefined,
      personality: values.personality as string | undefined,
      additionalInfo: values.additionalInfo as string | undefined,
    }),
    [],
  )

  /** 根据当前模式分发到更新或创建接口 */
  const submitCharacter = useCallback(
    async (values: Record<string, unknown>) => {
      const payload = buildPayload(values)
      if (isEdit && character) {
        await updateCharacter({ id: character.id, ...payload })
        return
      }
      await createCharacter({ novelId, ...payload })
    },
    [isEdit, character, novelId, buildPayload],
  )

  /** 触发表单校验，失败返回 null，避免调用方用 try/catch 包裹 */
  const validateForm = useCallback(async () => {
    try {
      return await formRef.current?.validateFields()
    } catch (error) {
      logger.error('表单验证失败:', error)
      return null
    }
  }, [])

  /** 点击「确定」的主流程：校验 → 提交 → 反馈 */
  const handleOk = useCallback(async () => {
    const values = await validateForm()
    if (!values) return

    const actionText = isEdit ? '更新' : '创建'
    setLoading(true)
    try {
      await submitCharacter(values)
      message.success(`${actionText}角色成功`)
      onClose?.()
      onSuccess?.()
    } catch (error) {
      message.error(`${actionText}角色失败: ${getErrorMsg(error)}`)
    } finally {
      setLoading(false)
    }
  }, [validateForm, submitCharacter, isEdit, message, onClose, onSuccess])

  return (
    <Modal
      title={modalTitle}
      open={open}
      width={600}
      okText={isEdit ? '更新' : '创建'}
      onOk={handleOk}
      onCancel={onClose}
      okButtonProps={{ loading }}
      afterClose={afterClose}
      destroyOnHidden
    >
      <Form ref={formRef} labelCol={{ flex: '90px' }} initialValues={initialValues}>
        <Form.Item
          label="角色名称"
          name="name"
          rules={[{ required: true, message: '请输入角色名称' }]}
        >
          {withOptimizeField(<Input placeholder="请输入角色名称" />, {
            tip: 'AI 优化角色名称',
            field: 'name',
            fieldLabel: '角色名称',
          })}
        </Form.Item>

        <Form.Item
          label="角色性别"
          name="gender"
          rules={[{ required: true, message: '请选择角色性别' }]}
        >
          {withOptimizeField(
            <Select placeholder="请选择角色性别" options={CharacterGenderOptions} />,
            {
              tip: 'AI 优化角色性别',
              field: 'gender',
              fieldLabel: '角色性别',
            },
          )}
        </Form.Item>

        <Form.Item label="角色类型" name="characterType">
          <Select placeholder="请选择角色类型（可选）" options={CharacterTypeOptions} allowClear />
        </Form.Item>

        <Form.Item label="角色背景" name="background">
          {withOptimizeField(<Input.TextArea placeholder="请输入角色背景" rows={3} />, {
            tip: 'AI 优化角色背景',
            field: 'background',
            fieldLabel: '角色背景',
          })}
        </Form.Item>

        <Form.Item label="外貌描写" name="appearance">
          {withOptimizeField(<Input.TextArea placeholder="请输入外貌描写" rows={3} />, {
            tip: 'AI 优化外貌描写',
            field: 'appearance',
            fieldLabel: '外貌描写',
          })}
        </Form.Item>

        <Form.Item label="性格特征" name="personality">
          {withOptimizeField(<Input.TextArea placeholder="请输入性格特征" rows={3} />, {
            tip: 'AI 优化性格特征',
            field: 'personality',
            fieldLabel: '性格特征',
          })}
        </Form.Item>

        <Form.Item label="其它描述" name="additionalInfo">
          {withOptimizeField(<Input.TextArea placeholder="请输入其它描述" rows={3} />, {
            tip: 'AI 优化其它描述',
            field: 'additional_info',
            fieldLabel: '其它描述',
          })}
        </Form.Item>
      </Form>
    </Modal>
  )
}
