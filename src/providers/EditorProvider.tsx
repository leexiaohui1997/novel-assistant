import { HistoryOutlined, LeftOutlined } from '@ant-design/icons'
import { Button, Input, Modal, Select, Tooltip, message } from 'antd'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import { EditorContext } from './EditorContext'

import type { EditorContextType, EditorOpenOptions } from './EditorContext'
import type { EditorHandle } from '@/components/Editor'

import Editor from '@/components/Editor'
import VersionDrawer from '@/components/VersionDrawer'
import {
  DEFAULT_VOLUME_SEQUENCE,
  createChapter,
  getChaptersWithPagination,
  getVolumes,
  resolveVolumesWithDefault,
  updateChapter,
  type Chapter,
  type ChapterVersion,
  type Volume,
} from '@/services/chapterService'
import { logger } from '@/utils/logger'
import { numToCn } from '@/utils/number'

/**
 * 查询指定分卷下「下一章」的序号（末章 sequence + 1，无章节则为 1）
 *
 * @param novelId - 小说 ID
 * @param volumeSequence - 分卷业务序号
 * @returns 下一章的 sequence
 */
const fetchNextChapterSequence = async (
  novelId: string,
  volumeSequence: number,
): Promise<number> => {
  const result = await getChaptersWithPagination(novelId, 1, 1, {
    volumeSequence,
    sortField: 'sequence',
    sortOrder: 'desc',
  })
  const last = result.data[0]
  return last ? last.sequence + 1 : 1
}

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

/**
 * 表单校验：标题与正文均不能为空（trim 后）
 *
 * @returns 校验通过返回 true，否则展示提示并返回 false
 */
const validateForm = (title: string, content: string, notify: (msg: string) => void): boolean => {
  if (!title.trim()) {
    notify('标题不能为空')
    return false
  }
  if (!content.trim()) {
    notify('正文不能为空')
    return false
  }
  return true
}

/** 保存调用参数 */
interface SaveChapterParams {
  chapter?: Chapter
  chapterId?: string
  novelId: string
  title: string
  content: string
  volumeId?: number
  sequence?: number
}

/**
 * 按模式（新建 / 编辑）分派到对应 API
 *
 * 新建时将 `sequence` 一起传给后端，直接落库为正式章节；
 * 编辑时保留原有章节序号不变。
 */
const saveChapter = async (params: SaveChapterParams): Promise<Chapter> => {
  const { chapter, novelId, title, content, volumeId, sequence } = params
  if (chapter) {
    return updateChapter(chapter.id, { title, content, sequence, volumeId })
  }
  return createChapter(novelId, { title, content, volumeId, sequence })
}

/**
 * 快捷键保存 Hook：监听 Ctrl/Cmd + S 触发保存
 *
 * @param onSave - 触发保存的回调
 */
const useSaveShortcut = (onSave: () => void): void => {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const isSaveKey = (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's'
      if (!isSaveKey) return
      e.preventDefault()
      onSave()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [onSave])
}

/**
 * 加载分卷列表 Hook
 *
 * 打开编辑器时拉取分卷列表并按策略解析初始选中 sequence
 */
const useLoadVolumes = (
  novelId: string,
  preferSequence: number | undefined,
): {
  volumes: Volume[]
  selectedSequence: number
  setSelectedSequence: React.Dispatch<React.SetStateAction<number>>
} => {
  const [volumes, setVolumes] = useState<Volume[]>([])
  const [selectedSequence, setSelectedSequence] = useState<number>(
    preferSequence ?? DEFAULT_VOLUME_SEQUENCE,
  )

  useEffect(() => {
    let cancelled = false
    const load = async () => {
      try {
        const list = await getVolumes(novelId)
        if (cancelled) return
        const resolved = resolveVolumesWithDefault(list, novelId)
        setVolumes(resolved)
        setSelectedSequence(resolveInitialSequence(resolved, preferSequence))
      } catch (e) {
        logger.error('编辑器加载分卷列表失败:', e)
      }
    }
    load()
    return () => {
      cancelled = true
    }
  }, [novelId, preferSequence])

  return { volumes, selectedSequence, setSelectedSequence }
}

