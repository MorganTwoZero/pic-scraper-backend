<template>
    <div v-if="isLikeable">
        <button @click="like" class="btn btn-primary">Like</button>
    </div>
</template>

<script setup>
import axios from 'axios';

const post_link = defineProps({
    post_link: {
        type: String,
        required: true,
    }
})

const isLikeable = post_link.post_link.includes('twitter.com')

function like(e) {
    e.preventDefault();
    axios.get('/like', {
        params: {
            post_link: post_link.post_link
        }
    })
    e.target.innerText = 'Liked'
    e.target.blur()
    e.target.disabled = true
    e.target.classList.add('btn-success')
}
</script>