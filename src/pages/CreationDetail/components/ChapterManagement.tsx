import { Button, Select, Space, message } from 'antd'
import { useCallback, useEffect, useMemo, useState } from 'react'

import ChapterTable from './ChapterTable'

import { useCreationState } from '@/hooks/useCreationState'
import { useEditor } from '@/hooks/useEditor'
import { useEditVolumeModal } from '@/hooks/useEditVolumeModal'
import {
  DEFAULT_VOLUME_SEQUENCE,
  type Volume,
  getVolumes,
  resolveVolumesWithDefault,
} from '@/services/chapterService'
import { logger } from '@/utils/logger'
import { numToCn } from '@/utils/number'

const ChapterManagement: React.FC = () => {
  const { novelId, novelInfo } = useCreationState()
  const editor = useEditor()
  const [messageApi, contextHolder] = message.useMessage()

  const [volumes, setVolumes] = useState<Volume[]>([])
  const [selectedSequence, setSelectedSequence] = useState<number>(DEFAULT_VOLUME_SEQUENCE)

  /** 保存成功后：用最新列表覆盖本地；若原选中 sequence 已被删除则回落到首卷 */
  const handleSaved = useCallback(
    (next: Volume[]) => {
      const resolved = resolveVolumesWithDefault(next, novelId)
      setVolumes(resolved)
      setSelectedSequence((prev) =>
        resolved.some((v) => v.sequence === prev) ? prev : resolved[0].sequence,
      )
    },
    [novelId],
  )

  const [editVolumeContext, editVolumeActions] = useEditVolumeModal({
    novelId,
    onSaved: handleSaved,
  })

  useEffect(() => {
    const load = async () => {
      try {
        const list = await getVolumes(novelId)
        const resolved = resolveVolumesWithDefault(list, novelId)
        setVolumes(resolved)
        // 默认选中第一个分卷的 sequence
        setSelectedSequence(resolved[0].sequence)
      } catch (e) {
        logger.error('加载分卷列表失败:', e)
        messageApi.error('加载分卷列表失败')
      }
    }
    load()
  }, [novelId, messageApi])

  const options = useMemo(
    () =>
      volumes.map((v) => ({
        value: v.sequence,
        label: `第${numToCn(v.sequence)}卷：${v.name}`,
      })),
    [volumes],
  )

  /** 新建章节 */
  const handleCreateChapter = useCallback(() => {
    editor.open({ novel: novelInfo, sequence: selectedSequence })
  }, [editor, novelInfo, selectedSequence])

  return (
    <>
      {contextHolder}
      {editVolumeContext}
      <div className="flex items-center justify-between mb-4!">
        <Select
          className="min-w-60"
          value={selectedSequence}
          options={options}
          onChange={setSelectedSequence}
        />
        <Space>
          <Button onClick={editVolumeActions.open}>编辑分卷</Button>
          <Button type="primary" onClick={handleCreateChapter}>
            新建章节
          </Button>
        </Space>
      </div>
      <ChapterTable volumeSequence={selectedSequence} />
    </>
  )
}

export default ChapterManagement
