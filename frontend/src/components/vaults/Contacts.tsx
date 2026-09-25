import { useEffect, useState, useCallback } from 'react'
import api, { reorderVault } from '../../lib/api'
import type { Contact } from '../../types'
import { Plus, Trash2, Search, Phone, Mail, MapPin, Edit3, ChevronUp, ChevronDown, X } from 'lucide-react'
import { calculateFractionalPosition } from '../../lib/reorder'
import AutoDirText from "../AutoDirText.tsx";
import LoadingSpinner from "../LoadingSpinner.tsx";
import {useConfirmation} from "../../context/ConfirmationContext.tsx";
import { useViewStore } from '../../stores/viewStore'
import ViewSwitcher from '../ViewSwitcher'
import ListView from '../views/ListView'
import GridView from '../views/GridView'
import CompactListView from '../views/CompactListView'

export default function Contacts() {
  const { confirm } = useConfirmation()
  const [contacts, setContacts] = useState<Contact[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editForm, setEditForm] = useState({
    name: '',
    phones: '',
    emails: '',
    addresses: '',
    notes: '',
  })
  const [newName, setNewName] = useState('')
  const [newPhones, setNewPhones] = useState('')
  const [newEmails, setNewEmails] = useState('')
  const [newAddresses, setNewAddresses] = useState('')
  const [newNotes, setNewNotes] = useState('')
  const [editOrder, setEditOrder] = useState(false)
  const view = useViewStore((s) => s.views.contacts || 'list')
  const [selectedContact, setSelectedContact] = useState<Contact | null>(null)
  const [editingInModal, setEditingInModal] = useState(false)

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

  useEffect(() => { fetchContacts() }, [fetchContacts])

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newName.trim()) return
    try {
      await api.post('/contacts', {
        name: newName,
        phones: newPhones ? newPhones.split(',').map(s => s.trim()) : [],
        emails: newEmails ? newEmails.split(',').map(s => s.trim()) : [],
        addresses: newAddresses ? newAddresses.split(',').map(s => s.trim()) : [],
        notes: newNotes,
      })
      setNewName(''); setNewPhones(''); setNewEmails(''); setNewAddresses(''); setNewNotes(''); setShowCreate(false)
      fetchContacts()
    } catch (err: any) { alert('Failed to create: ' + err.message) }
  }

  const startEdit = (c: Contact) => {
    setEditingId(c.id)
    setEditForm({
      name: c.name,
      phones: c.phones.join(', '),
      emails: c.emails.join(', '),
      addresses: c.addresses.join(', '),
      notes: c.notes,
    })
  }

  const startEditInModal = (c: Contact) => {
    setSelectedContact(c)
    setEditForm({
      name: c.name,
      phones: c.phones.join(', '),
      emails: c.emails.join(', '),
      addresses: c.addresses.join(', '),
      notes: c.notes,
    })
    setEditingInModal(true)
  }

  const cancelEdit = () => setEditingId(null)

  const handleUpdate = async (id: string) => {
    try {
      await api.put(`/contacts/${id}`, {
        name: editForm.name,
        phones: editForm.phones ? editForm.phones.split(',').map(s => s.trim()) : [],
        emails: editForm.emails ? editForm.emails.split(',').map(s => s.trim()) : [],
        addresses: editForm.addresses ? editForm.addresses.split(',').map(s => s.trim()) : [],
        notes: editForm.notes,
      })
      setEditingId(null)
      fetchContacts()
    } catch (err: any) { alert('Failed to update: ' + err.message) }
  }

  const handleDelete = async (id: string) => {
    const ok = await confirm('Delete this contact?')
    if (!ok) return
    try {
      await api.delete(`/contacts/${id}`)
      fetchContacts()
    } catch (err: any) { alert('Failed to delete: ' + err.message) }
  }

  const moveContact = async (index: number, direction: 'up' | 'down') => {
    const newPos = calculateFractionalPosition(filtered, index, direction)
    if (newPos === null) return

    const targetItem = filtered[index]
    const updatedContacts = contacts.map((c) => (c.id === targetItem.id ? { ...c, position: newPos } : c))
    updatedContacts.sort((a, b) => a.position - b.position)
    setContacts(updatedContacts)

    try {
      await reorderVault('contacts', [{ id: targetItem.id, position: newPos }])
    } catch (err: any) {
      fetchContacts()
      alert('Failed to reorder: ' + (err.response?.data?.error || err.message))
    }
  }

  const filtered = contacts.filter(c =>
      c.name.toLowerCase().includes(search.toLowerCase()) ||
      c.notes.toLowerCase().includes(search.toLowerCase()) ||
      c.phones.some(p => p.includes(search)) ||
      c.emails.some(e => e.includes(search))
  )

  const getPlainTextSnippet = (text: string, maxLen = 100) => {
    if (!text) return ''
    if (text.length <= maxLen) return text
    return text.slice(0, maxLen).trimEnd() + '…'
  }

  const renderContact = (c: Contact, index: number) => (
      <div
          key={c.id}
          className={`rounded bg-white p-4 shadow dark:bg-gray-800 group flex items-start gap-3 ${
              view === 'grid' ? 'h-48 overflow-hidden cursor-pointer' : ''
          }`}
          onClick={() => view === 'grid' && setSelectedContact(c)}
      >
        {editingId === c.id ? (
            <div className="flex-1 space-y-2" onClick={(e) => e.stopPropagation()}>
              <input type="text" value={editForm.name} onChange={e => setEditForm({...editForm, name: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" placeholder="Name" />
              <input type="text" value={editForm.phones} onChange={e => setEditForm({...editForm, phones: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" placeholder="Phones (comma separated)" />
              <input type="text" value={editForm.emails} onChange={e => setEditForm({...editForm, emails: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" placeholder="Emails (comma separated)" />
              <input type="text" value={editForm.addresses} onChange={e => setEditForm({...editForm, addresses: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" placeholder="Addresses (comma separated)" />
              <textarea value={editForm.notes} onChange={e => setEditForm({...editForm, notes: e.target.value})} rows={2} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" placeholder="Notes" />
              <div className="flex gap-2">
                <button onClick={(e) => { e.stopPropagation(); handleUpdate(c.id); }} className="rounded bg-green-600 px-3 py-1 text-white">Save</button>
                <button onClick={(e) => { e.stopPropagation(); cancelEdit(); }} className="rounded bg-gray-300 px-3 py-1 dark:bg-gray-600 dark:text-white">Cancel</button>
              </div>
            </div>
        ) : (
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <h3 className="font-semibold dark:text-white text-lg">{c.name}</h3>
                <div className="flex items-center gap-1">
                  {editOrder && (
                      <div className="flex flex-col gap-0.5">
                        <button
                            onClick={(e) => { e.stopPropagation(); moveContact(index, 'up'); }}
                            disabled={index === 0}
                            className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                            title="Move up"
                        >
                          <ChevronUp size={18} />
                        </button>
                        <button
                            onClick={(e) => { e.stopPropagation(); moveContact(index, 'down'); }}
                            disabled={index === filtered.length - 1}
                            className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                            title="Move down"
                        >
                          <ChevronDown size={18} />
                        </button>
                      </div>
                  )}
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button onClick={(e) => {
                      e.stopPropagation();
                      if (view === 'grid') {
                        startEditInModal(c);
                      } else {
                        startEdit(c);
                      }
                    }} className="text-gray-400 hover:text-blue-500 p-1"><Edit3 size={20} /></button>
                    <button onClick={(e) => { e.stopPropagation(); handleDelete(c.id); }} className="text-gray-400 hover:text-red-500 p-1"><Trash2 size={20} /></button>
                  </div>
                </div>
              </div>
              <div className="mt-2 space-y-1 text-sm text-gray-600 dark:text-gray-400">
                {view === 'grid' ? (
                    <p className="line-clamp-2">{getPlainTextSnippet(c.phones.join(', ') + ' ' + c.emails.join(', '))}</p>
                ) : (
                    <>
                      {c.phones.map((phone, i) => <div key={i} className="flex items-center gap-1"><Phone size={16} /> {phone}</div>)}
                      {c.emails.map((email, i) => <div key={i} className="flex items-center gap-1"><Mail size={16} /> {email}</div>)}
                      {c.addresses.map((addr, i) => <div key={i} className="flex items-center gap-1"><MapPin size={16} /> {addr}</div>)}
                    </>
                )}
              </div>
              {c.notes && view !== 'grid' && <AutoDirText text={c.notes} as="p" className="text-gray-500 dark:text-gray-400 text-sm mt-2 italic" />}
              <p className="text-xs text-gray-400 mt-2">{new Date(c.updated_at * 1000).toLocaleString()}</p>
            </div>
        )}
      </div>
  );

  return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold dark:text-white">Contacts</h2>
          <div className="flex gap-2 items-center">
            <ViewSwitcher vaultKey="contacts" />
            <button
                onClick={() => setEditOrder(!editOrder)}
                className={`flex items-center gap-2 rounded px-4 py-2 ${
                    editOrder ? 'bg-green-600 hover:bg-green-700 text-white' : 'bg-gray-200 dark:bg-gray-700 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600'
                }`}
            >
              {editOrder ? 'Done' : 'Edit Order'}
            </button>
            <button onClick={() => setShowCreate(!showCreate)} className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700">
              <Plus size={18} /> Add Contact
            </button>
          </div>
        </div>

        <div className="relative mb-4">
          <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
          <input type="text" placeholder="Search contacts..." value={search} onChange={(e) => setSearch(e.target.value)} className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
        </div>

        {showCreate && (
            <form onSubmit={handleCreate} className="mb-6 rounded bg-white p-4 shadow dark:bg-gray-800">
              <input type="text" placeholder="Name (required)" value={newName} onChange={(e) => setNewName(e.target.value)} required className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <input type="text" placeholder="Phone numbers (comma separated)" value={newPhones} onChange={(e) => setNewPhones(e.target.value)} className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <input type="text" placeholder="Emails (comma separated)" value={newEmails} onChange={(e) => setNewEmails(e.target.value)} className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <input type="text" placeholder="Addresses (comma separated)" value={newAddresses} onChange={(e) => setNewAddresses(e.target.value)} className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <textarea placeholder="Notes" value={newNotes} onChange={(e) => setNewNotes(e.target.value)} rows={2} className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <div className="flex gap-2">
                <button type="submit" className="rounded bg-green-600 px-4 py-2 text-white hover:bg-green-700">Save</button>
                <button type="button" onClick={() => setShowCreate(false)} className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white">Cancel</button>
              </div>
            </form>
        )}

        {loading && <LoadingSpinner message="Loading contacts..." />}
        {error && <p className="text-red-500">Error: {error}</p>}
        {!loading && !error && filtered.length === 0 && <p className="text-gray-500">No contacts found.</p>}

        {view === 'list' && <ListView items={filtered} renderItem={renderContact} />}
        {view === 'grid' && <GridView items={filtered} renderItem={renderContact} />}
        {view === 'compact' && <CompactListView items={filtered} renderItem={renderContact} />}


        {/* Detail modal */}
        {selectedContact && (
            <div
                className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4"
                onClick={() => { setSelectedContact(null); setEditingInModal(false); }}
            >
              <div
                  className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl max-h-[85vh] overflow-y-auto"
                  onClick={(e) => e.stopPropagation()}
              >
                {editingInModal ? (
                    <>
                      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                        <h3 className="text-xl font-bold dark:text-white">Edit Contact</h3>
                        <button type="button" onClick={() => { setEditingInModal(false); setSelectedContact(null); }} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"><X size={20} /></button>
                      </div>
                      <div className="p-6 space-y-3">
                        <input type="text" placeholder="Name" value={editForm.name} onChange={e => setEditForm({...editForm, name: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                        <input type="text" placeholder="Phones (comma separated)" value={editForm.phones} onChange={e => setEditForm({...editForm, phones: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                        <input type="text" placeholder="Emails (comma separated)" value={editForm.emails} onChange={e => setEditForm({...editForm, emails: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                        <input type="text" placeholder="Addresses (comma separated)" value={editForm.addresses} onChange={e => setEditForm({...editForm, addresses: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                        <textarea placeholder="Notes" value={editForm.notes} onChange={e => setEditForm({...editForm, notes: e.target.value})} rows={4} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                        <div className="flex gap-2">
                          <button onClick={() => { handleUpdate(selectedContact.id); setEditingInModal(false); setSelectedContact(null); }} className="rounded bg-green-600 px-4 py-2 text-white">Save</button>
                          <button onClick={() => { setSelectedContact(null); setEditingInModal(false); }} className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white">Cancel</button>
                        </div>
                      </div>
                    </>
                ) : (
                    <>
                      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                        <h3 className="text-xl font-bold dark:text-white">{selectedContact.name}</h3>
                        <button onClick={() => setSelectedContact(null)} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"><X size={20} /></button>
                      </div>
                      <div className="p-6 space-y-3">
                        <div>
                          <label className="text-xs text-gray-500 uppercase">Phones</label>
                          {selectedContact.phones.length > 0 ? selectedContact.phones.map((p,i) => <div key={i} className="flex items-center gap-1 dark:text-white"><Phone size={16}/> {p}</div>) : <p className="text-gray-400">None</p>}
                        </div>
                        <div>
                          <label className="text-xs text-gray-500 uppercase">Emails</label>
                          {selectedContact.emails.length > 0 ? selectedContact.emails.map((e,i) => <div key={i} className="flex items-center gap-1 dark:text-white"><Mail size={16}/> {e}</div>) : <p className="text-gray-400">None</p>}
                        </div>
                        <div>
                          <label className="text-xs text-gray-500 uppercase">Addresses</label>
                          {selectedContact.addresses.length > 0 ? selectedContact.addresses.map((a,i) => <div key={i} className="flex items-center gap-1 dark:text-white"><MapPin size={16}/> {a}</div>) : <p className="text-gray-400">None</p>}
                        </div>
                        {selectedContact.notes && (
                            <div>
                              <label className="text-xs text-gray-500 uppercase">Notes</label>
                              <AutoDirText text={selectedContact.notes} as="p" className="dark:text-white whitespace-pre-wrap" />
                            </div>
                        )}
                      </div>
                      <div className="border-t border-gray-200 dark:border-gray-700 p-4 text-xs text-gray-400">
                        Last updated: {new Date(selectedContact.updated_at * 1000).toLocaleString()}
                      </div>
                    </>
                )}
              </div>
            </div>
        )}
      </div>
  )
}