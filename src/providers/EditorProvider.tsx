import { HistoryOutlined, LeftOutlined } from '@ant-design/icons'
import { Button, Input, Modal, Select, Tooltip } from 'antd'
import { useCallback, useEffect, useMemo, useState } from 'react'

import { EditorContext } from './EditorContext'

import type { EditorContextType, EditorOpenOptions } from './EditorContext'

import Editor from '@/components/Editor'
import {
  DEFAULT_VOLUME_SEQUENCE,
  getVolumes,
  resolveVolumesWithDefault,
  type Volume,
} from '@/services/chapterService'
import { logger } from '@/utils/logger'
import { numToCn } from '@/utils/number'

/** 编辑器 Provider 属性 */
interface EditorProviderProps {
  children: React.ReactNode
}

/**
 * 解析选择器初始选中的分卷 sequence
 *
 * 优先级：preferSequence（在列表中存在） > 列表首项 sequence > DEFAULT_VOLUME_SEQUENCE
 *
 * @param volumes - 规整后的分卷列表
 * @param preferSequence - 调用方期望的 sequence（可选）
 * @returns 最终用于选择器的 sequence
 */
const resolveInitialSequence = (volumes: Volume[], preferSequence?: number): number => {
  if (volumes.length === 0) return DEFAULT_VOLUME_SEQUENCE
  if (preferSequence && volumes.some((v) => v.sequence === preferSequence)) {
    return preferSequence
  }
  return volumes[0].sequence
}

/** 全屏编辑器弹窗 */
const EditorModal: React.FC<EditorOpenOptions & { onClose: () => void }> = ({
  novel,
  chapter,
  sequence,
  onClose,
}) => {
  const [volumes, setVolumes] = useState<Volume[]>([])
  const [selectedSequence, setSelectedSequence] = useState<number>(
    sequence ?? DEFAULT_VOLUME_SEQUENCE,
  )

  // 编辑态：禁用选择器（存在 chapter 则视为编辑已有章节）
  const isEdit = Boolean(chapter)

  // 加载分卷列表
  useEffect(() => {
    let cancelled = false
    const load = async () => {
      try {
        const list = await getVolumes(novel.id)
        if (cancelled) return
        const resolved = resolveVolumesWithDefault(list, novel.id)
        setVolumes(resolved)
        setSelectedSequence(resolveInitialSequence(resolved, sequence))
      } catch (e) {
        logger.error('编辑器加载分卷列表失败:', e)
      }
    }
    load()
    return () => {
      cancelled = true
    }
  }, [novel.id, sequence])

  const volumeOptions = useMemo(
    () =>
      volumes.map((v) => ({
        value: v.sequence,
        label: `第${numToCn(v.sequence)}卷：${v.name}`,
      })),
    [volumes],
  )

  const title = (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-3">
        <Button color="default" variant="filled" icon={<LeftOutlined />} onClick={onClose} />
        <span className="text-base font-medium">{novel.title}</span>
        <Select
          className="min-w-50 font-normal"
          value={selectedSequence}
          options={volumeOptions}
          disabled={isEdit}
          onChange={setSelectedSequence}
          loading={volumes.length === 0}
        />
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
