const devHost = process.env.TAURI_DEV_HOST;

export default defineNuxtConfig({
	modules: [
		"@nuxt/eslint",
		"@nuxt/ui"
	],

	ssr: false,

	devtools: {
		enabled: false
	},

	app: {
		head: {
			title: "NILUJI Siswa",
			htmlAttrs: {
				lang: "id"
			},
			meta: [
				{ name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" },
				{ name: "description", content: "Masuk ke ujian NILUJI dari HP siswa." },
				{ name: "format-detection", content: "telephone=no" }
			]
		}
	},

	css: ["~/assets/css/main.css"],

	ignore: ["**/src-tauri/**"],

	devServer: {
		host: devHost || "localhost",
		port: 3000
	},

	compatibilityDate: "2026-01-01",

	vite: {
		clearScreen: false,
		envPrefix: ["VITE_", "TAURI_"],
		server: {
			strictPort: true,
			hmr: devHost ? { protocol: "ws", host: devHost, port: 3001 } : undefined
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

	// Aplikasi berjalan tanpa internet, jadi semua ikon dibundel ke klien.
	icon: {
		provider: "none",
		fallbackToApi: false,
		clientBundle: {
			scan: true,
			sizeLimitKb: 512
		}
	}
});