/**
 * 新建模式下获取下一章序号
 *
 * @returns nextSequence（null 表示尚未加载或失败）
 */
const useNextSequence = (
  isEdit: boolean,
  novelId: string,
  selectedSequence: number | undefined,
): number | null => {
  const [nextSequence, setNextSequence] = useState<number | null>(null)

  useEffect(() => {
    if (isEdit || !selectedSequence) return
    let cancelled = false
    const run = async () => {
      try {
        const next = await fetchNextChapterSequence(novelId, selectedSequence)
        if (!cancelled) setNextSequence(next)
      } catch (e) {
        logger.error('加载下一章序号失败:', e)
        if (!cancelled) setNextSequence(null)
      }
    }
    run()
    return () => {
      cancelled = true
    }
  }, [isEdit, novelId, selectedSequence])

  return nextSequence
}

/** 保存处理 Hook 参数 */
interface UseSaveHandlerParams {
  chapter?: Chapter
  novelId: string
  title: string
  content: string
  volumeId?: number
  /** 新建模式下的"下一章序号"；编辑模式下忽略 */
  nextSequence: number | null
  messageApi: ReturnType<typeof message.useMessage>[0]
  /** 保存成功后的回调 */
  onSuccess: (savedChapter: Chapter) => void
}
/**
 * 保存处理 Hook：封装校验、调接口、反馈、loading 状态
 */
const useSaveHandler = (
  params: UseSaveHandlerParams,
): { saving: boolean; handleSave: () => Promise<void> } => {
  const { chapter, novelId, title, content, volumeId, nextSequence, messageApi, onSuccess } = params
  const [saving, setSaving] = useState(false)

  const handleSave = useCallback(async () => {
    if (saving) return
    if (!validateForm(title, content, messageApi.warning)) return
    try {
      setSaving(true)
      const savedChapter = await saveChapter({
        chapter,
        novelId,
        title: title.trim(),
        content,
        volumeId,
        sequence: chapter ? chapter.sequence : (nextSequence ?? undefined),
      })
      messageApi.success('保存成功')
      onSuccess(savedChapter)
    } catch (e) {
      logger.error('保存章节失败:', e)
      messageApi.error('保存失败，请重试')
    } finally {
      setSaving(false)
    }
  }, [saving, title, content, messageApi, chapter, novelId, volumeId, nextSequence, onSuccess])

  return { saving, handleSave }
}

/**
 * 关闭处理 Hook：脏检测 + 二次确认
 */
const useCloseHandler = (
  isDirty: boolean,
  modal: ReturnType<typeof Modal.useModal>[0],
  onClose: () => void,
): (() => void) => {
  return useCallback(() => {
    if (!isDirty) {
      onClose()
      return
    }
    modal.confirm({
      title: '有未保存的修改',
      content: '确定要离开吗？未保存的内容将会丢失。',
      okText: '离开',
      cancelText: '继续编辑',
      okButtonProps: { danger: true },
      onOk: onClose,
    })
  }, [isDirty, modal, onClose])
}

/**
 * 章节表单 Hook：管理标题/正文 state 及脏检测
 */
const useChapterForm = (
  chapter: Chapter | undefined,
): {
  title: string
  content: string
  setTitle: React.Dispatch<React.SetStateAction<string>>
  setContent: React.Dispatch<React.SetStateAction<string>>
  isDirty: boolean
  resetBaseline: () => void
} => {
  const [initialTitle, setInitialTitle] = useState(chapter?.title ?? '')
  const [initialContent, setInitialContent] = useState(chapter?.content ?? '')
  const [title, setTitle] = useState<string>(initialTitle)
  const [content, setContent] = useState<string>(initialContent)
  const isDirty = useMemo(
    () => title !== initialTitle || content !== initialContent,
    [title, content, initialTitle, initialContent],
  )
  const resetBaseline = useCallback(() => {
    setInitialTitle(title)
    setInitialContent(content)
  }, [title, content])
  return { title, content, setTitle, setContent, isDirty, resetBaseline }
}

