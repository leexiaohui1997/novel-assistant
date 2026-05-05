import { App, Form, FormInstance, Input, Modal, Select } from 'antd'
import { camelCase } from 'lodash-es'
import { useCallback, useMemo, useRef, useState } from 'react'

import { WithAiAction } from '@/components/WithAiAction'
import { createCharacter, updateCharacter } from '@/services/characterService'
import { Character, CharacterGender, CharacterGenderOptions } from '@/types/character'
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

interface CharacterModalProps {
  open: boolean
  character?: Character
  novelId: string
  onClose?: () => void
  onSuccess?: () => void
  afterClose?: () => void
}

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

  const isEdit = useMemo(() => !!character, [character])
  const modalTitle = useMemo(
    () =>
      isEdit ? (
        '编辑角色'
      ) : (
        <WithAiAction
          tip="AI 创建角色"
          placement="rightTop"
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

  const initialValues = useMemo(
    () =>
      character
        ? {
            name: character.name,
            gender: character.gender,
            background: character.background,
            appearance: character.appearance,
            personality: character.personality,
            additionalInfo: character.additionalInfo,
          }
        : {},
    [character],
  )

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

  const handleOk = useCallback(async () => {
    try {
      const values = await formRef.current?.validateFields()
      logger.debug('表单验证成功:', values)
      try {
        setLoading(true)
        if (isEdit && character) {
          await updateCharacter({
            id: character.id,
            name: values.name,
            gender: values.gender,
            background: values.background,
            appearance: values.appearance,
            personality: values.personality,
            additionalInfo: values.additionalInfo,
          })
        } else {
          await createCharacter({
            novelId,
            name: values.name,
            gender: values.gender,
            background: values.background,
            appearance: values.appearance,
            personality: values.personality,
            additionalInfo: values.additionalInfo,
          })
        }
        message.success(`${isEdit ? '更新' : '创建'}角色成功`)
        onClose?.()
        onSuccess?.()
      } catch (error) {
        message.error(`${isEdit ? '更新' : '创建'}角色失败: ${getErrorMsg(error)}`)
      } finally {
        setLoading(false)
      }
    } catch (error) {
      logger.error('表单验证失败:', error)
    }
  }, [message, isEdit, character, novelId, onClose, onSuccess])

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
