export const usePages = () => {
	const router = useRouter();
	const { pageCategories } = useAppConfig();

	const routes = router.getRoutes().filter((route) => route.name !== "all");

	const topLevelRoutes: any[] = [];

	const categorizedRoutes = routes.reduce((acc, route) => {
		if (route.meta.hideFromNav) return acc;

		if (route.meta.top) {
			topLevelRoutes.push({
				label: route.meta.name as string || route.name,
				description: route.meta.description as string,
				icon: route.meta.icon || "i-lucide-file",
				to: route.path
			});
			return acc;
		}

		const category = route.meta.category as string || "other";
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
			label: route.meta.name as string || route.name,
			description: route.meta.description as string,
			icon: route.meta.icon || "i-lucide-file",
			to: route.path,
			order: (route.meta.order as number) ?? 99
		});

		return acc;
	}, {} as Record<string, any>);

	for (const group of Object.values(categorizedRoutes)) {
		(group as any).children.sort((a: any, b: any) => a.order - b.order);
	}

	const pages = [...topLevelRoutes, ...Object.values(categorizedRoutes)];

	return {
		pages
	};
};
