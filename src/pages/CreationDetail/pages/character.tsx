import { DeleteOutlined, EditOutlined, IdcardOutlined, PlusOutlined } from '@ant-design/icons'
import { App, Button, Card, Form, FormInstance, Input, Modal, Select, Tag } from 'antd'
import { useCallback, useMemo, useRef, useState } from 'react'

import ListWithGenerics, { ListRef } from '@/components/List'
import { useCreationState } from '@/hooks/useCreationState'
import {
  createCharacter,
  deleteCharacter,
  getCharactersWithPagination,
  updateCharacter,
} from '@/services/characterService'
import { Character, CharacterGenderLabels, CharacterGenderOptions } from '@/types/character'
import { getErrorMsg } from '@/utils/error'
import { logger } from '@/utils/logger'

export default function CreationDetailCharacter() {
  const { message } = App.useApp()
  const { novelId } = useCreationState()
  const formRef = useRef<FormInstance>(null)
  const listRef = useRef<ListRef>(null)

  const [editingCharacter, setEditingCharacter] = useState<Character>()
  const [modalIsOpen, setModalIsOpen] = useState(false)
  const [modalLoading, setModalLoading] = useState(false)

  const modalIsEdit = useMemo(() => !!editingCharacter, [editingCharacter])
  const modalTitle = useMemo(() => (modalIsEdit ? '编辑角色' : '创建角色'), [modalIsEdit])
  const modalOkLabel = useMemo(() => (modalIsEdit ? '更新' : '创建'), [modalIsEdit])

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
            characterName: editingCharacter.name,
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
            name: values.characterName,
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
            name: values.characterName,
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
                  <Button size="small" variant="text" color="primary" icon={<IdcardOutlined />}>
                    查看
                  </Button>,
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
                <div className="h-8 line-clamp-2 leading-4 text-sm flex items-center">
                  {itemInfo.background}
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
            name="characterName"
            rules={[{ required: true, message: '请输入角色名称' }]}
          >
            <Input placeholder="请输入角色名称" />
          </Form.Item>

          <Form.Item
            label="角色性别"
            name="gender"
            rules={[{ required: true, message: '请选择角色性别' }]}
          >
            <Select placeholder="请选择角色性别" options={CharacterGenderOptions} />
          </Form.Item>

          <Form.Item label="角色背景" name="background">
            <Input.TextArea placeholder="请输入角色背景" rows={3} />
          </Form.Item>

          <Form.Item label="外貌描写" name="appearance">
            <Input.TextArea placeholder="请输入外貌描写" rows={3} />
          </Form.Item>

          <Form.Item label="性格特征" name="personality">
            <Input.TextArea placeholder="请输入性格特征" rows={3} />
          </Form.Item>

          <Form.Item label="其它描述" name="additionalInfo">
            <Input.TextArea placeholder="请输入其它描述" rows={3} />
          </Form.Item>
        </Form>
      </Modal>
    </>
  )
}
