import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'
import ErrorBoundary from './components/ErrorBoundary'
import { ConfirmationProvider } from './context/ConfirmationContext'   // add

ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
        <ErrorBoundary>
            <ConfirmationProvider>      {/* wrap */}
                <App />
            </ConfirmationProvider>
        </ErrorBoundary>
    </React.StrictMode>,
)