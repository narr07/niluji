<script lang="ts" setup>
	import type { EditorToolbarItem } from "@nuxt/ui";

	defineProps<{
		placeholder?: string
	}>();

	const value = defineModel<string>({ default: "" });

	// Cuma format dasar yang dipakai (tebal/miring/garis bawah/coret) — tidak ada tabel, gambar
	// di dalam editor, AI, atau kolaborasi seperti template referensinya, karena soal cukup butuh
	// format teks sederhana saja.
	const toolbarItems: EditorToolbarItem[][] = [[
		{ kind: "mark", mark: "bold", icon: "lucide:bold", tooltip: { text: "Tebal" } },
		{ kind: "mark", mark: "italic", icon: "lucide:italic", tooltip: { text: "Miring" } },
		{ kind: "mark", mark: "underline", icon: "lucide:underline", tooltip: { text: "Garis bawah" } },
		{ kind: "mark", mark: "strike", icon: "lucide:strikethrough", tooltip: { text: "Coret" } }
	]];
</script>

<template>
	<UEditor
		v-model="value"
		content-type="markdown"
		:placeholder="placeholder ?? 'Tulis soal di sini...'"
		class="rounded-md border border-default min-h-32"
		:ui="{ base: 'p-3' }">
		<template #default="{ editor }">
			<UEditorToolbar :editor="editor" :items="toolbarItems" class="border-b border-default px-2 py-1" />
		</template>
	</UEditor>
</template>
