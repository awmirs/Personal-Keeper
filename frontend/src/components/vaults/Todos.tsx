import { useEffect, useState, useCallback } from 'react'
import api, { reorderVault } from '../../lib/api'
import type { Todo } from '../../types'
import { Plus, Trash2, Search, CheckCircle, Circle, ChevronUp, ChevronDown, X } from 'lucide-react'
import AutoDirText from "../AutoDirText.tsx";
import LoadingSpinner from "../LoadingSpinner.tsx";
import {useConfirmation} from "../../context/ConfirmationContext.tsx";
import { useViewStore } from '../../stores/viewStore'
import ViewSwitcher from '../ViewSwitcher'
import ListView from '../views/ListView'
import GridView from '../views/GridView'
import CompactListView from '../views/CompactListView'

export default function Todos() {
  const { confirm } = useConfirmation()
  const [todos, setTodos] = useState<Todo[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [newTitle, setNewTitle] = useState('')
  const [newDescription, setNewDescription] = useState('')
  const [editOrder, setEditOrder] = useState(false)
  const view = useViewStore((s) => s.views.todos || 'list')
  const [selectedTodo, setSelectedTodo] = useState<Todo | null>(null)

  const getPlainTextSnippet = (text: string, maxLen = 100) => {
    if (text.length <= maxLen) return text
    return text.slice(0, maxLen).trimEnd() + '…'
  }

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
    // Optimistically update local state
    const previousTodos = todos
    const updatedTodos = todos.map(t =>
        t.id === todo.id ? { ...t, completed: !t.completed, updated_at: Math.floor(Date.now() / 1000) } : t
    )
    setTodos(updatedTodos)

    try {
      await api.put(`/todos/${todo.id}`, { completed: !todo.completed })
    } catch (err: any) {
      // Revert on failure
      setTodos(previousTodos)
      alert('Failed to update: ' + err.message)
    }
  }

  const handleDelete = async (id: string) => {
    const ok = await confirm('Delete this todo?')
    if (!ok) return
    try {
      await api.delete(`/todos/${id}`)
      fetchTodos()
    } catch (err: any) {
      alert('Failed to delete: ' + err.message)
    }
  }

  const moveTodo = async (index: number, direction: 'up' | 'down') => {
    const newTodos = [...todos]
    const targetIndex = direction === 'up' ? index - 1 : index + 1
    if (targetIndex < 0 || targetIndex >= newTodos.length) return

    const itemA = newTodos[index]
    const itemB = newTodos[targetIndex]

    ;[newTodos[index], newTodos[targetIndex]] = [newTodos[targetIndex], newTodos[index]]

    const tempPos = itemA.position
    itemA.position = itemB.position
    itemB.position = tempPos

    newTodos.sort((a, b) => a.position - b.position)

    setTodos(newTodos)

    try {
      await reorderVault('todos', [
        { id: itemA.id, position: itemA.position },
        { id: itemB.id, position: itemB.position },
      ])
    } catch (err: any) {
      fetchTodos()
      alert('Failed to reorder: ' + (err.response?.data?.error || err.message))
    }
  }

  const filtered = todos.filter((todo) =>
      todo.title.toLowerCase().includes(search.toLowerCase()) ||
      todo.description.toLowerCase().includes(search.toLowerCase())
  )

  const renderTodo = (todo: Todo, index: number) => (
      <div
          key={todo.id}
          className={`rounded bg-white p-4 shadow dark:bg-gray-800 group flex items-start gap-3 ${
              todo.completed ? 'opacity-60' : ''
          } ${
              view === 'grid' ? 'h-48 overflow-hidden cursor-pointer' : ''
          }`}
          onClick={() => view === 'grid' && setSelectedTodo(todo)}
      >
        <button
            onClick={(e) => { e.stopPropagation(); toggleCompleted(todo); }}
            className="mt-0.5 text-gray-400 hover:text-green-500 flex-shrink-0 p-1"
            title={todo.completed ? 'Mark incomplete' : 'Mark complete'}
        >
          {todo.completed ? (
              <CheckCircle size={22} className="text-green-500" />
          ) : (
              <Circle size={22} />
          )}
        </button>
        <div className="flex-1 min-w-0">
          <div className="flex justify-between items-start">
            <h3
                className={`font-semibold dark:text-white ${
                    todo.completed ? 'line-through text-gray-400 dark:text-gray-500' : ''
                }`}
            >
              {todo.title}
            </h3>
            <div className="flex items-start gap-1">
              {editOrder && (
                  <div className="flex flex-col gap-0.5">
                    <button
                        onClick={(e) => { e.stopPropagation(); moveTodo(index, 'up'); }}
                        disabled={index === 0}
                        className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                        title="Move up"
                    >
                      <ChevronUp size={18} />
                    </button>
                    <button
                        onClick={(e) => { e.stopPropagation(); moveTodo(index, 'down'); }}
                        disabled={index === filtered.length - 1}
                        className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                        title="Move down"
                    >
                      <ChevronDown size={18} />
                    </button>
                  </div>
              )}
              <button
                  onClick={(e) => { e.stopPropagation(); handleDelete(todo.id); }}
                  className="text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 p-1"
                  title="Delete"
              >
                <Trash2 size={20} />
              </button>
            </div>
          </div>
          <hr className="my-2 border-gray-200 dark:border-gray-700" />
          {view === 'grid' ? (
              <p className="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
                {getPlainTextSnippet(todo.description || 'No description', 80)}
              </p>
          ) : (
              todo.description && (
                  <AutoDirText
                      text={todo.description}
                      as="p"
                      className="text-gray-600 dark:text-gray-400 text-sm mt-1"
                  />
              )
          )}
          <div className="text-xs text-gray-400 mt-1 space-y-0.5">
            <p>Created: {new Date(todo.created_at * 1000).toLocaleString()}</p>
            <p>Updated: {new Date(todo.updated_at * 1000).toLocaleString()}</p>
          </div>
        </div>
      </div>
  );

  return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold dark:text-white">Todos</h2>
          <div className="flex gap-2 items-center">
            <ViewSwitcher vaultKey="todos" />
            <button
                onClick={() => setEditOrder(!editOrder)}
                className={`flex items-center gap-2 rounded px-4 py-2 ${
                    editOrder ? 'bg-green-600 hover:bg-green-700 text-white' : 'bg-gray-200 dark:bg-gray-700 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600'
                }`}
            >
              {editOrder ? 'Done' : 'Edit Order'}
            </button>
            <button
                onClick={() => setShowCreate(!showCreate)}
                className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
            >
              <Plus size={18} />
              New Todo
            </button>
          </div>
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

        {loading && <LoadingSpinner message="Loading todos..." />}
        {error && <p className="text-red-500">Error: {error}</p>}
        {!loading && !error && filtered.length === 0 && (
            <p className="text-gray-500">No todos found.</p>
        )}

        {view === 'list' && <ListView items={filtered} renderItem={renderTodo} />}
        {view === 'grid' && <GridView items={filtered} renderItem={renderTodo} />}
        {view === 'compact' && <CompactListView items={filtered} renderItem={renderTodo} />}


        {/* Detail modal */}
        {selectedTodo && (
            <div
                className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4"
                onClick={() => setSelectedTodo(null)}
            >
              <div
                  className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl max-h-[85vh] overflow-y-auto"
                  onClick={(e) => e.stopPropagation()}
              >
                <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                  <h3 className="text-xl font-bold dark:text-white">{selectedTodo.title}</h3>
                  <button
                      onClick={() => setSelectedTodo(null)}
                      className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"
                  >
                    <X size={20} />
                  </button>
                </div>
                <div className="p-6 space-y-3">
                  <div>
                    <label className="text-xs text-gray-500 uppercase">Status</label>
                    <p className="dark:text-white">{selectedTodo.completed ? 'Completed' : 'Pending'}</p>
                  </div>
                  {selectedTodo.description && (
                      <div>
                        <label className="text-xs text-gray-500 uppercase">Description</label>
                        <AutoDirText text={selectedTodo.description} as="p" className="dark:text-white whitespace-pre-wrap" />
                      </div>
                  )}
                  <div>
                    <label className="text-xs text-gray-500 uppercase">Due Date</label>
                    <p className="dark:text-white">
                      {selectedTodo.due_date
                          ? new Date(selectedTodo.due_date * 1000).toLocaleString()
                          : 'No due date'}
                    </p>
                  </div>
                </div>
                <div className="border-t border-gray-200 dark:border-gray-700 p-4 text-xs text-gray-400">
                  Last updated: {new Date(selectedTodo.updated_at * 1000).toLocaleString()}
                </div>
              </div>
            </div>
        )}
      </div>
  )
}