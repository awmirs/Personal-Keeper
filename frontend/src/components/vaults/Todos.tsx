import { useEffect, useState, useCallback } from 'react'
import api from '../../lib/api'
import type { Todo } from '../../types'
import { Plus, Trash2, Search, CheckCircle, Circle } from 'lucide-react'

export default function Todos() {
  const [todos, setTodos] = useState<Todo[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [newTitle, setNewTitle] = useState('')
  const [newDescription, setNewDescription] = useState('')

  const fetchTodos = useCallback(async () => {
    try {
      setLoading(true)
      const res = await api.get('/todos')
      setTodos(res.data)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchTodos()
  }, [fetchTodos])

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newTitle.trim()) return
    try {
      await api.post('/todos', {
        title: newTitle,
        description: newDescription,
      })
      setNewTitle('')
      setNewDescription('')
      setShowCreate(false)
      fetchTodos()
    } catch (err: any) {
      alert('Failed to create: ' + err.message)
    }
  }

  const toggleCompleted = async (todo: Todo) => {
    try {
      await api.put(`/todos/${todo.id}`, { completed: !todo.completed })
      fetchTodos()
    } catch (err: any) {
      alert('Failed to update: ' + err.message)
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this todo?')) return
    try {
      await api.delete(`/todos/${id}`)
      fetchTodos()
    } catch (err: any) {
      alert('Failed to delete: ' + err.message)
    }
  }

  const filtered = todos.filter((todo) =>
      todo.title.toLowerCase().includes(search.toLowerCase()) ||
      todo.description.toLowerCase().includes(search.toLowerCase())
  )

  return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold dark:text-white">Todos</h2>
          <button
              onClick={() => setShowCreate(!showCreate)}
              className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
          >
            <Plus size={18} />
            New Todo
          </button>
        </div>

        <div className="relative mb-4">
          <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
          <input
              type="text"
              placeholder="Search todos..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
          />
        </div>

        {showCreate && (
            <form onSubmit={handleCreate} className="mb-6 rounded bg-white p-4 shadow dark:bg-gray-800">
              <input
                  type="text"
                  placeholder="Todo title"
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  required
              />
              <textarea
                  placeholder="Description (optional)"
                  value={newDescription}
                  onChange={(e) => setNewDescription(e.target.value)}
                  rows={2}
                  className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
              <div className="flex gap-2">
                <button
                    type="submit"
                    className="rounded bg-green-600 px-4 py-2 text-white hover:bg-green-700"
                >
                  Add
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
            <p className="text-gray-500">No todos found.</p>
        )}

        <div className="space-y-2">
          {filtered.map((todo) => (
              <div
                  key={todo.id}
                  className={`rounded bg-white p-4 shadow dark:bg-gray-800 group flex items-start gap-3 ${
                      todo.completed ? 'opacity-60' : ''
                  }`}
              >
                <button
                    onClick={() => toggleCompleted(todo)}
                    className="mt-0.5 text-gray-400 hover:text-green-500 flex-shrink-0"
                    title={todo.completed ? 'Mark incomplete' : 'Mark complete'}
                >
                  {todo.completed ? (
                      <CheckCircle size={20} className="text-green-500" />
                  ) : (
                      <Circle size={20} />
                  )}
                </button>
                <div className="flex-1 min-w-0">
                  <h3
                      className={`font-semibold dark:text-white ${
                          todo.completed ? 'line-through text-gray-400 dark:text-gray-500' : ''
                      }`}
                  >
                    {todo.title}
                  </h3>
                  {todo.description && (
                      <p className="text-gray-600 dark:text-gray-400 text-sm mt-1">
                        {todo.description}
                      </p>
                  )}
                  <p className="text-xs text-gray-400 mt-1">
                    {new Date(todo.updated_at * 1000).toLocaleString()}
                  </p>
                </div>
                <button
                    onClick={() => handleDelete(todo.id)}
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