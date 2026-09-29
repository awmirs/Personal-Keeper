import axios, { AxiosRequestConfig } from 'axios'
import { useAuthStore } from './auth'

const api = axios.create({ baseURL: '/api' })

// Request interceptor: attach access token
api.interceptors.request.use((config) => {
    const token = useAuthStore.getState().accessToken
    if (token) {
        config.headers.Authorization = `Bearer ${token}`
    }
    return config
})

// ----- Token refresh queue -----
let isRefreshing = false
let failedQueue: Array<{
    resolve: (token: string) => void
    reject: (err: any) => void
}> = []

const processQueue = (error: any, token: string | null) => {
    failedQueue.forEach(({ resolve, reject }) => {
        if (error) {
            reject(error)
        } else {
            resolve(token!)
        }
    })
    failedQueue = []
}

// Response interceptor: refresh on 401, retry once
api.interceptors.response.use(
    (res) => res,
    async (error) => {
        const originalRequest = error.config

        // If it's a 401 and we haven't already retried this request
        if (error.response?.status === 401 && !originalRequest._retry) {
            // Don't try to refresh if the failing request was the refresh endpoint itself
            if (originalRequest.url === '/auth/refresh') {
                useAuthStore.getState().logout()
                window.location.href = '/login'
                return Promise.reject(error)
            }

            if (isRefreshing) {
                // Queue this request until refresh completes
                return new Promise((resolve, reject) => {
                    failedQueue.push({
                        resolve: (token: string) => {
                            originalRequest.headers.Authorization = `Bearer ${token}`
                            resolve(api(originalRequest))
                        },
                        reject,
                    })
                })
            }

            originalRequest._retry = true
            isRefreshing = true

            const refreshToken = useAuthStore.getState().refreshToken
            if (!refreshToken) {
                // No refresh token available → logout immediately
                useAuthStore.getState().logout()
                window.location.href = '/login'
                return Promise.reject(error)
            }

            try {
                const res = await axios.post('/api/auth/refresh', { refresh_token: refreshToken })
                const { access_token, refresh_token } = res.data

                // Update store and localStorage
                useAuthStore.getState().setTokens(access_token, refresh_token)

                // Update failed queue and current request
                processQueue(null, access_token)
                originalRequest.headers.Authorization = `Bearer ${access_token}`
                return api(originalRequest)
            } catch (refreshError) {
                processQueue(refreshError, null)
                useAuthStore.getState().logout()
                window.location.href = '/login'
                return Promise.reject(refreshError)
            } finally {
                isRefreshing = false
            }
        }

        return Promise.reject(error)
    }
)

/**
 * Fetch a paginated list endpoint and return only its `items` array.
 *
 * Phase D1b changed every list endpoint to return `{ items, next_cursor }`.
 * This helper preserves the pre-D1c call sites, which expect a bare array.
 * It is a compatibility shim to be replaced by an explicit page-fetch when
 * the frontend consumes cursors directly (Phase D2).
 */
export async function getList<T>(url: string, config?: AxiosRequestConfig): Promise<T[]> {
    const res = await api.get(url, config)
    const data: unknown = res.data
    if (data !== null && typeof data === 'object' && Array.isArray((data as { items?: unknown }).items)) {
        return (data as { items: T[] }).items
    }
    if (Array.isArray(data)) return data as T[]
    return []
}
export default api

// Reorder helper: send new positions to the backend
export async function reorderVault(vault: string, positions: { id: string; position: number }[]) {
    return api.put(`/${vault}/reorder`, { positions })
}