import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'

// Disable default browser context menu
document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
});

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
