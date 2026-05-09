import { useEffect, useState } from 'react'
import api from '../../lib/api'
import type { Note } from '../../types'

export default function Notes() {
    const [notes, setNotes] = useState<Note[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)

    useEffect(() => {
        api.get('/notes')
            .then((res) => { setNotes(res.data); setLoading(false) })
            .catch((err) => { setError(err.message); setLoading(false) })
    }, [])

    if (loading) return <div className="text-gray-500">Loading...</div>
    if (error) return <div className="text-red-500">Error: {error}</div>

    return (
        <div>
            <h2 className="mb-4 text-2xl font-bold dark:text-white">Notes</h2>
            {notes.length === 0 && (
                <p className="text-gray-500">No notes yet. Create one!</p>
            )}
            <ul>
                {notes.map((note) => (
                    <li key={note.id} className="mb-2 rounded bg-white p-3 shadow dark:bg-gray-800">
                        <h3 className="font-semibold dark:text-white">{note.title}</h3>
                        <p className="text-gray-600 dark:text-gray-400">{note.content}</p>
                    </li>
                ))}
            </ul>
        </div>
    )
}