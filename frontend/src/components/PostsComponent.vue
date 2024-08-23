<template>
    <a @keydown.l="like" @click.prevent="toClipboard" class="image" :href="post.post.post_link">
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
        <button
            ref="likeButtonRef"
            @click.prevent="like"
            class="btn btn-primary"
            v-if="post.post.post_link.includes('twitter.com')">
        🤍
        </button>
    </div>
</template>

<script setup>
import { computed, onBeforeMount, ref } from 'vue'
import axios from 'axios';

const likeButtonRef = ref();
function like() {
    likeButtonRef.value.disabled = true;
    likeButtonRef.value.blur();
    axios.get('/like', {
        params: {
            post_link: post.post.post_link
        }
    }).then((response) => {
        likeButtonRef.value.classList.add('btn-success')
    })
    .catch((error) => {
        likeButtonRef.value.innerText = error
        likeButtonRef.value.classList.add('btn-danger')
    })
}

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

    post.post.preview_link = `${import.meta.env.VITE_APP_BACKEND_URL}/jpg?url=` + post.post.preview_link

    post.post.author_profile_image = `${import.meta.env.VITE_APP_BACKEND_URL}/jpg?url=` + post.post.author_profile_image
}

function LofterAuthorLink(post) {
    post.post.author_link = `https://www.lofter.com/front/blog/home-page/${post.post.author_link.match(/https:\/\/(.+?)\.lofter\.com/)[1]}`;
}

const created = computed(() => {
    return new Date(post.post.created).toLocaleTimeString('ru');
});

function toClipboard(e) {
        e.preventDefault();
        let text = '';
        if (post.post.post_link.startsWith('https://twitter.com/')) {
            text = post.post.post_link.replace('twitter', 'fxtwitter');
        } else if (post.post.post_link.startsWith('https://www.pixiv.net')) {
            text = post.post.post_link.replace('pixiv', 'phixiv');
        } else if (post.post.post_link.search("lofter") != -1) {
            text = `${post.post.post_link} [.](${post.post.clipboard_link.replace(/\?.*/, '')})`;
        } else {
            /* Delete everything after '?.' */
            text = `<${post.post.post_link}> [.](${post.post.preview_link.replace(/\?.*/, '')})`;
        }
        navigator.clipboard.writeText(text);
}

onBeforeMount(() => {
    if (post.post.post_link.startsWith('https://www.pixiv.net')) {
        PixivLink(post)
    }

    if (post.post.post_link.search("lofter") != -1) {
        LofterLink(post);
        LofterAuthorLink(post);
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
