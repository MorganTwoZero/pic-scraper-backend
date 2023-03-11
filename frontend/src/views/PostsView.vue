<template>
  <div v-for="post in posts" :key="post.post_link">
    <PostsComponent :post="post" />
  </div>
  <div id="loading">
    <svg viewBox="25 25 50 50">
      <circle cx="50" cy="50" r="20"></circle>
    </svg>
  </div>
</template>

<script setup>
import axios from 'axios'

import { onMounted, ref } from 'vue'

import router from '@/router'
import PostsComponent from '@/components/PostsComponent.vue'

let posts = ref([]);
let page = 1;

function setLoadingObserver() {
  const loadingObserver = new IntersectionObserver(getPosts);
  loadingObserver.observe(document.querySelector('#loading'))
}

function getPosts() {
  axios.get(router.currentRoute.value.fullPath + '?page=' + page).then(response => {
    posts.value.push(...response.data);
    page++;
  });
}

onMounted(() => {
  setLoadingObserver()
})
</script>

<style scoped>
div {
  border-radius: 2px;
  padding: 10px;
  border: 1px solid rgb(0, 0, 0);
  width: max-content;
  height: max-content;
}

#loading {
  border: 3px solid hsla(185, 100%, 62%, 0.2);
  border-top-color: #3cefff;
  border-radius: 50%;
  width: 3em;
  height: 3em;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>