import { name, version } from '@/../package.json'

// 应用常量定义
export const APP_NAME = import.meta.env.VITE_APP_TITLE || name
export const APP_VERSION = version
