import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

import App from './App.vue'

import router from './router'

import 'bootstrap/dist/css/bootstrap.css';
import {Dropdown} from 'bootstrap';

import axios from 'axios';

import Toast, { POSITION } from "vue-toastification";
import "vue-toastification/dist/index.css";

axios.defaults.withCredentials = true;
axios.defaults.baseURL = import.meta.env.VITE_APP_BACKEND_URL;  // the FastAPI backend

const app = createApp(App)

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

app.use(pinia)
app.use(router, Dropdown)
app.use(Toast, {position: POSITION.BOTTOM_RIGHT});
app.mount('#app')