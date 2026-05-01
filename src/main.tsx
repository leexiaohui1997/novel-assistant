import { ConfigProvider, App as AntdApp } from 'antd'
import zhCN from 'antd/locale/zh_CN'
import React from 'react'
import ReactDOM from 'react-dom/client'
import { Provider } from 'react-redux'

import App from './App'
import { store } from './store'
import './styles/global.css'
import './styles/dialog.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ConfigProvider
      locale={zhCN.default}
      theme={{
        token: {
          colorPrimary: '#ff6b35',
        },
      }}
    >
      <Provider store={store}>
        <AntdApp>
          <App />
        </AntdApp>
      </Provider>
    </ConfigProvider>
  </React.StrictMode>,
)
