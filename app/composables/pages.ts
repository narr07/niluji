interface NavRouteItem {
	label: string
	description?: string
	icon: string
	to: string
	order: number
}

interface NavCategoryGroup {
	label?: string
	icon: string
	type: "trigger"
	children: NavRouteItem[]
}

export const usePages = () => {
	const router = useRouter();
	const { pageCategories } = useAppConfig();

	const routes = router.getRoutes().filter((route) => route.name !== "all");

	const topLevelRoutes: NavRouteItem[] = [];

	const categorizedRoutes = routes.reduce<Record<string, NavCategoryGroup>>((acc, route) => {
		if (route.meta.hideFromNav) return acc;

		if (route.meta.top) {
			topLevelRoutes.push({
				label: (route.meta.name as string) || String(route.name ?? ""),
				description: route.meta.description as string | undefined,
				icon: (route.meta.icon as string) || "i-lucide-file",
				to: route.path,
				order: (route.meta.order as number) ?? 0
			});
			return acc;
		}

		const category = (route.meta.category as string) || "other";
		if (!category) return acc;

		if (!acc[category]) {
			acc[category] = {
				label: pageCategories[category as keyof typeof pageCategories]?.label,
				icon: pageCategories[category as keyof typeof pageCategories]?.icon || "i-lucide-folder",
				type: "trigger",
				children: []
			};
		}

		acc[category].children.push({
			label: (route.meta.name as string) || String(route.name ?? ""),
			description: route.meta.description as string | undefined,
			icon: (route.meta.icon as string) || "i-lucide-file",
			to: route.path,
			order: (route.meta.order as number) ?? 99
		});

		return acc;
	}, {});

	topLevelRoutes.sort((a, b) => a.order - b.order);

	for (const group of Object.values(categorizedRoutes)) {
		group.children.sort((a, b) => a.order - b.order);
	}

	const pages = [...topLevelRoutes, ...Object.values(categorizedRoutes)];

	return {
		pages
	};
};
