<template>
  <nav @click="collapseNavBar" class="navbar navbar-expand-sm navbar-light fixed-top bg-light">
    <div class="container">
      <a class="navbar-brand" href="/">Honkai Pic Scraper</a>
      <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarCollapse"
        aria-controls="navbarCollapse" aria-expanded="false" aria-label="Toggle navigation">
        <span class="navbar-toggler-icon"></span>
      </button>
      <div class="collapse navbar-collapse" id="navbarCollapse">
        <ul class="navbar-nav me-auto mb-2 mb-md-0">
          <li class="nav-item">
            <router-link @click="refresh" class="nav-link" to="/honkai">Honkai</router-link>
          </li>
          <li class="nav-item">
            <router-link @click="refresh" class="nav-link" to="/myfeed">My feed</router-link>
          </li>
          <li class="nav-item">
            <a @click="$emit('update')" class="nav-link">Update</a>
          </li>
          <li class="nav-item nav-link">
            Last update: {{ LastUpdate }}
          </li>
        </ul>
      </div>
    </div>
  </nav>
</template>

<script setup>
import { useStore } from '@/store'
import { storeToRefs } from 'pinia'
import router from '@/router'
const store = useStore()

const { LastUpdate } = storeToRefs(store)

function refresh(e) {
  if (e.target.getAttribute('href') == router.currentRoute.value.fullPath) {
    location.reload();
  }
}
function collapseNavBar() {
  const navbar = document.querySelector('.navbar-collapse');
  if (navbar.classList.contains('show')) {
    navbar.classList.remove('show');
  }
}
</script>

<style scoped>
a {
  cursor: pointer;
}
</style>