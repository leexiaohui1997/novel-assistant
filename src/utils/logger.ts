import log from 'loglevel'

// 根据环境设置日志级别
if (import.meta.env.DEV) {
  log.setLevel('debug')
} else {
  log.setLevel('warn')
}

export const logger = {
  debug: (...args: unknown[]) => {
    log.debug('[DEBUG]', ...args)
  },
  info: (...args: unknown[]) => {
    log.info('[INFO]', ...args)
  },
  warn: (...args: unknown[]) => {
    log.warn('[WARN]', ...args)
  },
  error: (...args: unknown[]) => {
    log.error('[ERROR]', ...args)
  },
}
