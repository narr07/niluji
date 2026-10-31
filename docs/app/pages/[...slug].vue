<script setup lang="ts">
	import type { ContentNavigationItem } from "@nuxt/content";
	import { findPageHeadline } from "@nuxt/content/utils";
	import { withoutTrailingSlash } from "ufo";

	definePageMeta({
		layout: "docs"
	});

	const route = useRoute();
	const { toc } = useAppConfig();
	const navigation = inject<Ref<ContentNavigationItem[]>>("navigation");
	const routePath = computed(() => withoutTrailingSlash(route.path));

	const { data: page } = await useAsyncData(routePath.value, () => queryCollection("docs").path(routePath.value).first());
	if (!page.value) {
		throw createError({ statusCode: 404, statusMessage: "Halaman dokumentasi tidak ditemukan", fatal: true });
	}

	const { data: surround } = await useAsyncData(`${routePath.value}-surround`, () => {
		return queryCollectionItemSurroundings("docs", routePath.value, {
			fields: ["description"]
		});
	});

	const title = page.value.seo?.title || page.value.title;
	const description = page.value.seo?.description || page.value.description;

	useSeoMeta({
		title,
		ogTitle: title,
		description,
		ogDescription: description
	});

	const headline = computed(() => findPageHeadline(navigation?.value, page.value?.path));

	const links = computed(() => {
		const list = [];
		if (toc?.bottom?.edit) {
			list.push({
				icon: "i-lucide-external-link",
				label: "Sunting halaman di GitHub",
				to: `${toc.bottom.edit}/${page?.value?.stem}.${page?.value?.extension}`,
				target: "_blank"
			});
		}

		return [...list, ...(toc?.bottom?.links || [])].filter(Boolean);
	});
</script>

<template>
	<UPage v-if="page">
		<UPageHeader
			:title="page.title"
			:description="page.description"
			:headline="headline"
			:ui="{
				wrapper: 'flex-col sm:flex-row sm:items-center sm:justify-between gap-4',
				links: 'shrink-0'
			}"
		>
			<template #links>
				<UButton
					v-for="(link, index) in page.links"
					:key="index"
					v-bind="link"
				/>

				<PageHeaderLinks />
			</template>
		</UPageHeader>

		<UPageBody>
			<ContentRenderer
				v-if="page"
				:value="page"
			/>

			<USeparator v-if="surround?.length" />

			<UContentSurround :surround="surround" />
		</UPageBody>

		<template
			v-if="page?.body?.toc?.links?.length"
			#right
		>
			<UContentToc
				:title="toc?.title"
				:links="page.body?.toc?.links"
				:ui="{
					linkText: 'whitespace-normal leading-snug'
				}"
			>
				<template
					v-if="toc?.bottom"
					#bottom
				>
					<div
						class="hidden lg:block space-y-4"
						:class="{ 'mt-6!': page.body?.toc?.links?.length }"
					>
						<USeparator
							v-if="page.body?.toc?.links?.length"
							type="dashed"
						/>

						<div>
							<p class="mb-2.5 text-sm font-semibold text-highlighted">
								{{ toc.bottom.title }}
							</p>
							<ul class="space-y-2">
								<li
									v-for="(item, idx) in links"
									:key="idx"
								>
									<NuxtLink
										:to="item.to"
										:target="item.target"
										class="flex items-start gap-2 text-sm text-muted transition-colors hover:text-highlighted"
									>
										<UIcon
											v-if="item.icon"
											:name="item.icon"
											class="mt-0.5 size-4 shrink-0"
										/>
										<span class="leading-snug">{{ item.label }}</span>
									</NuxtLink>
								</li>
							</ul>
						</div>
					</div>
				</template>
			</UContentToc>
		</template>
	</UPage>
</template>
