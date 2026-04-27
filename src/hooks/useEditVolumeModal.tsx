import { PlusCircleOutlined } from '@ant-design/icons'
import { Button, Modal, ModalProps, Space, message } from 'antd'
import React, { useCallback, useMemo, useState } from 'react'

import EditVolumeItem from '@/components/EditVolumeItem'
import {
  batchUpdateVolumes,
  getChaptersWithPagination,
  getVolumes,
  resolveVolumesWithDefault,
  SimpleVolume,
  Volume,
  VolumeUpsert,
} from '@/services/chapterService'
import { logger } from '@/utils/logger'

/** 编辑分卷弹窗操作方法 */
export interface EditVolumeModalActions {
  /** 打开弹窗（内部会自动拉取最新分卷数据） */
  open: () => void
  /** 关闭弹窗 */
  close: () => void
}

/** 编辑分卷弹窗 Hook 返回值 */
export type UseEditVolumeModalReturn = [React.ReactNode, EditVolumeModalActions]

/** 编辑分卷弹窗 Hook 参数 */
export interface UseEditVolumeModalOptions {
  /** 小说 ID（用于拉取分卷、批量提交） */
  novelId: string
  /** 弹窗标题，默认为 '分卷' */
  title?: string
  /** 是否可创建分卷，默认为 true */
  creatable?: boolean
  /** 批量保存成功回调，参数为最新分卷列表 */
  onSaved?: (volumes: Volume[]) => void
}

/**
 * 构建批量提交 payload：
 * - id>0 视为已有分卷，保留 id 走更新
 * - id<=0（虚拟默认卷 id=0、本地新建 draft id 为负）脱去 id 作为新增
 */
const buildUpsertPayload = (list: SimpleVolume[]): VolumeUpsert[] =>
  list.map(({ id, name, sequence }) => {
    const isNew = !id || id < 0
    return isNew ? { name, sequence } : { id, name, sequence }
  })

/** 生成 draft 条目的临时负数 id，保证本地 rename/delete 可按 id 匹配 */
let draftIdSeed = 0
const nextDraftId = () => {
  draftIdSeed -= 1
  return draftIdSeed
}

/**
 * 编辑分卷弹窗 Hook
 *
 * 封装编辑分卷弹窗的 UI 与完整交互逻辑，返回 [modalContext, modalActions]。
 * - modalContext: 弹窗 UI 片段，需渲染到组件 JSX 中
 * - modalActions: 提供操作方法（open / close）
 *
 * @example
 * const [modalContext, modalActions] = useEditVolumeModal({ novelId, onSaved })
 * {modalContext}
 * modalActions.open()
 */
