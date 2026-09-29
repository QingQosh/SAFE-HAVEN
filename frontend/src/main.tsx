import React from 'react'
import ReactDOM from 'react-dom/client'
import { Toaster } from 'react-hot-toast'
import App from './App'
import { validateConfig } from './config'
import './index.css'

// ============================================================
//  Config validation at startup (#84)
// ============================================================
const configErrors = validateConfig()

if (configErrors.length > 0) {
  // Render full-page error UI with actionable instructions
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <div style={{
      minHeight: '100vh',
      background: 'linear-gradient(to bottom, #0f172a, #1e293b)',
      color: '#f1f5f9',
      padding: '2rem',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      <div style={{ maxWidth: '42rem', margin: '0 auto' }}>
        <div style={{
          background: '#1e293b',
          border: '1px solid #ef4444',
          borderRadius: '0.75rem',
          padding: '2rem',
          marginTop: '4rem'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '1.5rem' }}>
            <div style={{
              width: '3rem',
              height: '3rem',
              background: '#ef4444',
              borderRadius: '50%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '1.5rem',
              fontWeight: 'bold'
            }}>!</div>
            <h1 style={{ fontSize: '1.5rem', fontWeight: 'bold', margin: 0 }}>
              Configuration Error
            </h1>
          </div>
          
          <p style={{ marginBottom: '1.5rem', color: '#cbd5e1' }}>
            SAFE-HAVEN detected {configErrors.length} configuration {configErrors.length === 1 ? 'error' : 'errors'}.
            The app cannot start until these are fixed.
          </p>

          {configErrors.map((error, idx) => (
            <div key={idx} style={{
              background: '#0f172a',
              border: '1px solid #334155',
              borderRadius: '0.5rem',
              padding: '1rem',
              marginBottom: '1rem'
            }}>
              <div style={{ fontWeight: 'bold', color: '#ef4444', marginBottom: '0.5rem' }}>
                {error.field}
              </div>
              <div style={{ marginBottom: '0.5rem', fontSize: '0.875rem' }}>
                {error.message}
              </div>
              <div style={{
                background: '#1e293b',
                border: '1px solid #22c55e',
                borderRadius: '0.375rem',
                padding: '0.75rem',
                fontSize: '0.875rem',
                color: '#22c55e'
              }}>
                <strong>Fix:</strong> {error.fix}
              </div>
            </div>
          ))}

          <div style={{
            marginTop: '2rem',
            padding: '1rem',
            background: '#0f172a',
            border: '1px solid #334155',
            borderRadius: '0.5rem',
            fontSize: '0.875rem'
          }}>
            <div style={{ fontWeight: 'bold', marginBottom: '0.5rem' }}>Quick Start</div>
            <ol style={{ margin: 0, paddingLeft: '1.5rem', color: '#cbd5e1' }}>
              <li>Copy <code style={{ background: '#1e293b', padding: '0.125rem 0.375rem', borderRadius: '0.25rem' }}>.env.example</code> to <code style={{ background: '#1e293b', padding: '0.125rem 0.375rem', borderRadius: '0.25rem' }}>.env</code></li>
              <li>Set all required environment variables in <code style={{ background: '#1e293b', padding: '0.125rem 0.375rem', borderRadius: '0.25rem' }}>.env</code></li>
              <li>Restart the dev server with <code style={{ background: '#1e293b', padding: '0.125rem 0.375rem', borderRadius: '0.25rem' }}>npm run dev</code></li>
            </ol>
          </div>
        </div>
      </div>
    </div>
  )
} else {
  // Config is valid — render the app normally
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App />
      <Toaster
        position="top-right"
        toastOptions={{
          duration: 5000,
          style: {
            background: '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '0.75rem',
            fontSize: '0.875rem',
          },
          success: {
            iconTheme: { primary: '#22c55e', secondary: '#1e293b' },
          },
          error: {
            iconTheme: { primary: '#ef4444', secondary: '#1e293b' },
          },
        }}
      />
    </React.StrictMode>,
  )
}
