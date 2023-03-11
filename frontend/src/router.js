import { createRouter, createWebHistory } from 'vue-router'

import PostsView from './views/PostsView.vue'

const routes = [
  {
    path: '/honkai',
    name: 'honkai',
    component: PostsView,
    meta: {title: 'Honkai'},
  },
  {
    path: '/myfeed',
    name: 'myfeed',
    component: PostsView,
    meta: {title: 'My feed'},
  },
  {
    path: '/:catchAll(.*)', redirect: '/honkai' 
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

const DEFAULT_TITLE = 'DEFAULT_TITLE';
router.afterEach((to) => {
    // Use next tick to handle router history correctly
    // see: https://github.com/vuejs/vue-router/issues/914#issuecomment-384477609
        document.title = to.meta.title || DEFAULT_TITLE;
});

export default router