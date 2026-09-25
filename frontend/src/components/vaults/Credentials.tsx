import { useEffect, useState, useCallback } from 'react'
import api, { reorderVault } from '../../lib/api'
import type { Credential } from '../../types'
import { Trash2, Lock, Eye, EyeOff, Copy, Check, ChevronUp, ChevronDown } from 'lucide-react'
import { calculateFractionalPosition } from '../../lib/reorder'
import AutoDirText from '../AutoDirText'
import LoadingSpinner from '../LoadingSpinner'
import { useConfirmation } from '../../context/ConfirmationContext'
import { useViewStore } from '../../stores/viewStore'
import VaultLayout from './VaultLayout'

export default function Credentials() {
    const { confirm } = useConfirmation()
    const [masterPassword, setMasterPassword] = useState('')
    const [unlockError, setUnlockError] = useState('')
    const [unlocked, setUnlocked] = useState(false)
    const [firstTime, setFirstTime] = useState<boolean | null>(null)

    useEffect(() => {
        api.get('/credentials/status')
            .then(res => setFirstTime(!res.data.configured))
            .catch(() => setFirstTime(false))
    }, [])

    const [credentials, setCredentials] = useState<Credential[]>([])
    const [loading, setLoading] = useState(false)
    const [search, setSearch] = useState('')

    const [showCreate, setShowCreate] = useState(false)
    const [newWebsite, setNewWebsite] = useState('')
    const [newUrl, setNewUrl] = useState('')
    const [newUsername, setNewUsername] = useState('')
    const [newPassword, setNewPassword] = useState('')
    const [newNotes, setNewNotes] = useState('')
    const [newTotp, setNewTotp] = useState('')

    const [selectedId, setSelectedId] = useState<string | null>(null)
    const [detail, setDetail] = useState<Credential | null>(null)
    const [detailLoading, setDetailLoading] = useState(false)
    const [showPassword, setShowPassword] = useState(false)
    const [copiedField, setCopiedField] = useState<string | null>(null)
    const [editOrder, setEditOrder] = useState(false)
    const view = useViewStore((s) => s.views.credentials || 'list')
    const [editing, setEditing] = useState(false)
    const [editForm, setEditForm] = useState({
        website: '',
        url: '',
        username: '',
        password: '',
        notes: '',
        totp_secret: '',
    })

    const fetchCredentials = useCallback(async () => {
        try {
            setLoading(true)
            const res = await api.get('/credentials')
            setCredentials(res.data)
        } catch (err) {
            console.error('Failed to fetch credentials', err)
        } finally {
            setLoading(false)
        }
    }, [])

    const handleUnlock = async (e: React.FormEvent) => {
        e.preventDefault()
        setUnlockError('')
        try {
            const res = await api.post('/credentials/unlock', { master_password: masterPassword })
            if (res.data.status === 'master_password_set') {
                setFirstTime(false)
            }
            setUnlocked(true)
            fetchCredentials()
        } catch (err: any) {
            setUnlockError(err.response?.data?.error || 'Unlock failed')
        }
    }

    const handleLock = async () => {
        await api.post('/credentials/lock')
        setUnlocked(false)
        setSelectedId(null)
        setDetail(null)
        setMasterPassword('')
    }

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!newWebsite.trim()) return
        try {
            await api.post('/credentials', {
                website: newWebsite,
                url: newUrl,
                username: newUsername,
                password: newPassword || undefined,
                notes: newNotes || undefined,
                totp_secret: newTotp || undefined,
            })
            setNewWebsite(''); setNewUrl(''); setNewUsername('')
            setNewPassword(''); setNewNotes(''); setNewTotp('')
            setShowCreate(false)
            fetchCredentials()
        } catch (err: any) {
            alert('Failed to create: ' + (err.response?.data?.error || err.message))
        }
    }

    const handleDelete = async (id: string) => {
        const ok = await confirm('Delete this credential?')
        if (!ok) return
        try {
            await api.delete(`/credentials/${id}`)
            if (selectedId === id) {
                setSelectedId(null)
                setDetail(null)
            }
            fetchCredentials()
        } catch (err: any) {
            alert('Failed to delete: ' + (err.response?.data?.error || err.message))
        }
    }

    const moveCredential = async (index: number, direction: 'up' | 'down') => {
        const newPos = calculateFractionalPosition(filtered, index, direction)
        if (newPos === null) return

        const targetItem = filtered[index]
        const updatedCreds = credentials.map((c) => (c.id === targetItem.id ? { ...c, position: newPos } : c))
        updatedCreds.sort((a, b) => a.position - b.position)
        setCredentials(updatedCreds)

        try {
            await reorderVault('credentials', [{ id: targetItem.id, position: newPos }])
        } catch (err: any) {
            fetchCredentials()
            alert('Failed to reorder: ' + (err.response?.data?.error || err.message))
        }
    }

    const fetchDetail = async (id: string) => {
        setSelectedId(id)
        setDetailLoading(true)
        setEditing(false)
        try {
            const res = await api.get(`/credentials/${id}`)
            setDetail(res.data)
            setEditForm({
                website: res.data.website || '',
                url: res.data.url || '',
                username: res.data.username || '',
                password: '',
                notes: res.data.notes_plain || '',
                totp_secret: res.data.totp_secret_plain || '',
            })
        } catch (err: any) {
            alert('Failed to load credential: ' + (err.response?.data?.error || err.message))
            setSelectedId(null)
        } finally {
            setDetailLoading(false)
        }
    }

    const handleUpdate = async () => {
        if (!detail) return
        try {
            const payload: any = {}
            if (editForm.website !== detail.website) payload.website = editForm.website
            if (editForm.url !== detail.url) payload.url = editForm.url
            if (editForm.username !== detail.username) payload.username = editForm.username
            if (editForm.password) payload.password = editForm.password
            if (editForm.notes !== (detail.notes_plain || '')) payload.notes = editForm.notes
            if (editForm.totp_secret !== (detail.totp_secret_plain || '')) payload.totp_secret = editForm.totp_secret

            await api.put(`/credentials/${detail.id}`, payload)
            setEditing(false)
            fetchDetail(detail.id)
            fetchCredentials()
        } catch (err: any) {
            alert('Failed to update: ' + (err.response?.data?.error || err.message))
        }
    }

    const copyToClipboard = async (text: string, field: string) => {
        try {
            await navigator.clipboard.writeText(text)
            setCopiedField(field)
            setTimeout(() => setCopiedField(null), 2000)
        } catch {
            const textarea = document.createElement('textarea')
            textarea.value = text
            document.body.appendChild(textarea)
            textarea.select()
            document.execCommand('copy')
            document.body.removeChild(textarea)
            setCopiedField(field)
            setTimeout(() => setCopiedField(null), 2000)
        }
    }

    const filtered = credentials.filter(c =>
        c.website.toLowerCase().includes(search.toLowerCase()) ||
        c.username.toLowerCase().includes(search.toLowerCase())
    )

    if (!unlocked) {
        if (firstTime === null) {
            return (
                <div className="max-w-md mx-auto mt-20">
                    <LoadingSpinner message="Checking vault..." />
                </div>
            )
        }

        return (
            <div className="max-w-md mx-auto mt-20">
                <div className="bg-white dark:bg-gray-800 rounded shadow p-6 text-center">
                    <Lock size={48} className="mx-auto mb-4 text-gray-400" />
                    <h2 className="text-xl font-bold dark:text-white mb-2">Credentials Vault</h2>
                    <p className="text-gray-500 dark:text-gray-400 mb-4">
                        {firstTime
                            ? 'First time here? Choose a strong master password to protect your credentials.'
                            : 'Enter your master password to unlock.'}
                    </p>
                    <form onSubmit={handleUnlock}>
                        <input
                            type="password"
                            placeholder={firstTime ? 'New Master Password' : 'Master Password'}
                            value={masterPassword}
                            onChange={e => setMasterPassword(e.target.value)}
                            className="w-full rounded border p-2 mb-3 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                            autoFocus
                        />
                        {unlockError && <p className="text-red-500 text-sm mb-3">{unlockError}</p>}
                        <button
                            type="submit"
                            className="w-full rounded bg-blue-600 py-2 text-white hover:bg-blue-700"
                        >
                            {firstTime ? 'Set Master Password' : 'Unlock'}
                        </button>
                    </form>
                </div>
            </div>
        )
    }

    const renderCredential = (cred: Credential, index: number) => (
        <div
            key={cred.id}
            className={`rounded bg-white p-3 shadow dark:bg-gray-800 flex items-center justify-between group cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 ${
                view === 'grid' ? 'h-24 overflow-hidden' : ''
            }`}
            onClick={() => fetchDetail(cred.id)}
        >
            <div className="flex-1 min-w-0">
                <div className="font-semibold dark:text-white truncate">{cred.website}</div>
                <div className="text-sm text-gray-500 dark:text-gray-400">{cred.username}</div>
            </div>
            <div className="flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
                {editOrder && (
                    <div className="flex flex-col gap-0.5">
                        <button
                            onClick={(e) => { e.stopPropagation(); moveCredential(index, 'up') }}
                            disabled={index === 0}
                            className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                            title="Move up"
                        >
                            <ChevronUp size={18} />
                        </button>
                        <button
                            onClick={(e) => { e.stopPropagation(); moveCredential(index, 'down') }}
                            disabled={index === filtered.length - 1}
                            className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                            title="Move down"
                        >
                            <ChevronDown size={18} />
                        </button>
                    </div>
                )}
                <button
                    onClick={(e) => { e.stopPropagation(); handleDelete(cred.id) }}
                    className="text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity p-1"
                >
                    <Trash2 size={20} />
                </button>
            </div>
        </div>
    )

    return (
        <VaultLayout
            title="Credentials"
            vaultKey="credentials"
            search={search}
            onSearchChange={setSearch}
            editOrder={editOrder}
            onToggleEditOrder={() => setEditOrder(!editOrder)}
            onAdd={() => setShowCreate(!showCreate)}
            addLabel="Add"
            extraActions={
                <button
                    type="button"
                    onClick={handleLock}
                    className="flex items-center gap-2 rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white hover:bg-gray-400 dark:hover:bg-gray-500"
                >
                    <Lock size={18} /> Lock
                </button>
            }
            loading={loading}
            items={filtered}
            renderItem={renderCredential}
            createForm={showCreate && (
                <form onSubmit={handleCreate} className="mb-6 rounded bg-white p-4 shadow dark:bg-gray-800">
                    <input
                        type="text"
                        placeholder="Website (required)"
                        value={newWebsite}
                        onChange={e => setNewWebsite(e.target.value)}
                        required
                        className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                    <input
                        type="text"
                        placeholder="URL (optional)"
                        value={newUrl}
                        onChange={e => setNewUrl(e.target.value)}
                        className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                    <input
                        type="text"
                        placeholder="Username"
                        value={newUsername}
                        onChange={e => setNewUsername(e.target.value)}
                        className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                    <input
                        type="password"
                        placeholder="Password"
                        value={newPassword}
                        onChange={e => setNewPassword(e.target.value)}
                        className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                    <textarea
                        placeholder="Notes"
                        value={newNotes}
                        onChange={e => setNewNotes(e.target.value)}
                        rows={2}
                        className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                    <input
                        type="text"
                        placeholder="TOTP Secret (optional)"
                        value={newTotp}
                        onChange={e => setNewTotp(e.target.value)}
                        className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    />
                    <div className="flex gap-2">
                        <button type="submit" className="rounded bg-green-600 px-4 py-2 text-white">Save</button>
                        <button type="button" onClick={() => setShowCreate(false)} className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white">Cancel</button>
                    </div>
                </form>
            )}
            detailModal={selectedId && (
                <div className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center" onClick={() => { setSelectedId(null); setDetail(null) }}>
                    <div className="bg-white dark:bg-gray-800 rounded shadow-lg p-6 w-full max-w-md max-h-[80vh] overflow-y-auto" onClick={e => e.stopPropagation()}>
                        {detailLoading ? (
                            <LoadingSpinner message="Decrypting..." />
                        ) : detail ? (
                            <>
                                <h3 className="text-xl font-bold dark:text-white mb-4">
                                    {editing ? (
                                        <input
                                            type="text"
                                            value={editForm.website}
                                            onChange={e => setEditForm({...editForm, website: e.target.value})}
                                            className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                        />
                                    ) : (
                                        detail.website
                                    )}
                                </h3>
                                {editing ? (
                                    <div className="space-y-3">
                                        <input type="text" placeholder="URL" value={editForm.url} onChange={e => setEditForm({...editForm, url: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                                        <input type="text" placeholder="Username" value={editForm.username} onChange={e => setEditForm({...editForm, username: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                                        <input type="password" placeholder="New password (leave blank to keep)" value={editForm.password} onChange={e => setEditForm({...editForm, password: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                                        <textarea placeholder="Notes" value={editForm.notes} onChange={e => setEditForm({...editForm, notes: e.target.value})} rows={2} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                                        <input type="text" placeholder="TOTP Secret" value={editForm.totp_secret} onChange={e => setEditForm({...editForm, totp_secret: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                                        <div className="flex gap-2">
                                            <button onClick={handleUpdate} className="rounded bg-green-600 px-4 py-2 text-white">Save</button>
                                            <button onClick={() => setEditing(false)} className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white">Cancel</button>
                                        </div>
                                    </div>
                                ) : (
                                    <>
                                        <div className="space-y-3">
                                            <div>
                                                <label className="text-xs text-gray-500 uppercase">URL</label>
                                                <p className="dark:text-white">{detail.url || '—'}</p>
                                            </div>
                                            <div>
                                                <label className="text-xs text-gray-500 uppercase">Username</label>
                                                <div className="flex items-center gap-2">
                                                    <span className="dark:text-white">{detail.username}</span>
                                                    <button onClick={() => copyToClipboard(detail.username, 'username')} className="text-gray-400 hover:text-blue-500 p-1">
                                                        {copiedField === 'username' ? <Check size={20} className="text-green-500" /> : <Copy size={20} />}
                                                    </button>
                                                </div>
                                            </div>
                                            {detail.password_plain && (
                                                <div>
                                                    <label className="text-xs text-gray-500 uppercase">Password</label>
                                                    <div className="flex items-center gap-2">
                                                        <span className="dark:text-white font-mono">{showPassword ? detail.password_plain : '••••••••'}</span>
                                                        <button onClick={() => setShowPassword(!showPassword)} className="text-gray-400 hover:text-blue-500 p-1">
                                                            {showPassword ? <EyeOff size={20} /> : <Eye size={20} />}
                                                        </button>
                                                        <button onClick={() => copyToClipboard(detail.password_plain!, 'password')} className="text-gray-400 hover:text-blue-500 p-1">
                                                            {copiedField === 'password' ? <Check size={20} className="text-green-500" /> : <Copy size={20} />}
                                                        </button>
                                                    </div>
                                                </div>
                                            )}
                                            {detail.notes_plain && (
                                                <div>
                                                    <label className="text-xs text-gray-500 uppercase">Notes</label>
                                                    <AutoDirText text={detail.notes_plain || ''} as="p" className="dark:text-white whitespace-pre-wrap" />
                                                </div>
                                            )}
                                            {detail.totp_secret_plain && (
                                                <div>
                                                    <label className="text-xs text-gray-500 uppercase">TOTP Secret</label>
                                                    <div className="flex items-center gap-2">
                                                        <AutoDirText text={showPassword ? detail.totp_secret_plain || '' : '••••••••'} as="span" className="dark:text-white font-mono" />
                                                        <button onClick={() => setShowPassword(!showPassword)} className="text-gray-400 hover:text-blue-500 p-1">
                                                            {showPassword ? <EyeOff size={20} /> : <Eye size={20} />}
                                                        </button>
                                                        <button onClick={() => copyToClipboard(detail.totp_secret_plain!, 'totp')} className="text-gray-400 hover:text-blue-500 p-1">
                                                            {copiedField === 'totp' ? <Check size={20} className="text-green-500" /> : <Copy size={20} />}
                                                        </button>
                                                    </div>
                                                </div>
                                            )}
                                        </div>
                                        <div className="flex gap-2 mt-4">
                                            <button onClick={() => setEditing(true)} className="flex-1 rounded bg-blue-600 py-2 text-white">Edit</button>
                                            <button onClick={() => { setSelectedId(null); setDetail(null) }} className="flex-1 rounded bg-gray-300 py-2 dark:bg-gray-600 dark:text-white">Close</button>
                                        </div>
                                    </>
                                )}
                            </>
                        ) : (
                            <p className="text-red-500">Failed to load details.</p>
                        )}
                    </div>
                </div>
            )}
        />
    )
}
