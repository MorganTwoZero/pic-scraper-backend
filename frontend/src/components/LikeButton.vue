<template>
    <div v-if="isLikeable">
        <button @click="like" class="btn btn-primary">🤍</button>
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
    }).then((response) => {
        e.target.blur()
        e.target.disabled = true
        e.target.classList.add('btn-success')
    })
    .catch((error) => {
        e.target.innerText = error
        e.target.blur()
        e.target.disabled = true
        e.target.classList.add('btn-danger')
    })
}
</script>