import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import { App, Button, Card, Form, FormInstance, Input, Modal, Select, Tag } from 'antd'
import { camelCase } from 'lodash-es'
import { useCallback, useMemo, useRef, useState } from 'react'

import ListWithGenerics, { ListRef } from '@/components/List'
import { WithAiAction } from '@/components/WithAiAction'
import { useCreationState } from '@/hooks/useCreationState'
import {
  createCharacter,
  deleteCharacter,
  getCharactersWithPagination,
  updateCharacter,
} from '@/services/characterService'
import {
  Character,
  CharacterGender,
  CharacterGenderLabels,
  CharacterGenderOptions,
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

export default function CreationDetailCharacter() {
  const { message } = App.useApp()
  const { novelId } = useCreationState()
  const formRef = useRef<FormInstance>(null)
  const listRef = useRef<ListRef>(null)

  const [editingCharacter, setEditingCharacter] = useState<Character>()
  const [modalIsOpen, setModalIsOpen] = useState(false)
  const [modalLoading, setModalLoading] = useState(false)

  const modalIsEdit = useMemo(() => !!editingCharacter, [editingCharacter])
  const modalOkLabel = useMemo(() => (modalIsEdit ? '更新' : '创建'), [modalIsEdit])

  /**
   * 处理 AI 生成角色的结果
   */
  const handleAiGenerateResult = useCallback(
    (result: GeneratedCharacter) => {
      logger.debug('AI 生成角色结果:', result)

      // 将 AI 返回的数据填充到表单中
      formRef.current?.setFieldsValue({
        name: result.name,
        gender: result.gender,
        background: result.background,
        appearance: result.appearance,
        personality: result.personality,
        additionalInfo: result.additional_info,
      })

      // 打开模态框
      setModalIsOpen(true)
      message.success('AI 已生成角色建议，请确认后创建')
    },
    [message],
  )

  /**
   * 处理 AI 优化的结果
   */
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

  const modalTitle = useMemo(
    () =>
      modalIsEdit ? (
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
    [modalIsEdit, handleAiGenerateResult, novelId],
  )

  const fetchList = useCallback(
    async (page: number, pageSize: number) => {
      return getCharactersWithPagination(page, pageSize, novelId)
    },
    [novelId],
  )

  const modalInitialValues = useMemo(
    () =>
      editingCharacter
        ? {
            name: editingCharacter.name,
            gender: editingCharacter.gender,
            background: editingCharacter.background,
            appearance: editingCharacter.appearance,
            personality: editingCharacter.personality,
            additionalInfo: editingCharacter.additionalInfo,
          }
        : {},
    [editingCharacter],
  )

  const modalAfterClose = useCallback(() => {
    setEditingCharacter(undefined)
  }, [])

  const modalOkhandle = useCallback(async () => {
    try {
      const values = await formRef.current?.validateFields()
      logger.debug('表单验证成功:', values)
      try {
        setModalLoading(true)
        if (modalIsEdit && editingCharacter) {
          // 更新角色
          await updateCharacter({
            id: editingCharacter.id,
            name: values.name,
            gender: values.gender,
            background: values.background,
            appearance: values.appearance,
            personality: values.personality,
            additionalInfo: values.additionalInfo,
          })
        } else {
          // 创建角色
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
        message.success(`${modalOkLabel}角色成功`)
        setModalIsOpen(false)
        void listRef.current?.refresh()
      } catch (error) {
        message.error(`${modalOkLabel}角色失败: ${getErrorMsg(error)}`)
      } finally {
        setModalLoading(false)
      }
    } catch (error) {
      logger.error('表单验证失败:', error)
    }
  }, [message, modalOkLabel, modalIsEdit, editingCharacter, novelId])

  const handleEdit = useCallback(
    (character: Character) => {
      setEditingCharacter(character)
      setModalIsOpen(true)
    },
    [setEditingCharacter, setModalIsOpen],
  )

  const handleDelete = useCallback(
    (character: Character) => {
      Modal.confirm({
        title: '确认删除',
        content: `确定要删除角色「${character.name}」吗？此操作不可恢复。`,
        okText: '删除',
        okType: 'danger',
        cancelText: '取消',
        onOk: async () => {
          try {
            await deleteCharacter(character.id)
            message.success('删除角色成功')
            void listRef.current?.refresh()
          } catch (error) {
            logger.error('删除角色失败:', error)
            message.error(`删除角色失败: ${getErrorMsg(error)}`)
          }
        },
      })
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

  return (
    <>
      <div className="p-6">
        <Card
          title="角色管理"
          classNames={{ body: 'p-3!' }}
          extra={
            <Button
              type="primary"
              shape="round"
              icon={<PlusOutlined />}
              onClick={() => setModalIsOpen(true)}
            >
              创建角色
            </Button>
          }
        >
          <ListWithGenerics
            ref={listRef}
            classNames={{
              list: 'flex flex-wrap relative',
              item: 'w-1/3 p-3',
            }}
            emptyDescription="暂无角色"
            fetchList={fetchList}
            renderItem={(itemInfo, _, order) => (
              <Card
                size="small"
                title={
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-gray-400"># {order}</span>
                      <span>{itemInfo.name}</span>
                    </div>

                    <Tag>性别：{CharacterGenderLabels[itemInfo.gender]}</Tag>
                  </div>
                }
                actions={[
                  <Button
                    size="small"
                    variant="text"
                    color="primary"
                    icon={<EditOutlined />}
                    onClick={() => handleEdit(itemInfo)}
                  >
                    编辑
                  </Button>,
                  <Button
                    size="small"
                    variant="text"
                    color="danger"
                    icon={<DeleteOutlined />}
                    onClick={() => handleDelete(itemInfo)}
                  >
                    删除
                  </Button>,
                ]}
              >
                <div className="h-8 flex items-center">
                  <div className="line-clamp-2 leading-4 text-sm">
                    {itemInfo.appearance ||
                      itemInfo.background ||
                      itemInfo.personality ||
                      itemInfo.additionalInfo ||
                      '暂无描述'}
                  </div>
                </div>
              </Card>
            )}
          />
        </Card>
      </div>

      <Modal
        title={modalTitle}
        open={modalIsOpen}
        width={600}
        okText={modalOkLabel}
        onOk={modalOkhandle}
        onCancel={() => setModalIsOpen(false)}
        afterClose={modalAfterClose}
        okButtonProps={{ loading: modalLoading }}
        destroyOnHidden
      >
        <Form ref={formRef} labelCol={{ flex: '90px' }} initialValues={modalInitialValues}>
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
    </>
  )
}
