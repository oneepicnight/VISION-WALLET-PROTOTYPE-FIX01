import { useNavigate } from 'react-router-dom'

export default function Splash() {
  const navigate = useNavigate()

  return (
    <div className="splash-container">
      <div className="splash-content">
        {/* Animated background grid */}
        <div className="splash-bg">
          <div className="splash-gradient"></div>
        </div>
        
        <div>
          <h1 className="splash-title">
            Welcome, <span className="splash-accent">Dreamer</span>.
          </h1>
          <p className="splash-subtitle">
            The world is yours to shape.
          </p>
        </div>
        
        <button 
          onClick={() => navigate('/handle')}
          className="splash-button"
        >
          Enter
        </button>
      </div>
    </div>
  )
}