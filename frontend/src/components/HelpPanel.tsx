import { Tooltip } from './Tooltip'
import { getGlossaryTerm } from '../lib/glossary'

export interface HelpPanelProps {
  title: string
  description: string
  tips?: string[]
  learnMoreUrl?: string
  glossaryTerms?: string[]
}

/**
 * Contextual help panel for complex operations
 * Shows description, tips, and related glossary terms
 */
export function HelpPanel({
  title,
  description,
  tips = [],
  learnMoreUrl,
  glossaryTerms = [],
}: HelpPanelProps) {
  return (
    <div className="bg-slate-900/50 border border-slate-700 rounded-lg p-4 mb-4">
      <div className="flex items-start gap-3">
        <svg
          viewBox="0 0 20 20"
          fill="currentColor"
          className="w-5 h-5 text-stellar-400 flex-shrink-0 mt-0.5"
        >
          <path
            fillRule="evenodd"
            d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
            clipRule="evenodd"
          />
        </svg>
        <div className="flex-1">
          <h3 className="font-semibold text-slate-100 mb-1">{title}</h3>
          <p className="text-sm text-slate-300 mb-2">{description}</p>

          {tips.length > 0 && (
            <div className="mb-2">
              <p className="text-xs font-semibold text-slate-400 mb-1">
                💡 Tips:
              </p>
              <ul className="text-xs text-slate-300 space-y-1 ml-4">
                {tips.map((tip, idx) => (
                  <li key={idx} className="list-disc">
                    {tip}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {glossaryTerms.length > 0 && (
            <div className="mb-2">
              <p className="text-xs font-semibold text-slate-400 mb-1">
                📖 Related Terms:
              </p>
              <div className="flex flex-wrap gap-2">
                {glossaryTerms.map((term) => {
                  const glossTerm = getGlossaryTerm(term)
                  return glossTerm ? (
                    <Tooltip
                      key={term}
                      content={glossTerm.definition}
                      position="top"
                      title={glossTerm.term}
                      icon={false}
                    >
                      <span className="inline-block bg-slate-800 text-slate-300 px-2 py-1 rounded text-xs hover:bg-slate-700 transition-colors cursor-help">
                        {glossTerm.term}
                      </span>
                    </Tooltip>
                  ) : null
                })}
              </div>
            </div>
          )}

          {learnMoreUrl && (
            <a
              href={learnMoreUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-block text-xs text-stellar-400 hover:text-stellar-300 transition-colors font-medium"
            >
              Learn more →
            </a>
          )}
        </div>
      </div>
    </div>
  )
}

/**
 * Inline help text that appears next to form fields
 */
export function FieldHelp({ children }: { children: string | React.ReactNode }) {
  return <p className="text-xs text-slate-400 mt-1">{children}</p>
}

/**
 * Example text shown below form fields
 */
export function FieldExample({ children }: { children: string }) {
  return (
    <p className="text-xs text-slate-500 mt-1 pl-3 border-l-2 border-slate-700">
      Example: <span className="text-slate-400 font-mono">{children}</span>
    </p>
  )
}
