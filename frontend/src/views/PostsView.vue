<template>
  <div v-for="post in posts" :key="post.post_link" @keydown.enter="openInNewTab" @keydown.up.prevent="prevPost" @keydown.down.prevent="nextPost" tabindex="0"> 
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
let page = 0;
let isLoading = false;
let nextPage = true;
let throttleTimeout = null;

function openInNewTab(event) {
  window.open(event.target.href, "_blank");
}

function nextPost(event) {
  event.target.parentNode.nextSibling.firstElementChild.focus({ preventScroll: true });
  event.target.parentNode.nextSibling.firstElementChild.scrollIntoView({ behavior: "smooth", block: "center" });
}

function prevPost(event) {
  event.target.parentNode.previousSibling.firstElementChild.focus({ preventScroll: true });
  event.target.parentNode.previousSibling.firstElementChild.scrollIntoView({ behavior: "smooth", block: "center" });
}

function setLoadingObserver() {
  const loadingObserver = new IntersectionObserver(handleObserver);
  loadingObserver.observe(document.querySelector('#loading'))
}

async function getPosts() {
  if (nextPage && !isLoading) {
    try {
      isLoading = true;
      const response = await axios.get(router.currentRoute.value.fullPath + '?page=' + page);
      if (response.data.length === 0) {
        nextPage = false;
      } else {
        let filtered = filterObjects(response.data, getListFromLocalStorage());
        posts.value.push(...filtered);
        page += 1;
      }
    } catch (error) {
      console.log(error);
    } finally {
      isLoading = false;
    }
  }
}

function getListFromLocalStorage() {
  const storedList = localStorage.getItem('filterList');
  return storedList ? JSON.parse(storedList) : [];
}

function filterObjects(objects, filterList) {
  return objects.filter(obj => 
    !filterList.some(item => Object.values(obj).includes(item))
  );
}

function handleObserver(entries) {
  if (throttleTimeout) {
    clearTimeout(throttleTimeout);
  }

  throttleTimeout = setTimeout(() => {
    if (entries[0].isIntersecting) {
      getPosts();
    }
  }, 100);
}

onMounted(() => {
  setLoadingObserver();
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
