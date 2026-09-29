import { useEffect, useState } from 'react'
import toast from 'react-hot-toast'

type FeedbackType = 'bug' | 'feature' | 'feedback'

interface FeedbackModalProps {
  onClose: () => void
}

export function FeedbackModal({ onClose }: FeedbackModalProps) {
  const [type, setType] = useState<FeedbackType>('feedback')
  const [description, setDescription] = useState('')
  const [email, setEmail] = useState('')
  const [screenshots, setScreenshots] = useState<File[]>([])

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') onClose()
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [onClose])

  function handleFiles(event: React.ChangeEvent<HTMLInputElement>) {
    const files = Array.from(event.target.files ?? []).filter((file) => file.type.startsWith('image/'))
    setScreenshots((current) => [...current, ...files].slice(0, 5))
    event.target.value = ''
  }

  function removeScreenshot(index: number) {
    setScreenshots((current) => current.filter((_, fileIndex) => fileIndex !== index))
  }

  function handleSubmit(event: React.FormEvent) {
    event.preventDefault()
    if (!description.trim()) return

    toast.success('Thanks for the feedback!')
    onClose()
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 p-4 backdrop-blur-sm"
      role="presentation"
      onMouseDown={(event) => { if (event.target === event.currentTarget) onClose() }}
    >
      <div
        className="card w-full max-w-lg max-h-[90vh] overflow-y-auto p-6 shadow-2xl"
        role="dialog"
        aria-modal="true"
        aria-labelledby="feedback-title"
      >
        <div className="flex items-start justify-between gap-4 mb-6">
          <div>
            <h2 id="feedback-title" className="text-lg font-semibold">Share feedback</h2>
            <p className="text-sm text-slate-400 mt-1">Help us improve SAFE-HAVEN.</p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="text-slate-400 hover:text-slate-100 text-xl leading-none"
            aria-label="Close feedback dialog"
          >
            ×
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-5">
          <div>
            <label htmlFor="feedback-type" className="label">Feedback type</label>
            <select
              id="feedback-type"
              className="input"
              value={type}
              onChange={(event) => setType(event.target.value as FeedbackType)}
            >
              <option value="bug">Bug report</option>
              <option value="feature">Feature request</option>
              <option value="feedback">General feedback</option>
            </select>
          </div>

          <div>
            <label htmlFor="feedback-description" className="label">Description</label>
            <textarea
              id="feedback-description"
              className="input min-h-32 resize-y"
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder="Tell us what happened or what you would like to see..."
              required
            />
          </div>

          <div>
            <label htmlFor="feedback-email" className="label">Email <span className="normal-case text-slate-500">(optional)</span></label>
            <input
              id="feedback-email"
              className="input"
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="you@example.com"
            />
          </div>

          <div>
            <label htmlFor="feedback-screenshots" className="label">Screenshots <span className="normal-case text-slate-500">(optional, up to 5)</span></label>
            <input
              id="feedback-screenshots"
              className="block w-full text-sm text-slate-400 file:mr-3 file:rounded-lg file:border-0 file:bg-slate-700 file:px-3 file:py-2 file:text-sm file:font-medium file:text-slate-100 hover:file:bg-slate-600"
              type="file"
              accept="image/*"
              multiple
              onChange={handleFiles}
              disabled={screenshots.length >= 5}
            />
            {screenshots.length > 0 && (
              <div className="grid grid-cols-5 gap-2 mt-3">
                {screenshots.map((file, index) => (
                  <ScreenshotPreview key={`${file.name}-${index}`} file={file} onRemove={() => removeScreenshot(index)} />
                ))}
              </div>
            )}
          </div>

          <div className="flex justify-end gap-3 pt-1">
            <button type="button" className="btn-secondary" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn-primary" disabled={!description.trim()}>Send feedback</button>
          </div>
        </form>
      </div>
    </div>
  )
}

function ScreenshotPreview({ file, onRemove }: { file: File; onRemove: () => void }) {
  const [previewUrl, setPreviewUrl] = useState('')

  useEffect(() => {
    const url = URL.createObjectURL(file)
    setPreviewUrl(url)
    return () => URL.revokeObjectURL(url)
  }, [file])

  return (
    <div className="relative aspect-square overflow-hidden rounded-lg border border-slate-700 bg-slate-800">
      {previewUrl && <img src={previewUrl} alt={file.name} className="h-full w-full object-cover" />}
      <button
        type="button"
        onClick={onRemove}
        className="absolute right-1 top-1 h-5 w-5 rounded-full bg-slate-950/80 text-xs text-slate-200 hover:bg-red-700"
        aria-label={`Remove ${file.name}`}
      >
        ×
      </button>
    </div>
  )
}
