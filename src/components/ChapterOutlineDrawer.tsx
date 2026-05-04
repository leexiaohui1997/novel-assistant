import { ReadOutlined } from '@ant-design/icons'
import { App, Button, Drawer, Form, FormInstance, Input, Tooltip } from 'antd'
import { useCallback, useEffect, useImperativeHandle, useRef, useState } from 'react'

import {
  ChapterOutline,
  editChapterOutline,
  getChapterOutline,
} from '@/services/chapterOutlineService'
import { Chapter } from '@/services/chapterService'
import { Novel } from '@/services/novelService'
import { getErrorMsg } from '@/utils/error'
import { logger } from '@/utils/logger'

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
  const { message } = App.useApp()
  const formRef = useRef<FormInstance>(null)

  const [visible, setVisible] = useState(false)
  const [doingSave, setDoingSave] = useState(false)
  const [loading, setLoading] = useState(false)
  const [outline, setOutline] = useState<ChapterOutline>()

  const open = useCallback(() => setVisible(true), [])
  const close = useCallback(() => setVisible(false), [])

  const handleSave = useCallback(async () => {
    try {
      const values = await formRef.current?.validateFields()
      logger.debug('表单校验成功', values)
      try {
        setDoingSave(true)
        // 保存本章大纲
        await editChapterOutline(novel.id, chapter?.id, values.positioning)
        message.success('保存成功')
      } catch (e) {
        message.error(`保存失败: ${getErrorMsg(e)}`)
      } finally {
        setDoingSave(false)
      }
    } catch (e) {
      logger.error(`表单校验失败: ${getErrorMsg(e)}`)
    }
  }, [message, novel, chapter])

  useImperativeHandle(ref, () => ({
    open,
    close,
  }))

  useEffect(() => {
    // 获取本章大纲
    if (!visible) return

    const fetchOutline = async () => {
      try {
        setLoading(true)
        const outline = await getChapterOutline(novel.id, chapter?.id)
        setOutline(outline || undefined)
      } catch (e) {
        logger.error(`获取大纲失败: ${getErrorMsg(e)}`)
        message.error(`获取大纲失败: ${getErrorMsg(e)}`)
      } finally {
        setLoading(false)
      }
    }

    fetchOutline()
  }, [visible, novel.id, chapter?.id, message])

  useEffect(() => {
    formRef.current?.setFieldsValue({
      positioning: outline?.positioning || '',
    })
  }, [outline])

  return (
    <Drawer
      title="本章大纲"
      open={visible}
      size={600}
      classNames={{ body: 'p-0!' }}
      onClose={close}
      destroyOnHidden
      loading={loading}
      extra={
        <Button type="primary" shape="round" loading={doingSave} onClick={handleSave}>
          保存
        </Button>
      }
    >
      <div className="p-6">
        <Form
          ref={formRef}
          layout="vertical"
          initialValues={{
            positioning: outline?.positioning || '',
          }}
        >
          <Form.Item label="本章定位" name="positioning">
            <Input.TextArea placeholder="请输入本章定位" rows={4} maxLength={200} showCount />
          </Form.Item>
        </Form>
      </div>
    </Drawer>
  )
}

export function ChapterOutlineTrigger(props: Omit<ChapterOutlineDrawerProps, 'ref'>) {
  const drawerRef = useRef<ChapterOutlineDrawerHandle>(null)

  return (
    <>
      <Tooltip title="本章大纲" placement="bottom" mouseEnterDelay={1} color="white">
        <Button type="text" icon={<ReadOutlined />} onClick={() => drawerRef.current?.open()} />
      </Tooltip>
      <ChapterOutlineDrawer ref={drawerRef} {...props} />
    </>
  )
}