export const useEditVolumeModal = ({
  novelId,
  title = '分卷',
  creatable = true,
  onSaved,
}: UseEditVolumeModalOptions): UseEditVolumeModalReturn => {
  const [messageApi, contextHolder] = message.useMessage()
  const [open, setOpen] = useState(false)
  const [volumes, setVolumes] = useState<SimpleVolume[]>([])
  const [submitting, setSubmitting] = useState(false)
  const [latestHasChapters, setLatestHasChapters] = useState(false)

  const close = useCallback(() => setOpen(false), [])

  /** 查询最新一卷下是否存在章节 */
  const checkLatestHasChapters = useCallback(
    async (latest: SimpleVolume) => {
      if (!latest.id) return false
      try {
        const res = await getChaptersWithPagination(novelId, 1, 1, {
          volumeId: latest.sequence,
          isDraft: false,
        })
        return res.total > 0
      } catch (e) {
        logger.error('查询最新一卷章节数失败:', e)
        return false
      }
    },
    [novelId],
  )

  /** 打开弹窗：重置状态并拉取最新分卷 */
  const openAndLoad = useCallback(async () => {
    setOpen(true)
    setVolumes([])
    setLatestHasChapters(false)
    try {
      const list = await getVolumes(novelId)
      const resolved = resolveVolumesWithDefault(list, novelId)
      setVolumes(resolved.map(({ id, name, sequence }) => ({ id, name, sequence })))
      const hasChapters = await checkLatestHasChapters(resolved[resolved.length - 1])
      setLatestHasChapters(hasChapters)
    } catch (e) {
      logger.error('加载分卷列表失败:', e)
      messageApi.error('加载分卷列表失败')
    }
  }, [novelId, checkLatestHasChapters, messageApi])

  /** 本地重命名 */
  const handleRename = useCallback((id: number, newName: string) => {
    setVolumes((prev) => prev.map((v) => (v.id === id ? { ...v, name: newName } : v)))
  }, [])

  /** 本地删除 */
  const handleDelete = useCallback((id: number) => {
    setVolumes((prev) => prev.filter((v) => v.id !== id))
  }, [])

  /** 新建分卷：追加空白条目（由 EditVolumeItem 根据 isNew 自动进入编辑态） */
  const handleCreate = useCallback(() => {
    setVolumes((prev) => {
      const nextSeq = prev.length ? Math.max(...prev.map((v) => v.sequence)) + 1 : 1
      return [...prev, { id: nextDraftId(), name: '', sequence: nextSeq }]
    })
  }, [])

  /** 确认保存：批量提交 */
  const handleSubmit = useCallback(async () => {
    setSubmitting(true)
    try {
      const payload = buildUpsertPayload(volumes)
      const next = await batchUpdateVolumes(novelId, payload)
      messageApi.success('保存成功')
      onSaved?.(next)
      setOpen(false)
    } catch (e) {
      logger.error('保存分卷失败:', e)
      messageApi.error('保存分卷失败')
    } finally {
      setSubmitting(false)
    }
  }, [volumes, novelId, messageApi, onSaved])

  /** 计算某条分卷的删除态 */
  const resolveDeletable = useCallback(
    (v: SimpleVolume, idx: number) => {
      if (volumes.length <= 1) return { deletable: false, reason: '至少保留一卷' }
      if (idx !== volumes.length - 1) return { deletable: false, reason: '仅可删除最后一卷' }
      if (v.id && latestHasChapters) {
        return { deletable: false, reason: '该卷下存在章节，无法删除' }
      }
      return { deletable: true, reason: '' }
    },
    [volumes.length, latestHasChapters],
  )

  const renderFooter: Exclude<ModalProps['footer'], React.ReactNode> = useCallback(
    (_originNode, { OkBtn: _OkBtn, CancelBtn }) => {
      return (
        <div className="flex justify-between items-center w-full">
          <Space>
            {creatable && (
              <Button icon={<PlusCircleOutlined />} type="text" onClick={handleCreate}>
                新建分卷
              </Button>
            )}
          </Space>
          <Space>
            <CancelBtn />
            <Button type="primary" loading={submitting} onClick={handleSubmit}>
              确认
            </Button>
          </Space>
        </div>
      )
    },
    [creatable, submitting, handleCreate, handleSubmit],
  )

  const renderedBody = useMemo(
    () => (
      <div className="flex flex-col gap-2 min-h-50">
        {volumes.map((volume, idx) => {
          const { deletable, reason } = resolveDeletable(volume, idx)
          const notPersisted = !volume.id || volume.id < 0
          // isNew：新建且尚未命名 —— 取消时整条移除
          const isNew = notPersisted && !volume.name
          // isDraft：尚未落库（含已命名未提交） —— 删除时免二次确认
          const isDraft = notPersisted
          return (
            <EditVolumeItem
              key={volume.id}
              volume={volume}
              isNew={isNew}
              isDraft={isDraft}
              deletable={deletable}
              disabledReason={reason}
              onRename={(name) => handleRename(volume.id, name)}
              onDelete={() => handleDelete(volume.id)}
              onCancelCreate={() => handleDelete(volume.id)}
            />
          )
        })}
      </div>
    ),
    [volumes, resolveDeletable, handleRename, handleDelete],
  )

  const modalActions: EditVolumeModalActions = useMemo(
    () => ({ open: openAndLoad, close }),
    [openAndLoad, close],
  )

  const modalContext = (
    <>
      {contextHolder}
      <Modal
        title={title}
        open={open}
        onCancel={close}
        footer={renderFooter}
        mask={{ closable: !submitting }}
        closable={!submitting}
      >
        <div>{renderedBody}</div>
      </Modal>
    </>
  )

  return [modalContext, modalActions]
}
