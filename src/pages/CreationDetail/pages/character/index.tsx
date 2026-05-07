import { PlusOutlined } from '@ant-design/icons'
import { App, Button, Card } from 'antd'
import { useCallback, useRef, useState } from 'react'

import { CharacterCard } from './components/CharacterCard'
import { CharacterModal } from './components/CharacterModal'

import ListWithGenerics, { ListRef } from '@/components/List'
import { useCreationState } from '@/hooks/useCreationState'
import { deleteCharacter, getCharactersWithPagination } from '@/services/characterService'
import { Character } from '@/types/character'
import { getErrorMsg } from '@/utils/error'
import { logger } from '@/utils/logger'

export default function CreationDetailCharacter() {
  const { message, modal } = App.useApp()
  const { novelId } = useCreationState()
  const listRef = useRef<ListRef>(null)

  const [editingCharacter, setEditingCharacter] = useState<Character>()
  const [modalIsOpen, setModalIsOpen] = useState(false)

  const fetchList = useCallback(
    async (page: number, pageSize: number) => {
      return getCharactersWithPagination(page, pageSize, novelId)
    },
    [novelId],
  )

  const handleDelete = useCallback(
    (character: Character) => {
      modal.confirm({
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
    [message, modal],
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
              <CharacterCard
                key={itemInfo.id}
                character={itemInfo}
                order={order}
                onEdit={(char) => {
                  setEditingCharacter(char)
                  setModalIsOpen(true)
                }}
                onDelete={handleDelete}
              />
            )}
          />
        </Card>
      </div>

      <CharacterModal
        open={modalIsOpen}
        character={editingCharacter}
        novelId={novelId}
        onClose={() => setModalIsOpen(false)}
        onSuccess={() => void listRef.current?.refresh()}
        afterClose={() => setEditingCharacter(undefined)}
      />
    </>
  )
}
