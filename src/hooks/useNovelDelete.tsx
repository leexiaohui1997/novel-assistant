import { Modal, Input, Alert, App } from 'antd'
import { useCallback, useState } from 'react'

import { Novel, deleteNovel as deleteNovelApi } from '@/services/novelService'
import { getErrorMsg } from '@/utils/error'

export type UseNovelDeleteProps = {
  onSuccess?: (novel: Novel) => void
}

export function useNovelDelete({ onSuccess }: UseNovelDeleteProps = {}) {
  const { message } = App.useApp()
  const [novel, setNovel] = useState<Novel>()
  const [inputValue, setInputValue] = useState('')
  const [doingDelete, setDoingDelete] = useState(false)

  const modalOpened = !!novel
  const deleteNovel = setNovel
  const closeModal = () => {
    setNovel(undefined)
  }

  const afterClose = useCallback(() => {
    setInputValue('')
  }, [])

  const handleOk = useCallback(
    async (novel: Novel) => {
      try {
        setDoingDelete(true)
        await deleteNovelApi(novel.id)
        onSuccess?.(novel)
        message.success('删除成功')
        closeModal()
      } catch (e) {
        message.error(getErrorMsg(e))
      } finally {
        setDoingDelete(false)
      }
    },
    [message, onSuccess],
  )

  const modalContext = (
    <Modal
      title="删除作品"
      open={modalOpened}
      okButtonProps={{ danger: true, loading: doingDelete, disabled: inputValue !== novel?.title }}
      onCancel={closeModal}
      onOk={() => novel && handleOk(novel)}
      afterClose={afterClose}
    >
      {modalOpened && (
        <div className="flex flex-col gap-4">
          <Alert
            type="error"
            title="危险操作"
            description={
              <>
                <p>确定要删除作品《{novel.title}》吗？此操作不可恢复。</p>
                <p>请输入作品名称以确认删除：</p>
              </>
            }
            showIcon
          />

          <Input
            placeholder="请输入作品名称"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
          />
        </div>
      )}
    </Modal>
  )

  return { deleteNovel, modalContext }
}
