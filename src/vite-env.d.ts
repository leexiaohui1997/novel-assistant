/// <reference types="vite/client" />

/**
 * 应用环境变量接口
 * 定义所有以 VITE_ 开头的环境变量及其类型
 */
interface ImportMetaEnv {
  /** 应用程序标题 */
  readonly VITE_APP_TITLE: string
}

/**
 * 扩展 ImportMeta 接口
 * 将自定义的环境变量类型添加到 import.meta 对象上
 */
interface ImportMeta {
  readonly env: ImportMetaEnv
}
