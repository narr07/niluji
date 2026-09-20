<script lang="ts" setup>
	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface QuestionSummary {
		subject: string
		class: string | null
		jenis: string | null
	}

	const props = defineProps<{
		kelas: string
		subjects: Subject[]
		questions: QuestionSummary[]
	}>();

	// Badge di kartu pelajaran ini nunjukin JUMLAH JENIS UJIAN yang ada, bukan jumlah soal —
	// jumlah soal per jenis ada di kartu jenis-nya sendiri satu level di bawah.
	const jenisCountFor = (subjectName: string) =>
		new Set(
			props.questions.filter((q) => q.class === props.kelas && q.subject === subjectName && q.jenis).map((q) => q.jenis)
		).size;
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
				{{ jenisCountFor(s.name) }}
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