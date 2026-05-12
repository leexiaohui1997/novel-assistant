import { MinusOutlined } from '@ant-design/icons'
import { App, Button, Space, Table, Tag, Tooltip } from 'antd'
import { useCallback, useImperativeHandle, useMemo, useState } from 'react'

import { ChapterOutlinePanelHandle } from './common'

import { WithAiAction } from '@/components/WithAiAction'
import { useRefresh } from '@/hooks/useRefresh'
import { useEditorForm } from '@/providers/EditorFormContext'
import {
  AiTerm,
  ChapterTerm,
  getChapterTerms,
  TERM_TYPE_LABELS,
  TermType,
  updateChapterTerms,
} from '@/services/termsService'
import { getErrorMsg } from '@/utils/error'
import { logger } from '@/utils/logger'

export type TermsBibleProps = {
  novelId: string
  chapterId?: string
  ref?: React.Ref<ChapterOutlinePanelHandle>
}

/** AI 提取名词 Action 的返回结果结构 */
type ExtractTermsResult = {
  terms: AiTerm[]
}

/**
 * 校验 AI 返回结果是否符合 ExtractTermsResult 结构。
 * 仅校验顶层 terms 字段是否为数组，详细字段交由消费方按需处理。
 */
function isValidExtractResult(raw: unknown): raw is ExtractTermsResult {
  if (!raw || typeof raw !== 'object') return false
  const terms = (raw as { terms?: unknown }).terms
  return Array.isArray(terms)
}

export default function TermsBible({ novelId, chapterId, ref }: TermsBibleProps) {
  const { message } = App.useApp()
  const { content } = useEditorForm()
  const [aiTerms, setAiTerms] = useState<AiTerm[]>([])

  const refreshFn = useCallback(() => {
    return getChapterTerms({ novelId, chapterId })
  }, [novelId, chapterId])

  const { data, loading, refresh, setData } = useRefresh<ChapterTerm[]>({
    refreshFn,
    actionName: '加载章节名词',
    initialData: [],
  })

  const tableData = useMemo(() => {
    return [
      ...aiTerms,
      ...data
        .filter((item) => !aiTerms.some((t) => t.id === item.term.id))
        .map<AiTerm>((item) => ({
          id: item.term.id,
          name: item.term.name,
          term_type: item.term.termType,
          description: item.description,
        })),
    ].map((item, index) => ({
      ...item,
      index,
    }))
  }, [aiTerms, data])

  const handleRemove = useCallback(
    (index: number) => {
      const item = tableData[index]
      if (!item) return

      const aiIdx = aiTerms.findIndex((t) => t.id === item.id && t.name === item.name)
      if (aiIdx !== -1) {
        setAiTerms(aiTerms.filter((_, i) => i !== aiIdx))
        return
      }

      const dataIdx = data.findIndex((rel) => rel.term.id === item.id)
      if (dataIdx !== -1) {
        setData(data.filter((_, i) => i !== dataIdx))
      }
    },
    [tableData, data, aiTerms, setData],
  )

  const handleAiResult = useCallback(
    (result: ExtractTermsResult) => {
      if (!isValidExtractResult(result)) {
        logger.error('extract_terms 返回数据格式异常:', result)
        message.error('AI 返回数据格式异常')
        return
      }
      if (result.terms.length === 0) {
        setAiTerms([])
        message.info('未提取到任何名词')
        return
      }
      setAiTerms(result.terms)
    },
    [message],
  )

  const save = useCallback(
    async (setDoingSave: (loading: boolean) => unknown) => {
      // 剔除内部 index 字段，构造提交给后端的 terms
      const terms: AiTerm[] = tableData.map(({ index: _index, ...rest }) => rest)

      try {
        setDoingSave(true)
        await updateChapterTerms({ novelId, chapterId, terms })
        await refresh()
        setAiTerms([])
        message.success('本章名词保存成功')
      } catch (e) {
        logger.error('保存本章名词失败', e)
        message.error(`本章名词保存失败: ${getErrorMsg(e)}`)
      } finally {
        setDoingSave(false)
      }
    },
    [tableData, novelId, chapterId, refresh, message],
  )

  useImperativeHandle(ref, () => ({
    save,
  }))

  return (
    <>
      <WithAiAction<ExtractTermsResult>
        tip="AI 提取名词"
        placement="rightTop"
        triggerSize="small"
        aiAction={{
          actionName: 'extract_terms',
          getParams: () => {
            if (!content || content.trim() === '') {
              throw new Error('正文为空，无法提取名词')
            }
            return { novel_id: novelId, content }
          },
        }}
        onResult={handleAiResult}
        classNames={{
          root: 'items-center',
          left: '',
        }}
      >
        <div className="mb-2">本章名词</div>
      </WithAiAction>
      <Table
        size="small"
        rowKey="index"
        loading={loading}
        expandable={{
          expandedRowRender: (record) => (
            <div className="text-xs">
              <span className="font-bold">本章描述：</span>
              <span>{record.description}</span>
            </div>
          ),
        }}
        columns={[
          {
            title: '名词',
            dataIndex: 'name',
            minWidth: 100,
            render: (_, record) => (
              <Space>
                <span>{record.name}</span>
                {!record.id && <Tag color="green">新</Tag>}
              </Space>
            ),
          },
          {
            title: '类型',
            dataIndex: 'term_type',
            minWidth: 140,
            render: (v: TermType) => TERM_TYPE_LABELS[v],
          },
          {
            title: '操作',
            dataIndex: 'action',
            width: 100,
            align: 'center',
            fixed: 'right',
            render: (_, record) => (
              <Space>
                <Tooltip title="移除">
                  <Button
                    size="small"
                    color="danger"
                    variant="text"
                    icon={<MinusOutlined />}
                    onClick={() => handleRemove(record.index)}
                  />
                </Tooltip>
              </Space>
            ),
          },
        ]}
        dataSource={tableData}
      />
    </>
  )
}
