<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	const props = defineProps<{
		kelas: string
		pelajaran: string
	}>();

	interface Subject {
		id: number
		name: string
	}

	interface QuestionTypeRecord {
		id: number
		name: string
	}

	interface SessionProgress {
		class: string | null
		subject: string
		jenis: string | null
	}

	const subjects = ref<Subject[]>([]);
	const questionTypes = ref<QuestionTypeRecord[]>([]);
	const sessions = ref<SessionProgress[]>([]);
	const loading = ref(true);

	const subjectId = computed(() => subjects.value.find((s) => s.name === props.pelajaran)?.id);

	const load = async () => {
		loading.value = true;
		try {
			subjects.value = await invoke<Subject[]>("list_subjects");
			[questionTypes.value, sessions.value] = await Promise.all([
				invoke<QuestionTypeRecord[]>("list_question_types", { class: props.kelas, subjectId: subjectId.value }),
				invoke<SessionProgress[]>("list_exam_sessions")
			]);
		} finally {
			loading.value = false;
		}
	};

	// Gabungan dari DUA sumber: jenis yang terdaftar di registry (question_types — biar jenis
	// yang belum ada sesinya tetap kelihatan, badge-nya "0 sesi") DAN jenis yang benar-benar
	// dipakai di data sesi ujian nyata (exam_sessions). Union ini sengaja — kalau baris
	// registry-nya kehapus/tergeser scope (mis. lewat fitur hapus jenis bertingkat di Bank
	// Soal), data sesi yang sudah ada tidak boleh sampai hilang dari tampilan cuma gara-gara
	// registry-nya tidak sinkron lagi.
	const jenisGroups = computed(() => {
		const relevantSessions = sessions.value.filter((s) => s.class === props.kelas && s.subject === props.pelajaran && s.jenis);
		const names = new Set<string>([...questionTypes.value.map((t) => t.name), ...relevantSessions.map((s) => s.jenis as string)]);
		return [...names].map((name) => ({
			id: questionTypes.value.find((t) => t.name === name)?.id ?? name,
			jenis: name,
			count: relevantSessions.filter((s) => s.jenis === name).length
		}));
	});

	onMounted(load);
</script>

<template>
	<div class="space-y-6">
		<div v-if="loading" class="grid gap-3 sm:grid-cols-3 lg:grid-cols-4">
			<USkeleton v-for="i in 3" :key="i" class="h-24 rounded-lg" />
		</div>

		<div v-else-if="jenisGroups.length" class="grid gap-3 grid-cols-2 sm:grid-cols-3 lg:grid-cols-4">
			<UCard
				v-for="g in jenisGroups"
				:key="g.id"
				variant="subtle"
				class="relative cursor-pointer hover:ring-primary transition-colors"
				:ui="{ body: 'p-3' }"
				@click="navigateTo(`/hasil/${kelas}/${encodeURIComponent(pelajaran)}/${encodeURIComponent(g.jenis)}`)"
			>
				<UBadge
					color="neutral"
					variant="subtle"
					size="lg"
					class="absolute top-2 right-2">
					{{ g.count }} sesi
				</UBadge>

				<div class="pr-14">
					<p class="text-xl font-bold truncate">
						{{ g.jenis }}
					</p>
				</div>
			</UCard>
		</div>

		<UAlert
			v-else
			icon="i-lucide-inbox"
			title="Belum ada jenis ujian"
			description="Buat jenis ujian dulu di Bank Soal untuk kelas & mata pelajaran ini."
			variant="subtle"
			color="neutral" />
	</div>
</template>
