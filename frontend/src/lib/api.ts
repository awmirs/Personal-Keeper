import axios from 'axios'
import { useAuthStore } from './auth'

const api = axios.create({ baseURL: '/' })

api.interceptors.request.use((config) => {
    const token = useAuthStore.getState().accessToken
    if (token) {
        config.headers.Authorization = `Bearer ${token}`
    }
    return config
})

// Optional: handle 401 and refresh token logic
api.interceptors.response.use(
    (res) => res,
    async (err) => {
        if (err.response?.status === 401) {
            useAuthStore.getState().logout()
            window.location.href = '/login'
        }
        return Promise.reject(err)
    }
)

export default api