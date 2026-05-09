import { useEffect, useState, useCallback } from 'react'
import api from '../../lib/api'
import type { Contact } from '../../types'
import { Plus, Trash2, Search, Phone, Mail, MapPin } from 'lucide-react'

export default function Contacts() {
  const [contacts, setContacts] = useState<Contact[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [newName, setNewName] = useState('')
  const [newPhones, setNewPhones] = useState('')
  const [newEmails, setNewEmails] = useState('')
  const [newAddresses, setNewAddresses] = useState('')
  const [newNotes, setNewNotes] = useState('')

  const fetchContacts = useCallback(async () => {
    try {
      setLoading(true)
      const res = await api.get('/contacts')
      setContacts(res.data)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchContacts()
  }, [fetchContacts])

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newName.trim()) return
    try {
      await api.post('/contacts', {
        name: newName,
        phones: newPhones ? newPhones.split(',').map((s) => s.trim()) : [],
        emails: newEmails ? newEmails.split(',').map((s) => s.trim()) : [],
        addresses: newAddresses ? newAddresses.split(',').map((s) => s.trim()) : [],
        notes: newNotes,
      })
      setNewName('')
      setNewPhones('')
      setNewEmails('')
      setNewAddresses('')
      setNewNotes('')
      setShowCreate(false)
      fetchContacts()
    } catch (err: any) {
      alert('Failed to create: ' + err.message)
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this contact?')) return
    try {
      await api.delete(`/contacts/${id}`)
      fetchContacts()
    } catch (err: any) {
      alert('Failed to delete: ' + err.message)
    }
  }

  const filtered = contacts.filter(
      (c) =>
          c.name.toLowerCase().includes(search.toLowerCase()) ||
          c.notes.toLowerCase().includes(search.toLowerCase()) ||
          c.phones.some((p) => p.includes(search)) ||
          c.emails.some((e) => e.includes(search))
  )

  return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold dark:text-white">Contacts</h2>
          <button
              onClick={() => setShowCreate(!showCreate)}
              className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
          >
            <Plus size={18} />
            Add Contact
          </button>
        </div>

        <div className="relative mb-4">
          <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
          <input
              type="text"
              placeholder="Search contacts..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
          />
        </div>

        {showCreate && (
            <form onSubmit={handleCreate} className="mb-6 rounded bg-white p-4 shadow dark:bg-gray-800">
              <input
                  type="text"
                  placeholder="Name (required)"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  required
              />
              <input
                  type="text"
                  placeholder="Phone numbers (comma separated)"
                  value={newPhones}
                  onChange={(e) => setNewPhones(e.target.value)}
                  className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
              <input
                  type="text"
                  placeholder="Emails (comma separated)"
                  value={newEmails}
                  onChange={(e) => setNewEmails(e.target.value)}
                  className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
              <input
                  type="text"
                  placeholder="Addresses (comma separated)"
                  value={newAddresses}
                  onChange={(e) => setNewAddresses(e.target.value)}
                  className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
              <textarea
                  placeholder="Notes"
                  value={newNotes}
                  onChange={(e) => setNewNotes(e.target.value)}
                  rows={2}
                  className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
              <div className="flex gap-2">
                <button
                    type="submit"
                    className="rounded bg-green-600 px-4 py-2 text-white hover:bg-green-700"
                >
                  Save
                </button>
                <button
                    type="button"
                    onClick={() => setShowCreate(false)}
                    className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white"
                >
                  Cancel
                </button>
              </div>
            </form>
        )}

        {loading && <p className="text-gray-500">Loading...</p>}
        {error && <p className="text-red-500">Error: {error}</p>}
        {!loading && !error && filtered.length === 0 && (
            <p className="text-gray-500">No contacts found.</p>
        )}

        <div className="space-y-3">
          {filtered.map((c) => (
              <div
                  key={c.id}
                  className="rounded bg-white p-4 shadow dark:bg-gray-800 group flex items-start gap-3"
              >
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold dark:text-white text-lg">{c.name}</h3>
                  <div className="mt-2 space-y-1 text-sm text-gray-600 dark:text-gray-400">
                    {c.phones.map((phone, i) => (
                        <div key={i} className="flex items-center gap-1">
                          <Phone size={14} /> {phone}
                        </div>
                    ))}
                    {c.emails.map((email, i) => (
                        <div key={i} className="flex items-center gap-1">
                          <Mail size={14} /> {email}
                        </div>
                    ))}
                    {c.addresses.map((addr, i) => (
                        <div key={i} className="flex items-center gap-1">
                          <MapPin size={14} /> {addr}
                        </div>
                    ))}
                  </div>
                  {c.notes && (
                      <p className="text-gray-500 dark:text-gray-400 text-sm mt-2 italic">
                        {c.notes}
                      </p>
                  )}
                  <p className="text-xs text-gray-400 mt-2">
                    {new Date(c.updated_at * 1000).toLocaleString()}
                  </p>
                </div>
                <button
                    onClick={() => handleDelete(c.id)}
                    className="text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
                    title="Delete"
                >
                  <Trash2 size={18} />
                </button>
              </div>
          ))}
        </div>
      </div>
  )
}