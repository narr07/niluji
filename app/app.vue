<template>
	<Html class="overflow-x-hidden">
		<Body class="font-sans antialiased">
			<UApp>
				<NuxtLayout>
					<NuxtPage />
				</NuxtLayout>
				<ConfirmDeleteModal />
				<AppUpdateModal />
			</UApp>
		</Body>
	</Html>
</template>