/**
 * 计算展示用的章节序号：编辑态取 chapter.sequence，新建态取 nextSequence
 */
const computeChapterSequence = (
  isEdit: boolean,
  chapter: Chapter | undefined,
  nextSequence: number | null,
): number | null => {
  if (isEdit) return chapter?.sequence ?? null
  return nextSequence
}

/** EditorHeader 属性 */
interface EditorHeaderProps {
  novelTitle: string
  selectedSequence: number
  onSelectSequence: (s: number) => void
  volumeOptions: { value: number; label: string }[]
  volumeLoading: boolean
  isEdit: boolean
  chapterSequence: number | null
  saving: boolean
  onSave: () => void
  onClose: () => void
  /** 历史记录按钮点击回调 */
  onOpenHistory: () => void
  /** 历史记录按钮是否禁用（新建模式下禁用） */
  historyDisabled: boolean
}

/** 编辑器顶部栏 */
const EditorHeader: React.FC<EditorHeaderProps> = ({
  novelTitle,
  selectedSequence,
  onSelectSequence,
  volumeOptions,
  volumeLoading,
  isEdit,
  chapterSequence,
  saving,
  onSave,
  onClose,
  onOpenHistory,
  historyDisabled,
}) => {
  const sequenceLabel = chapterSequence === null ? '' : `第 ${chapterSequence} 章`
  return (
    <div className="flex items-center justify-between">
      <div className="flex flex-1 items-center gap-3">
        <Button color="default" variant="filled" icon={<LeftOutlined />} onClick={onClose} />
        <span className="text-base font-medium">{novelTitle}</span>
        <Select
          className="min-w-50 font-normal"
          value={selectedSequence}
          options={volumeOptions}
          disabled={isEdit}
          onChange={onSelectSequence}
          loading={volumeLoading}
        />
        <div className="w-20">
          <Input readOnly className="font-normal" value={sequenceLabel} placeholder="加载中..." />
        </div>
      </div>
      <div className="flex items-center gap-2">
        <Tooltip title="历史记录" placement="bottom" mouseEnterDelay={1} color="white">
          <Button
            type="text"
            icon={<HistoryOutlined />}
            disabled={historyDisabled}
            onClick={onOpenHistory}
          />
        </Tooltip>
        <Button type="primary" shape="round" loading={saving} onClick={onSave}>
          保存
        </Button>
      </div>
    </div>
  )
}

/** 全屏编辑器弹窗 */
const EditorModal: React.FC<
  EditorOpenOptions & { onClose: () => void; onChapterCreated: (chapter: Chapter) => void }
