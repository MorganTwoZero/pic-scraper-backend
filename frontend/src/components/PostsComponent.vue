<template>
    <a @click="toClipboard" class="image" :href="post.post.post_link">
        <img class="preview_link" :src="post.post.preview_link">
        <div class="counter_wrapper">
            <div class="images_count">
                {{ post.post.images_number }}
            </div>
        </div>
    </a>
    <div class="author">
        <a :href="post.post.author_link">
            <img :src="post.post.author_profile_image">
            {{ post.post.author }}
        </a>
        {{ created }}
    </div>

    <div>
    <LikeButton :post_link="post.post.post_link" />
    </div>
</template>

<script setup>
import { computed, defineProps, onBeforeMount } from 'vue'

import LikeButton from './LikeButton.vue';

const post = defineProps({
    post: {
        type: Object,
        required: true,
    }
})

function PixivLink(post) {
    post.post.preview_link = post.post.post_link.replace('net', 'sbs') + '.jpg'
}

function LofterLink(post) {
    post.post.clipboard_link = post.post.preview_link

    post.post.preview_link = `${process.env.VUE_APP_BACKEND_URL}/lofter?lofter_link=` + post.post.preview_link.slice(0, post.post.preview_link.search("imageView")-1) + "&preview=true"

    post.post.author_profile_image = `${process.env.VUE_APP_BACKEND_URL}/lofter?lofter_link=` + post.post.author_profile_image.slice(0, post.post.author_profile_image.search("imageView")-1)
}

const created = computed(() => {
    return new Date(post.post.created).toLocaleTimeString('ru');
});

function toClipboard(e) {
        e.preventDefault();
        let text = '';
        if (post.post.post_link.startsWith('https://twitter.com/')) {
            text = `<${post.post.post_link}> ${post.post.preview_link}?name=orig`;
        } else if (post.post.post_link.startsWith('https://www.pixiv.net')) {
            text = post.post.post_link.replace('net', 'sbs');
        } else if (post.post.post_link.search("lofter") != -1) {
            text = `${post.post.post_link} ${post.post.clipboard_link.replace(/\?.*/, '')}`;
        } else {
            /* Delete everything after '?.' */
            text = `<${post.post.post_link}> ${post.post.preview_link.replace(/\?.*/, '')}`;
        }
        navigator.clipboard.writeText(text);
}

onBeforeMount(() => {
    if (post.post.post_link.startsWith('https://www.pixiv.net')) {
        PixivLink(post)
    }

    if (post.post.post_link.search("lofter") != -1) {
        LofterLink(post)
    }
})
</script>

<style scoped>
img {
    max-width: 90vw;
    max-height: 80vh;
    object-fit: cover;
}

.author {
    max-width: fit-content;
}

.author img {
    width: 50px;
}

.counter_wrapper {
    position: absolute;
    right: 0px;
    padding: 4px 4px 0px;
}

.images_count {
    display: flex;
    justify-content: center;
    align-items: center;
    box-sizing: border-box;
    height: 20px;
    min-width: 20px;
    color: rgb(255, 255, 255);
    font-weight: bold;
    background: rgba(0, 0, 0, 0.32);
    border-radius: 10px;
    font-size: 10px;
    line-height: 10px;
}

.image {
    position: relative;
    display: flex;
    margin-bottom: 10px;
    width: max-content;
}
</style>