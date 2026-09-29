/**
 * Security Context — tracks user's security practice adoption
 * Stores which security recommendations the user has acknowledged/completed
 */

import React, { createContext, useCallback, useContext, useEffect, useState } from 'react'
import { SecurityChecklist, SecurityChecklistItem } from '../lib/security'

interface SecurityContextValue {
  /** Checklist of completed security practices */
  checklist: SecurityChecklist
  /** Mark a security practice as completed/acknowledged */
  markComplete: (item: SecurityChecklistItem) => void
  /** Mark a security practice as incomplete */
  markIncomplete: (item: SecurityChecklistItem) => void
  /** Get completion count */
  getCompletionCount: () => number
  /** Reset all security practices */
  resetChecklist: () => void
  /** Get completion percentage */
  getCompletionPercentage: () => number
}

const SecurityContext = createContext<SecurityContextValue | null>(null)

const SECURITY_CHECKLIST_STORAGE_KEY = 'safe-haven_security_checklist'

/**
 * Initialize security checklist from localStorage
 */
function initializeChecklist(): SecurityChecklist {
  if (typeof window === 'undefined' || typeof localStorage === 'undefined') {
    return {}
  }

  try {
    const saved = localStorage.getItem(SECURITY_CHECKLIST_STORAGE_KEY)
    if (saved) {
      return JSON.parse(saved) as SecurityChecklist
    }
  } catch (error) {
    console.error('Failed to load security checklist:', error)
  }

  return {}
}

export function SecurityProvider({ children }: { children: React.ReactNode }) {
  const [checklist, setChecklist] = useState<SecurityChecklist>(
    initializeChecklist()
  )

  // Persist checklist to localStorage whenever it changes
  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(SECURITY_CHECKLIST_STORAGE_KEY, JSON.stringify(checklist))
      } catch (error) {
        console.error('Failed to save security checklist:', error)
      }
    }
  }, [checklist])

  const markComplete = useCallback((item: SecurityChecklistItem) => {
    setChecklist((prev) => ({
      ...prev,
      [item]: true,
    }))
  }, [])

  const markIncomplete = useCallback((item: SecurityChecklistItem) => {
    setChecklist((prev) => ({
      ...prev,
      [item]: false,
    }))
  }, [])

  const getCompletionCount = useCallback(() => {
    return Object.values(checklist).filter((v) => v === true).length
  }, [checklist])

  const getCompletionPercentage = useCallback(() => {
    const totalItems = Object.keys(SecurityChecklistItem).length
    if (totalItems === 0) return 0
    return Math.round((getCompletionCount() / totalItems) * 100)
  }, [getCompletionCount])

  const resetChecklist = useCallback(() => {
    setChecklist({})
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.removeItem(SECURITY_CHECKLIST_STORAGE_KEY)
      } catch (error) {
        console.error('Failed to reset security checklist:', error)
      }
    }
  }, [])

  return (
    <SecurityContext.Provider
      value={{
        checklist,
        markComplete,
        markIncomplete,
        getCompletionCount,
        resetChecklist,
        getCompletionPercentage,
      }}
    >
      {children}
    </SecurityContext.Provider>
  )
}

/**
 * Hook to access security context
 * Must be used inside SecurityProvider
 */
export function useSecurity(): SecurityContextValue {
  const ctx = useContext(SecurityContext)
  if (!ctx) {
    throw new Error('useSecurity must be used inside SecurityProvider')
  }
  return ctx
}
