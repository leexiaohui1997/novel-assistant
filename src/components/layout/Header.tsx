import { APP_NAME } from '@/utils/constants'
import './styles.css'

const Header: React.FC = () => {
  return (
    <header className="header">
      <div className="header-left">
        <h1 className="header-title">{APP_NAME}</h1>
      </div>
      <div className="header-right" />
    </header>
  )
}

export default Header