> = ({ novel, chapter: initialChapter, sequence, onSubmit, onClose, onChapterCreated }) => {
  // 新建转编辑：首次保存成功后用后端返回的 chapter 更新引用
  const [activeChapter, setActiveChapter] = useState(initialChapter)
  const isEdit = Boolean(activeChapter)
  const editorRef = useRef<EditorHandle>(null)

  // 分卷列表与选中项
  const { volumes, selectedSequence, setSelectedSequence } = useLoadVolumes(novel.id, sequence)
  // 编辑器表单：标题 / 正文 + 脏检测 + 重置基准
  const { title, content, setTitle, setContent, isDirty, resetBaseline } =
    useChapterForm(activeChapter)
  // antd message / modal hooks
  const [messageApi, messageCtx] = message.useMessage()
  const [modal, modalCtx] = Modal.useModal()

  // 当前选中分卷对应的分卷 ID（用于保存章节时写入 volume_chapters 关联）
  const selectedVolumeId = useMemo(
    () => volumes.find((v) => v.sequence === selectedSequence)?.id,
    [volumes, selectedSequence],
  )

  // 新建模式下的下一章序号（按分卷 sequence 查询）
  const nextSequence = useNextSequence(isEdit, novel.id, selectedSequence)
  const chapterSequence = computeChapterSequence(isEdit, activeChapter, nextSequence)

  // 分卷选择器选项
  const volumeOptions = useMemo(
    () =>
      volumes.map((v) => ({
        value: v.sequence,
        label: `第${numToCn(v.sequence)}卷：${v.name}`,
      })),
    [volumes],
  )

  const handleSaveSuccess = useCallback(
    (savedChapter: Chapter) => {
      // 新建模式首次保存：更新 chapter 引用，切换为编辑模式
      if (!activeChapter) {
        setActiveChapter(savedChapter)
        onChapterCreated(savedChapter)
      }
      onSubmit?.()
      resetBaseline()
    },
    [activeChapter, onChapterCreated, onSubmit, resetBaseline],
  )

  // 历史版本 Drawer 开关
  const [historyOpen, setHistoryOpen] = useState(false)
  const openHistory = useCallback(() => setHistoryOpen(true), [])
  const closeHistory = useCallback(() => setHistoryOpen(false), [])

  // 应用历史版本：覆盖标题/正文，不自动保存，Drawer 保持打开
  const handleApplyVersion = useCallback(
    (version: ChapterVersion) => {
      setTitle(version.title)
      setContent(version.content)
      editorRef.current?.setContent(version.content)
      messageApi.success('应用成功')
    },
    [setTitle, setContent, messageApi],
  )

  // 保存处理
  const { saving, handleSave } = useSaveHandler({
    chapter: activeChapter,
    novelId: novel.id,
    title,
    content,
    volumeId: selectedVolumeId,
    nextSequence,
    messageApi,
    onSuccess: handleSaveSuccess,
  })

  // 关闭处理（带脏检测）
  const handleClose = useCloseHandler(isDirty, modal, onClose)

  // 快捷键：Ctrl / Cmd + S 保存
  useSaveShortcut(handleSave)

  return (
    <>
      {messageCtx}
      {modalCtx}
      <Modal
        open
        closable={false}
        footer={null}
        title={
          <EditorHeader
            novelTitle={novel.title}
            selectedSequence={selectedSequence}
            onSelectSequence={setSelectedSequence}
            volumeOptions={volumeOptions}
            volumeLoading={volumes.length === 0}
            isEdit={isEdit}
            chapterSequence={chapterSequence}
            saving={saving}
            onSave={handleSave}
            onClose={handleClose}
            onOpenHistory={openHistory}
            historyDisabled={!isEdit}
          />
        }
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
                value={title}
                onChange={(e) => setTitle(e.target.value)}
              />
            </div>

            {/* 正文 */}
            <Editor
              ref={editorRef}
              className="flex-1"
              placeholder="请输入正文"
              value={content}
              onChange={setContent}
            />
          </div>
        </div>
      </Modal>
      <VersionDrawer
        open={historyOpen}
        onClose={closeHistory}
        chapterId={activeChapter?.id}
        onApply={handleApplyVersion}
      />
    </>
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

  const handleChapterCreated = useCallback((_chapter: Chapter) => {
    // 预留：新建章节创建成功后的扩展点
    // 当前新建转编辑逻辑已在 EditorModal 内部处理
  }, [])

  const contextValue: EditorContextType = useMemo(() => ({ open, close }), [open, close])

  return (
    <EditorContext.Provider value={contextValue}>
      {children}
      {options && (
        <EditorModal {...options} onClose={close} onChapterCreated={handleChapterCreated} />
      )}
    </EditorContext.Provider>
  )
}
