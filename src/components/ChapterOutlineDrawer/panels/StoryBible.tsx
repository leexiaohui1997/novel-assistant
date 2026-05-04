import { App, Form, FormInstance, Input, Spin } from 'antd'
import React, { useCallback, useEffect, useImperativeHandle, useRef, useState } from 'react'

import { ChapterOutlinePanelHandle } from './common'

import {
  ChapterOutline,
  editChapterOutline,
  getChapterOutline,
} from '@/services/chapterOutlineService'
import { getErrorMsg } from '@/utils/error'
import { logger } from '@/utils/logger'

export type StoryBibleProps = {
  novelId: string
  chapterId?: string
  ref?: React.Ref<ChapterOutlinePanelHandle>
}

export function StoryBible({ novelId, chapterId, ref }: StoryBibleProps) {
  const { message } = App.useApp()
  const formRef = useRef<FormInstance>(null)

  const [outline, setOutline] = useState<ChapterOutline>()
  const [loading, setLoading] = useState(false)

  const save = useCallback(
    async (setDoingSave: (doingSave: boolean) => void) => {
      if (!formRef.current) {
        logger.debug('formRef.current is null')
        return
      }

      try {
        const values = await formRef.current.validateFields()
        try {
          setDoingSave(true)
          // 保存本章大纲
          await editChapterOutline(novelId, chapterId, values.positioning)
          message.success('设定集保存成功')
        } catch (e) {
          message.error(`设定集保存失败: ${getErrorMsg(e)}`)
        } finally {
          setDoingSave(false)
        }
      } catch (e) {
        logger.error(`表单校验失败: ${getErrorMsg(e)}`)
      }
    },
    [message, novelId, chapterId],
  )

  useImperativeHandle(ref, () => ({ save }))

  useEffect(() => {
    // 获取本章大纲
    const fetchOutline = async () => {
      try {
        setLoading(true)
        const outline = await getChapterOutline(novelId, chapterId)
        setOutline(outline || undefined)
      } catch (e) {
        message.error(`获取大纲失败: ${getErrorMsg(e)}`)
      } finally {
        setLoading(false)
      }
    }

    fetchOutline()
  }, [novelId, chapterId, message])

  useEffect(() => {
    formRef.current?.setFieldsValue({
      positioning: outline?.positioning || '',
    })
  }, [outline])

  if (loading) {
    return (
      <div className="h-25 flex items-center justify-center">
        <Spin />
      </div>
    )
  }

  return (
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
  )
}
