<template>
  <NavBar />
  <div class="main container">
    <router-view :key="$route.fullPath" />
  </div>
</template>

<script setup>
import axios from 'axios';
import NavBar from '@/components/NavBar.vue'
import { useToast } from "vue-toastification";
import { useStore } from '@/store'
import { onBeforeUpdate } from 'vue'
const toast = useToast()

const store = useStore()

onBeforeUpdate(() => {
  store.lastUpdateSetter()
})
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}

.main {
  padding-top: 5em;
}

.main.container {
  display: flex;
  flex-flow: column;
  align-items: center;
}

.main.container>div {
  margin: 10px;
}
</style>
