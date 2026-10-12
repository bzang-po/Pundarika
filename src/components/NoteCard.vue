<template>
  <v-card>
    <v-card-title>Note Title</v-card-title>
    <v-card-text>
        <v-form>
            <v-text-field label="Note Content" outlined></v-text-field>
        </v-form>
        <div class="markdown-note" v-html="noteContent"></div>
    </v-card-text>

  </v-card>
</template>

<script setup lang="ts">
import { ref } from "vue";

interface Props {
  noteContent: string;
}

const props = defineProps<Props>();
const noteContent = ref(props.noteContent);
</script>