// frontend/src/components/timeline/EditItemModal.tsx
// Create / edit dialog for a single timeline item. Sends a merged payload
// back to the parent, which applies it to the matching vault endpoint.
import { useEffect, useState } from 'react'
import { Loader2, Lock, X } from 'lucide-react'
import { KIND_LABELS } from '../../lib/timeline'
import type { TimelineItem, VaultKind } from '../../lib/timeline'

interface EditItemModalProps {
    mode: 'create' | 'edit'
    kind: VaultKind
    item: TimelineItem | null
    saving: boolean
    error: string | null
    onSave: (payload: Record<string, unknown>) => void
    onClose: () => void
}

const FIELD =
    'w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-900 dark:text-white'
const LABEL = 'mb-1 block text-xs font-medium text-gray-600 dark:text-gray-300'

function splitLines(value: string): string[] {
    return value.split('\n').map((line) => line.trim()).filter(Boolean)
}

export default function EditItemModal({ mode, kind, item, saving, error, onSave, onClose }: EditItemModalProps) {
    const raw = item?.raw ?? {}

    const [title, setTitle] = useState(String(raw.title ?? ''))
    const [content, setContent] = useState(String(raw.content ?? ''))
    const [description, setDescription] = useState(String(raw.description ?? ''))
    const [url, setUrl] = useState(String(raw.url ?? ''))
    const [name, setName] = useState(String(raw.name ?? ''))
    const [website, setWebsite] = useState(String(raw.website ?? ''))
    const [username, setUsername] = useState(String(raw.username ?? ''))
    const [phones, setPhones] = useState(Array.isArray(raw.phones) ? raw.phones.join('\n') : '')
    const [emails, setEmails] = useState(Array.isArray(raw.emails) ? raw.emails.join('\n') : '')
    const [notes, setNotes] = useState(String(raw.notes ?? ''))
    const [tags, setTags] = useState(Array.isArray(raw.tags) ? raw.tags.join(', ') : '')
    const [completed, setCompleted] = useState(Boolean(raw.completed))

    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && !saving) onClose()
        }
        document.addEventListener('keydown', handler)
        return () => document.removeEventListener('keydown', handler)
    }, [onClose, saving])

    const parsedTags = (): string[] => tags.split(',').map((tag) => tag.trim()).filter(Boolean)

    const canSave = (): boolean => {
        switch (kind) {
            case 'clipboard':
                return content.trim().length > 0
            case 'todo':
                return title.trim().length > 0
            case 'bookmark':
                return url.trim().length > 0
            case 'contact':
                return name.trim().length > 0
            case 'credential':
                return website.trim().length > 0 || username.trim().length > 0
            case 'note':
            default:
                return title.trim().length > 0 || content.trim().length > 0
        }
    }

    const buildPayload = (): Record<string, unknown> => {
        switch (kind) {
            case 'clipboard':
                return { content, tags: parsedTags() }
            case 'todo':
                return { title: title.trim(), description, completed, tags: parsedTags() }
            case 'bookmark':
                return { url: url.trim(), title: title.trim(), description, tags: parsedTags() }
            case 'contact':
                return { name: name.trim(), phones: splitLines(phones), emails: splitLines(emails), notes, tags: parsedTags() }
            case 'credential':
                return { website: website.trim(), url: url.trim(), username: username.trim(), tags: parsedTags() }
            case 'note':
            default:
                return { title: title.trim() || 'Untitled note', content, tags: parsedTags() }
        }
    }

    const submit = () => {
        if (!saving && canSave()) onSave(buildPayload())
    }

    return (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
            onClick={() => !saving && onClose()}
        >
            <div
                className="flex max-h-[90vh] w-full max-w-lg flex-col rounded-lg bg-white shadow-xl dark:bg-gray-800"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between border-b border-gray-200 p-4 dark:border-gray-700">
                    <h3 className="text-lg font-semibold dark:text-white">
                        {mode === 'create' ? `New ${KIND_LABELS[kind]}` : `Edit ${KIND_LABELS[kind]}`}
                    </h3>
                    <button
                        onClick={() => !saving && onClose()}
                        className="rounded p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                        <X size={20} />
                    </button>
                </div>

                <div className="space-y-4 overflow-y-auto p-5">
                    {(kind === 'note' || kind === 'todo' || kind === 'bookmark') && (
                        <div>
                            <label className={LABEL}>Title</label>
                            <input
                                dir="auto"
                                className={FIELD}
                                value={title}
                                onChange={(e) => setTitle(e.target.value)}
                                placeholder={kind === 'bookmark' ? 'Bookmark title' : 'Title'}
                            />
                        </div>
                    )}

                    {kind === 'note' && (
                        <div>
                            <label className={LABEL}>Content</label>
                            <textarea
                                dir="auto"
                                rows={7}
                                className={FIELD}
                                value={content}
                                onChange={(e) => setContent(e.target.value)}
                                placeholder="Write your note…"
                            />
                        </div>
                    )}

                    {kind === 'clipboard' && (
                        <div>
                            <label className={LABEL}>Content</label>
                            <textarea
                                dir="auto"
                                rows={8}
                                className={`${FIELD} font-mono`}
                                value={content}
                                onChange={(e) => setContent(e.target.value)}
                                placeholder="Clipboard text…"
                            />
                        </div>
                    )}

                    {kind === 'todo' && (
                        <>
                            <div className="flex items-center gap-2">
                                <input
                                    id="todo-completed"
                                    type="checkbox"
                                    checked={completed}
                                    onChange={(e) => setCompleted(e.target.checked)}
                                    className="h-4 w-4 rounded border-gray-300 accent-emerald-600"
                                />
                                <label htmlFor="todo-completed" className="text-sm text-gray-700 dark:text-gray-300">
                                    Completed
                                </label>
                            </div>
                            <div>
                                <label className={LABEL}>Description</label>
                                <textarea
                                    dir="auto"
                                    rows={4}
                                    className={FIELD}
                                    value={description}
                                    onChange={(e) => setDescription(e.target.value)}
                                    placeholder="Optional details…"
                                />
                            </div>
                        </>
                    )}

                    {(kind === 'bookmark' || kind === 'credential') && (
                        <div>
                            <label className={LABEL}>URL</label>
                            <input
                                dir="ltr"
                                className={FIELD}
                                value={url}
                                onChange={(e) => setUrl(e.target.value)}
                                placeholder="https://…"
                            />
                        </div>
                    )}

                    {kind === 'bookmark' && (
                        <div>
                            <label className={LABEL}>Description</label>
                            <textarea
                                dir="auto"
                                rows={3}
                                className={FIELD}
                                value={description}
                                onChange={(e) => setDescription(e.target.value)}
                                placeholder="Optional description…"
                            />
                        </div>
                    )}

                    {kind === 'credential' && (
                        <>
                            <div>
                                <label className={LABEL}>Website</label>
                                <input
                                    dir="auto"
                                    className={FIELD}
                                    value={website}
                                    onChange={(e) => setWebsite(e.target.value)}
                                    placeholder="e.g. GitHub"
                                />
                            </div>
                            <div>
                                <label className={LABEL}>Username</label>
                                <input
                                    dir="auto"
                                    className={FIELD}
                                    value={username}
                                    onChange={(e) => setUsername(e.target.value)}
                                />
                            </div>
                            <div className="flex items-start gap-2 rounded-lg bg-amber-50 p-3 text-xs text-amber-700 dark:bg-amber-900/30 dark:text-amber-300">
                                <Lock size={14} className="mt-0.5 shrink-0" />
                                <span>Passwords and secrets are encrypted — manage them from the Credentials vault.</span>
                            </div>
                        </>
                    )}

                    {kind === 'contact' && (
                        <>
                            <div>
                                <label className={LABEL}>Name</label>
                                <input dir="auto" className={FIELD} value={name} onChange={(e) => setName(e.target.value)} />
                            </div>
                            <div>
                                <label className={LABEL}>Phones (one per line)</label>
                                <textarea rows={3} className={FIELD} value={phones} onChange={(e) => setPhones(e.target.value)} />
                            </div>
                            <div>
                                <label className={LABEL}>Emails (one per line)</label>
                                <textarea rows={3} className={FIELD} value={emails} onChange={(e) => setEmails(e.target.value)} />
                            </div>
                            <div>
                                <label className={LABEL}>Notes</label>
                                <textarea dir="auto" rows={3} className={FIELD} value={notes} onChange={(e) => setNotes(e.target.value)} />
                            </div>
                        </>
                    )}

                    <div>
                        <label className={LABEL}>Tags (comma separated)</label>
                        <input
                            dir="auto"
                            className={FIELD}
                            value={tags}
                            onChange={(e) => setTags(e.target.value)}
                            placeholder="personal, work"
                        />
                    </div>
                </div>

                {error && (
                    <div className="border-t border-gray-200 px-5 py-3 text-sm text-red-600 dark:border-gray-700 dark:text-red-400">
                        {error}
                    </div>
                )}

                <div className="flex justify-end gap-3 border-t border-gray-200 p-4 dark:border-gray-700">
                    <button
                        onClick={() => !saving && onClose()}
                        className="rounded border border-gray-300 bg-white px-4 py-2 text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
                    >
                        Cancel
                    </button>
                    <button
                        onClick={submit}
                        disabled={saving || !canSave()}
                        className="inline-flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                        {saving && <Loader2 size={16} className="animate-spin" />}
                        {mode === 'create' ? 'Create' : 'Save changes'}
                    </button>
                </div>
            </div>
        </div>
    )
}
