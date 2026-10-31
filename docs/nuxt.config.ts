export default defineNuxtConfig({
	modules: [
		"@nuxt/eslint",
		"@nuxt/image",
		"@nuxt/ui",
		"@nuxt/content"
	],

	devtools: {
		enabled: false
	},

	app: {
		baseURL: process.env.NUXT_APP_BASE_URL || "/niluji/",
		head: {
			title: "NILUJI — Platform Ujian CBT Offline untuk Sekolah",
			meta: [
				{
					name: "description",
					content: "Dokumentasi resmi dan pusat unduhan NILUJI — platform ujian berbasis komputer (CBT) 100% offline untuk SD, SMP, dan SMA berbasis Tauri 2 & Nuxt 4."
				},
				{
					name: "author",
					content: "Dinar Permadi Yusup (permadi.dev)"
				}
			]
		}
	},

	css: ["~/assets/css/main.css"],

	content: {
		build: {
			markdown: {
				toc: {
					searchDepth: 2
				}
			}
		},
		experimental: {
			sqliteConnector: "native"
		}
	},

	experimental: {
		asyncContext: true
	},

	compatibilityDate: "2026-01-01",

	nitro: {
		prerender: {
			routes: [
				"/",
				"/rilis"
			],
			crawlLinks: true
		}
	},

	eslint: {
		config: {
			stylistic: {
				indent: "tab",
				quotes: "double",
				semi: true,
				commaDangle: "never",
				braceStyle: "1tbs"
			}
		}
	},

	icon: {
		serverBundle: "local",
		fallbackToApi: false,
		clientBundle: {
			scan: true,
			sizeLimitKb: 512,
			icons: [
				"lucide:rocket",
				"lucide:graduation-cap",
				"lucide:shield-check",
				"lucide:laptop",
				"lucide:wrench",
				"lucide:bug",
				"lucide:history",
				"lucide:globe",
				"lucide:user",
				"lucide:star",
				"lucide:download",
				"lucide:book-open",
				"lucide:book-open-check",
				"lucide:arrow-right",
				"lucide:wifi-off",
				"lucide:radio",
				"lucide:file-text",
				"lucide:file-spreadsheet",
				"lucide:file-code",
				"lucide:file-warning",
				"lucide:folder-archive",
				"lucide:bar-chart-3",
				"lucide:database-backup",
				"lucide:id-card",
				"lucide:smartphone",
				"lucide:cloud-download",
				"lucide:server",
				"lucide:user-x",
				"lucide:lock",
				"lucide:clock",
				"lucide:monitor",
				"lucide:terminal",
				"lucide:users",
				"lucide:circle-check",
				"lucide:check",
				"lucide:check-circle",
				"lucide:link",
				"lucide:external-link",
				"lucide:chevron-down",
				"simple-icons:github"
			]
		}
	}
});
