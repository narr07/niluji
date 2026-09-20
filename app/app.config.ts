export default defineAppConfig({
	app: {
		name: "NILUJI",
		shortName: "NILUJI",
		description: "Platform CBT dan e-Rapor terintegrasi untuk sekolah",
		tagline: "Ujian dan Rapor dalam Satu Sistem",
		author: "narr07",
		repo: "https://github.com/narr07/niluji",
		tauriSite: "https://tauri.app",
		nuxtSite: "https://nuxt.com",
		nuxtUiSite: "https://ui.nuxt.com"
	},
	// Data sekolah sekarang dikelola lewat halaman Pengaturan (tersimpan di database),
	// lihat app/pages/pengaturan.vue.
	pageCategories: {
		cbt: {
			label: "Ujian",
			icon: "lucide:graduation-cap"
		},
		other: {
			label: "Other",
			icon: "lucide:folder"
		},
		settings: {
			label: "Pengaturan",
			icon: "lucide:settings"
		}
	},
	ui: {
		colors: {
			primary: "blue",
			neutral: "neutral"
		},
		// card: {
		//   defaultVariants: {
		//     variant: 'soft'
		//   }
		// },
		button: {
			slots: {
				base: "cursor-pointer"
			},
			defaultVariants: {
				variant: "subtle",
			}
		},
		badge: {
			defaultVariants: {
				variant: "soft"
			}
		},
		formField: {
			slots: {
				root: "w-full"
			}
		},
		input: {
			slots: {
				root: "w-full"
			},
			defaultVariants: {
				variant: "soft"
			}
		},
		textarea: {
			slots: {
				root: "w-full",
				base: "resize-none"
			},
			defaultVariants: {
				variant: "soft"
			}
		},
		select: {
			defaultVariants: {
				variant: "soft"
			}
		},
		selectMenu: {
			defaultVariants: {
				variant: "soft"
			}
		},
		inputMenu: {
			defaultVariants: {
				variant: "soft"
			}
		},
		inputNumber: {
			defaultVariants: {
				variant: "soft"
			}
		},
		inputTags: {
			defaultVariants: {
				variant: "soft"
			}
		},
		inputDate: {
			defaultVariants: {
				variant: "soft"
			}
		},
		inputTime: {
			defaultVariants: {
				variant: "soft"
			}
		},
		pinInput: {
			defaultVariants: {
				variant: "soft"
			}
		},
		accordion: {
			slots: {
				trigger: "cursor-pointer",
				item: "md:py-2"
			}
		},
		navigationMenu: {
			slots: {
				link: "cursor-pointer"
			}
		}
	}
});
