<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";
	import type { EditorToolbarItem } from "@nuxt/ui";
	import type { Editor } from "@tiptap/vue-3";

	defineProps<{
		placeholder?: string
	}>();

	const value = defineModel<string>({ default: "" });

	const emit = defineEmits<{
		"image-picked": [{ path: string, dataUrl: string }]
	}>();

	// UEditor (content-type="markdown") sinkron-ulang seluruh isi lewat markdown setiap kali
	// prop modelValue-nya berubah — TERMASUK gema dari perubahan yang baru saja dia emit sendiri
	// via v-model dua-arah. Baris kosong paling akhir tidak punya representasi di markdown, jadi
	// begitu Enter ditekan di ujung dokumen, baris barunya langsung "dihapus lagi" oleh sinkron-
	// ulang itu sebelum sempat kelihatan. Diputus dengan binding SATU ARAH ke UEditor (internalValue,
	// cuma di-set ulang kalau `value` berubah dari LUAR, mis. ganti soal yang diedit) — perubahan
	// dari ketikan sendiri cuma mengalir KELUAR ke `value`, tidak pernah dipush balik ke UEditor.
	const internalValue = ref(value.value);
	let lastEmitted = value.value;

	watch(value, (newVal) => {
		if (newVal === lastEmitted) return;
		internalValue.value = newVal;
	});

	const onEditorUpdate = (newVal: string) => {
		lastEmitted = newVal;
		value.value = newVal;
	};

	// Cuma format dasar yang dipakai (tebal/miring/garis bawah/coret) — tidak ada tabel, AI, atau
	// kolaborasi seperti template referensinya, karena soal cukup butuh format teks sederhana saja.
	// Gambar TIDAK dijadikan node editor asli (Tiptap) — tetap 1 file per soal, tapi tombol
	// "Sisipkan Gambar" langsung buka file picker & upload, lalu naruh PENANDA posisi ([[gambar]])
	// persis di titik kursor terakhir — jadi satu klik, bukan dua langkah terpisah yang membingungkan.
	const toolbarItems: EditorToolbarItem[][] = [[
		{ kind: "mark", mark: "bold", icon: "lucide:bold", tooltip: { text: "Tebal" } },
		{ kind: "mark", mark: "italic", icon: "lucide:italic", tooltip: { text: "Miring" } },
		{ kind: "mark", mark: "underline", icon: "lucide:underline", tooltip: { text: "Garis bawah" } },
		{ kind: "mark", mark: "strike", icon: "lucide:strikethrough", tooltip: { text: "Coret" } }
	]];

	const uploadingImage = ref(false);

	const insertImage = async (editor: Editor) => {
		const path = await openDialog({
			multiple: false,
			filters: [{ name: "Gambar", extensions: ["jpg", "jpeg", "png", "gif", "webp"] }]
		});
		if (!path) return;

		uploadingImage.value = true;
		try {
			const saved = await invoke<{ path: string, dataUrl: string }>("save_question_image", { path });
			editor.chain().focus().insertContent(`${insertSoalImagePlaceholder} `).run();
			emit("image-picked", saved);
		} finally {
			uploadingImage.value = false;
		}
	};
</script>

<template>
	<UEditor
		:model-value="internalValue"
		content-type="markdown"
		:placeholder="placeholder ?? 'Tulis soal di sini...'"
		class="rounded-md border border-default"
		:ui="{ base: 'p-3', content: 'min-h-32 max-h-[60vh] resize-y overflow-auto' }"
		@update:model-value="onEditorUpdate">
		<template #default="{ editor }">
			<div class="flex items-center border-b border-default">
				<UEditorToolbar :editor="editor" :items="toolbarItems" class="px-2 py-1 flex-1" />
				<UButton
					size="xs"
					variant="ghost"
					color="neutral"
					icon="lucide:image-plus"
					class="mr-2"
					:loading="uploadingImage"
					title="Pilih gambar & sisipkan di posisi kursor"
					@click="insertImage(editor)">
					Sisipkan Gambar
				</UButton>
			</div>
		</template>
	</UEditor>
</template>
