import { HistoryOutlined, LeftOutlined } from '@ant-design/icons'
import { Button, Input, Modal, Tooltip } from 'antd'
import { useCallback, useMemo, useState } from 'react'

import { EditorContext } from './EditorContext'

import type { EditorContextType, EditorOpenOptions } from './EditorContext'

import Editor from '@/components/Editor'

/** 编辑器 Provider 属性 */
interface EditorProviderProps {
  children: React.ReactNode
}

/** 全屏编辑器弹窗 */
const EditorModal: React.FC<EditorOpenOptions & { onClose: () => void }> = ({ novel, onClose }) => {
  const title = (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-3">
        <Button color="default" variant="filled" icon={<LeftOutlined />} onClick={onClose} />
        <span className="text-base font-medium">{novel.title}</span>
      </div>
      <div className="flex items-center gap-2">
        <Tooltip title="历史记录" placement="bottom" mouseEnterDelay={1} color="white">
          <Button type="text" icon={<HistoryOutlined />} />
        </Tooltip>
        <Button type="primary" shape="round">
          下一步
        </Button>
      </div>
    </div>
  )

  return (
    <Modal
      open
      closable={false}
      footer={null}
      title={title}
      className="w-full! h-full! max-w-full!"
      classNames={{
        container: 'rounded-none! h-full max-h-full!',
        body: 'bg-gray-100 pt-4!',
      }}
    >
      <div className="flex-1 w-full h-0 max-w-240 mx-auto! overflow-auto p-0!">
        <div className="min-h-full p-16! pb-21! bg-white rounded-xl flex flex-col">
          {/* 头部 */}
          <div className="flex items-center mb-8! gap-6">
            {/* 标题 */}
            <Input
              variant="borderless"
              placeholder="请输入标题"
              size="large"
              maxLength={30}
              showCount
              className="show-count-on-focus"
              classNames={{
                count: 'text-sm',
                input: 'text-xl!',
              }}
            />
          </div>

          {/* 正文 */}
          <Editor className="flex-1" placeholder="请输入正文" />
        </div>
      </div>
    </Modal>
  )
}

/** 编辑器 Provider 组件 */
export const EditorProvider: React.FC<EditorProviderProps> = ({ children }) => {
  const [options, setOptions] = useState<EditorOpenOptions | null>(null)

  const open = useCallback((opts: EditorOpenOptions) => {
    setOptions(opts)
  }, [])

  const close = useCallback(() => {
    setOptions(null)
  }, [])

  const contextValue: EditorContextType = useMemo(() => ({ open, close }), [open, close])

  return (
    <EditorContext.Provider value={contextValue}>
      {children}
      {options && <EditorModal {...options} onClose={close} />}
    </EditorContext.Provider>
  )
}
