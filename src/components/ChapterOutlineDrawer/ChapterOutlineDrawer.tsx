import { Button, Drawer, Tabs } from 'antd'
import { useCallback, useImperativeHandle, useRef, useState } from 'react'

import { ChapterOutlinePanelHandle } from './panels/common'
import { StoryBible } from './panels/StoryBible'

import { Chapter } from '@/services/chapterService'
import { Novel } from '@/services/novelService'

export type ChapterOutlineDrawerHandle = {
  open: () => void
  close: () => void
}

export type ChapterOutlineDrawerProps = {
  novel: Novel
  chapter?: Chapter
  ref?: React.Ref<ChapterOutlineDrawerHandle>
}

export function ChapterOutlineDrawer({ novel, chapter, ref }: ChapterOutlineDrawerProps) {
  const panelRef = useRef<ChapterOutlinePanelHandle>(null)

  const [visible, setVisible] = useState(false)
  const [doingSave, setDoingSave] = useState(false)

  const open = useCallback(() => setVisible(true), [])
  const close = useCallback(() => setVisible(false), [])

  useImperativeHandle(ref, () => ({
    open,
    close,
  }))

  return (
    <Drawer
      title="本章大纲"
      open={visible}
      size={600}
      classNames={{ body: 'p-0!' }}
      onClose={close}
      destroyOnHidden
      extra={
        <Button
          type="primary"
          shape="round"
          loading={doingSave}
          onClick={() => panelRef.current?.save(setDoingSave)}
        >
          保存
        </Button>
      }
    >
      <Tabs
        type="card"
        items={[
          {
            key: 'story-bible',
            label: '设定集',
            children: <StoryBible novelId={novel.id} chapterId={chapter?.id} ref={panelRef} />,
          },
        ]}
        classNames={{
          root: 'h-full',
          header: 'mb-0!',
          content: 'p-4',
        }}
        destroyOnHidden
      />
    </Drawer>
  )
}
