<script lang="ts" setup>
	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface QuestionTypeRecord {
		id: number
		class: string | null
		subjectId: number | null
	}

	const props = defineProps<{
		kelas: string
		subjects: Subject[]
		questionTypes: QuestionTypeRecord[]
	}>();

	// Badge di kartu pelajaran ini nunjukin JUMLAH JENIS UJIAN yang berlaku untuk kelas+pelajaran
	// ini — dihitung dari jenis yang TERDAFTAR (termasuk yang masih kosong, belum ada soalnya),
	// bukan cuma yang sudah dipakai di soal. class/subjectId NULL pada jenis berarti "berlaku
	// untuk semua", jadi tetap dihitung meski bukan scope spesifik kelas/pelajaran ini.
	const jenisCountFor = (subjectId: number) =>
		props.questionTypes.filter(
			(jt) => (jt.class === null || jt.class === props.kelas) && (jt.subjectId === null || jt.subjectId === subjectId)
		).length;
</script>

<template>
	<div class="grid gap-3 grid-cols-2 sm:grid-cols-3 lg:grid-cols-4">
		<UCard
			v-for="s in subjects"
			:key="s.id"
			variant="subtle"
			class="relative cursor-pointer hover:ring-primary transition-colors"
			:ui="{ body: 'p-3' }"
			@click="navigateTo(`/soal/${kelas}/${encodeURIComponent(s.name)}`)"
		>
			<UBadge
				color="primary"
				variant="solid"
				size="lg"
				class="absolute top-2 right-2"
			>
				{{ jenisCountFor(s.id) }}
			</UBadge>

			<div class="pr-6">
				<p class="text-xl font-bold truncate">
					{{ s.code || s.name }}
				</p>
				<p v-if="s.code" class="text-muted text-xs truncate">
					{{ s.name }}
				</p>
			</div>
		</UCard>
	</div>
</template>