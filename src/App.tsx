import { useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { useWalletStore } from './state/wallet'
import { pingStatus } from './lib/api'
import { requireWallet } from './lib/guards'
import Splash from './routes/Splash'
import HandleClaim from './routes/HandleClaim'
import SecureKey from './routes/SecureKey'
import Home from './routes/Home'
import Settings from './routes/Settings'
import { Market, Orders } from './modules/market'
import { VaultCard } from './components/VaultCard'
import { env } from './utils/env'

// Protected components
const ProtectedHome = requireWallet(Home)
const ProtectedSettings = requireWallet(Settings)

// Status bar component
function StatusBar() {
  const { profile, node } = useWalletStore()
  
  const statusColor = {
    'up': 'up',
    'degraded': 'degraded', 
    'down': 'down'
  }[node.status]

  const handleEnterVision = () => {
    if (!profile) return

    // Try deep link first
    const deepLink = `vision://enter?address=${encodeURIComponent(profile.address)}&handle=${encodeURIComponent(profile.handle)}`
    
    // Create a temporary link to test if protocol handler exists
    const link = document.createElement('a')
    link.href = deepLink
    
    // Show connecting toast
    const showToast = (message: string) => {
      // Simple toast implementation
      const toast = document.createElement('div')
      toast.textContent = message
      toast.className = 'fixed top-16 right-4 bg-accent/90 text-white px-4 py-2 rounded-lg shadow-lg z-50'
      document.body.appendChild(toast)
      setTimeout(() => toast.remove(), 3000)
    }

    showToast('Connecting to portal...')
    
    try {
      // Try the deep link
      link.click()
      
      // Fallback after a short delay if deep link doesn't work
      setTimeout(() => {
        const fallbackUrl = 'http://127.0.0.1:5173/vision'
        window.open(fallbackUrl, '_blank')
      }, 1000)
    } catch (error) {
      // Immediate fallback
      const fallbackUrl = 'http://127.0.0.1:5173/vision'
      window.open(fallbackUrl, '_blank')
    }
  }

  return (
    <div className="status-bar">
      <div className="status-left">
        <div className={`status-dot ${statusColor}`} title={`Node ${node.status}`}></div>
        <span className="status-handle">
          {profile ? `@${profile.handle}` : '@handle'}
        </span>
      </div>
      <div className="status-right">
        {profile && (
          <a 
            href="/settings"
            className="settings-link"
            title="Settings"
          >
            ⚙️
          </a>
        )}
        <button 
          onClick={handleEnterVision}
          disabled={!profile} 
          className="enter-vision-btn"
        >
          Enter Vision
        </button>
      </div>
    </div>
  )
}

function App() {
  const { setNode } = useWalletStore()

  // Poll node status every 5 seconds
  useEffect(() => {
    const checkStatus = async () => {
      try {
        const result = await pingStatus()
        const now = Date.now()
        
        if (result.up && result.info && typeof result.info === 'object') {
          // Node is responding with valid data
          setNode({ status: 'up', lastSeen: now })
        } else if (result.up) {
          // Node responding but missing fields
          setNode({ status: 'degraded', lastSeen: now })
        } else {
          // Network error
          setNode({ status: 'down' })
        }
      } catch (error) {
        setNode({ status: 'down' })
      }
    }

    // Check immediately
    checkStatus()
    
    // Then poll every 5 seconds
    const interval = setInterval(checkStatus, 5000)
    
    return () => clearInterval(interval)
  }, [setNode])

  return (
    <Router>
      <StatusBar />
      <nav className="top-nav">
        <a href="/home">Home</a>
        <a href="/market">Market</a>
        <a href="/settings">Settings</a>
  {env.FEATURE_DEV_PANEL && <a href="/miner">Miner</a>}
  {env.FEATURE_DEV_PANEL && <a href="/orders">Orders</a>}
      </nav>
      <div className="main-content">
        <div className="p-4">
          <VaultCard />
        </div>
        <Routes>
          <Route path="/" element={<Splash />} />
          <Route path="/market" element={<Market />} />
          <Route path="/handle" element={<HandleClaim />} />
          <Route path="/secure" element={<SecureKey />} />
          <Route path="/home" element={<ProtectedHome />} />
          {env.FEATURE_DEV_PANEL && <Route path="/orders" element={<Orders />} />}
          <Route path="/settings" element={<ProtectedSettings />} />
        </Routes>
      </div>
    </Router>
  )
}

export default App