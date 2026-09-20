<script lang="ts" setup>
	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface SessionProgress {
		class: string | null
		subject: string
	}

	const props = defineProps<{
		kelas: string
		subjects: Subject[]
		sessions: SessionProgress[]
	}>();

	const countFor = (subjectName: string) =>
		props.sessions.filter((s) => s.class === props.kelas && s.subject === subjectName).length;
</script>

<template>
	<div class="grid gap-4 sm:grid-cols-3 lg:grid-cols-4">
		<UCard
			v-for="s in subjects"
			:key="s.id"
			class="cursor-pointer hover:ring-2 hover:ring-primary transition-shadow"
			@click="navigateTo(`/hasil/${kelas}/${encodeURIComponent(s.name)}`)"
		>
			<div class="text-center py-4">
				<p class="text-lg font-semibold">
					{{ s.code || s.name }}
				</p>
				<p class="text-muted text-xs mt-1">
					{{ s.name }}
				</p>
				<p class="text-muted text-sm mt-1">
					{{ countFor(s.name) }} sesi
				</p>
			</div>
		</UCard>
	</div>
</template>
