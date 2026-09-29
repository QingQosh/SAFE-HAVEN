import { useState, useRef, useEffect } from 'react'

export interface TooltipProps {
  /** The content to show in tooltip */
  content: string | React.ReactNode
  /** Position of tooltip relative to trigger */
  position?: 'top' | 'bottom' | 'left' | 'right'
  /** Delay in ms before showing on hover */
  delay?: number
  /** Show tooltip on click in addition to hover */
  clickable?: boolean
  /** Optional title shown above content */
  title?: string
  /** Custom className for tooltip container */
  className?: string
  /** Children that trigger the tooltip */
  children: React.ReactNode
  /** Optional icon to show inline with children */
  icon?: boolean
}

/**
 * Reusable tooltip component with hover and click behavior
 * Supports multiple positions and customizable content
 */
export function Tooltip({
  content,
  position = 'top',
  delay = 300,
  clickable = false,
  title,
  className = '',
  children,
  icon = true,
}: TooltipProps) {
  const [isVisible, setIsVisible] = useState(false)
  const [isClickOpen, setIsClickOpen] = useState(false)
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>()
  const triggerRef = useRef<HTMLDivElement>(null)
  const tooltipRef = useRef<HTMLDivElement>(null)

  // Clear timeout on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  // Close tooltip when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        triggerRef.current &&
        !triggerRef.current.contains(event.target as Node) &&
        tooltipRef.current &&
        !tooltipRef.current.contains(event.target as Node)
      ) {
        setIsClickOpen(false)
      }
    }

    if (isClickOpen) {
      document.addEventListener('mousedown', handleClickOutside)
      return () => document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [isClickOpen])

  const handleMouseEnter = () => {
    timeoutRef.current = setTimeout(() => {
      setIsVisible(true)
    }, delay)
  }

  const handleMouseLeave = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
    }
    setIsVisible(false)
  }

  const handleClick = () => {
    if (clickable) {
      setIsClickOpen(!isClickOpen)
      setIsVisible(false)
    }
  }

  const show = isVisible || isClickOpen

  // Position classes for tooltip
  const positionClasses = {
    top: 'bottom-full mb-2 left-1/2 -translate-x-1/2',
    bottom: 'top-full mt-2 left-1/2 -translate-x-1/2',
    left: 'right-full mr-2 top-1/2 -translate-y-1/2',
    right: 'left-full ml-2 top-1/2 -translate-y-1/2',
  }

  // Arrow classes for tooltip
  const arrowClasses = {
    top: 'top-full left-1/2 -translate-x-1/2 border-t-slate-700 border-t-6 border-x-6 border-x-transparent border-b-0',
    bottom: 'bottom-full left-1/2 -translate-x-1/2 border-b-slate-700 border-b-6 border-x-6 border-x-transparent border-t-0',
    left: 'left-full top-1/2 -translate-y-1/2 border-l-slate-700 border-l-6 border-y-6 border-y-transparent border-r-0',
    right: 'right-full top-1/2 -translate-y-1/2 border-r-slate-700 border-r-6 border-y-6 border-y-transparent border-l-0',
  }

  return (
    <div className="relative inline-block" ref={triggerRef}>
      {/* Trigger */}
      <div
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        onClick={handleClick}
        className={`inline-flex items-center gap-1 cursor-help ${clickable ? 'cursor-pointer' : ''} ${className}`}
      >
        {children}
        {icon && (
          <svg
            viewBox="0 0 20 20"
            fill="currentColor"
            className="w-4 h-4 text-slate-500 hover:text-slate-300 transition-colors flex-shrink-0"
            role="img"
            aria-label="Help"
          >
            <path
              fillRule="evenodd"
              d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
              clipRule="evenodd"
            />
          </svg>
        )}
      </div>

      {/* Tooltip */}
      {show && (
        <div
          ref={tooltipRef}
          className={`absolute z-50 whitespace-normal bg-slate-800 border border-slate-600 rounded-lg shadow-lg p-3 max-w-xs text-sm text-slate-200 ${positionClasses[position]}`}
          role="tooltip"
        >
          {/* Arrow */}
          <div className={`absolute w-0 h-0 ${arrowClasses[position]}`} />

          {/* Content */}
          {title && (
            <div>
              <p className="font-semibold text-slate-100 mb-1">{title}</p>
            </div>
          )}
          <div className="leading-relaxed">{content}</div>
        </div>
      )}
    </div>
  )
}
