import { LinkOutlined, Loading3QuartersOutlined } from '@ant-design/icons'
import { Spin, Tooltip, Typography } from 'antd'
import { useCallback } from 'react'

import { useCreationState } from '@/hooks/useCreationState'
import { useEditor } from '@/hooks/useEditor'
import { useLoading } from '@/hooks/useLoading'
import { useRefresh } from '@/hooks/useRefresh'
import { getChapterById } from '@/services/chapterService'
import { getChapterRelationsByTerm } from '@/services/termsService'
import { numToCn } from '@/utils/number'

export type TermChapterDescProps = {
  termId: string
  refresh?: () => unknown
}

export default function TermChapterDesc({ termId, refresh }: TermChapterDescProps) {
  const editor = useEditor()
  const { novelInfo } = useCreationState()

  const refreshFn = useCallback(() => getChapterRelationsByTerm({ termId }), [termId])

  const { data, loading } = useRefresh({
    refreshFn,
    initialData: [],
  })

  const { execute, loading: doingExecute } = useLoading(getChapterById)

  const handleClick = useCallback(
    async (chapterId: string, volumeSequence: number) => {
      const chapter = await execute(chapterId)
      editor.open({
        novel: novelInfo,
        chapter,
        sequence: volumeSequence,
        onSubmit: () => {
          refresh?.()
        },
      })
    },
    [execute, novelInfo, editor, refresh],
  )

  if (loading) {
    return <Spin size="small" />
  }

  return (
    <div className="text-sm">
      {data.map((item) => (
        <span key={item.id}>
          <span>{item.description}</span>
          <Tooltip
            title={`第${numToCn(item.volumeSequence)}卷第${item.chapterSequence}章 ${item.chapterTitle}`}
          >
            <Typography.Link
              className="mx-1"
              onClick={() => handleClick(item.chapterId, item.volumeSequence)}
            >
              {doingExecute ? <Loading3QuartersOutlined spin /> : <LinkOutlined />}
            </Typography.Link>
          </Tooltip>
          {!item.description.endsWith('。') && <span>。</span>}
        </span>
      ))}
    </div>
  )
}
