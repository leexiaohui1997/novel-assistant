import { PlusOutlined } from '@ant-design/icons'
import { App, Button, Card, Form, FormInstance, Input, Modal, Select } from 'antd'
import { useCallback, useMemo, useRef, useState } from 'react'

import { useCreationState } from '@/hooks/useCreationState'
import { createCharacter, getCharactersByNovel, updateCharacter } from '@/services/characterService'
import { Character, CharacterGenderOptions } from '@/types/character'
import { getErrorMsg } from '@/utils/error'
import { logger } from '@/utils/logger'

export default function CreationDetailCharacter() {
  const { message } = App.useApp()
  const { novelId } = useCreationState()
  const formRef = useRef<FormInstance>(null)

  const [, setCharacters] = useState<Character[]>([])
  const [editingCharacter, setEditingCharacter] = useState<Character>()
  const [modalIsOpen, setModalIsOpen] = useState(false)
  const [modalLoading, setModalLoading] = useState(false)

  const modalIsEdit = useMemo(() => !!editingCharacter, [editingCharacter])
  const modalTitle = useMemo(() => (modalIsEdit ? '编辑角色' : '创建角色'), [modalIsEdit])
  const modalOkLabel = useMemo(() => (modalIsEdit ? '更新' : '创建'), [modalIsEdit])

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

  const refreshList = useCallback(async () => {
    try {
      const list = await getCharactersByNovel(novelId)
      setCharacters(list)
    } catch (error) {
      logger.error('获取角色列表失败:', error)
      message.error('获取角色列表失败')
    }
  }, [novelId, message])

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
        void refreshList()
      } catch (error) {
        message.error(`${modalOkLabel}角色失败: ${getErrorMsg(error)}`)
      } finally {
        setModalLoading(false)
      }
    } catch (error) {
      logger.error('表单验证失败:', error)
    }
  }, [message, modalOkLabel, modalIsEdit, editingCharacter, novelId, refreshList])

  return (
    <>
      <div className="p-6">
        <Card
          title="角色管理"
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
        ></Card>
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
