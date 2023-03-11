import { defineStore } from 'pinia'
import axios from 'axios';

export const useStore = defineStore({
    id: 'auth',
    state: () => ({
        lastUpdate: null,
    }),
    persist: true,
    getters: {
        LastUpdate: state => state.lastUpdate,
    },
    actions: {
        async lastUpdateSetter() {
            axios.get('update/last_update').then(res => {
                this.lastUpdate = new Date(res.data).toLocaleTimeString('ru')
              })
        }
    }
})