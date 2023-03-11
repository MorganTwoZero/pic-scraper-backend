<template>
  <NavBar @update="update" />
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

function update() {
  toast.info("Update requested")
  axios.get('/update').then(response => {
    toast(response.data.message, {
      duration: 1000
    });
  }).catch(error => {
    console.log(error);
    toast.error(error, {
      duration: 1000
    });
  });
}
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
