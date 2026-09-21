<script lang="ts" setup>
	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface QuestionTypeRecord {
		id: number
		name: string
		class: string | null
		subjectId: number | null
	}

	interface SessionProgress {
		class: string | null
		subject: string
		jenis: string | null
	}

	const props = defineProps<{
		kelas: string
		subjects: Subject[]
		questionTypes: QuestionTypeRecord[]
		sessions: SessionProgress[]
	}>();

	// Badge di sini nunjukin JUMLAH JENIS UJIAN yang berlaku untuk kelas+pelajaran ini — gabungan
	// dari jenis yang TERDAFTAR di registry (sama seperti SoalPelajaranGrid) DAN jenis yang
	// benar-benar dipakai di sesi ujian nyata. Union ini sengaja, supaya kalau baris registry-nya
	// kehapus/tergeser scope, jenis yang datanya masih ada tidak ikut hilang dari hitungan.
	const jenisCountFor = (subjectId: number, subjectName: string) => {
		const registered = props.questionTypes
			.filter((jt) => (jt.class === null || jt.class === props.kelas) && (jt.subjectId === null || jt.subjectId === subjectId))
			.map((jt) => jt.name);
		const fromSessions = props.sessions
			.filter((s) => s.class === props.kelas && s.subject === subjectName && s.jenis)
			.map((s) => s.jenis as string);
		return new Set([...registered, ...fromSessions]).size;
	};
</script>

<template>
	<div class="grid gap-3 grid-cols-2 sm:grid-cols-3 lg:grid-cols-4">
		<UCard
			v-for="s in subjects"
			:key="s.id"
			variant="subtle"
			class="relative cursor-pointer hover:ring-primary transition-colors"
			:ui="{ body: 'p-3' }"
			@click="navigateTo(`/hasil/${kelas}/${encodeURIComponent(s.name)}`)"
		>
			<UBadge color="neutral" variant="subtle" size="lg" class="absolute top-2 right-2">
				{{ jenisCountFor(s.id, s.name) }}
			</UBadge>

			<div class="pr-14">
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
