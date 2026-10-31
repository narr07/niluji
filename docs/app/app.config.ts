export default defineAppConfig({
	ui: {
		colors: {
			primary: "emerald",
			neutral: "slate"
		},
		footer: {
			slots: {
				root: "border-t border-default",
				left: "text-sm text-muted"
			}
		},
		prose: {
			codeIcon: {
				md: "i-lucide-file-text",
				markdown: "i-lucide-file-text",
				csv: "i-lucide-file-spreadsheet",
				json: "i-lucide-file-code",
				bash: "i-lucide-terminal",
				sh: "i-lucide-terminal",
				powershell: "i-lucide-terminal"
			}
		}
	},
	seo: {
		siteName: "Dokumentasi NILUJI"
	},
	repository: "narr07/niluji",
	version: "1.0.1",
	header: {
		title: "NILUJI",
		to: "/",
		logo: {
			alt: "NILUJI",
			light: "",
			dark: ""
		},
		search: true,
		colorMode: true,
		links: [
			{
				label: "Panduan",
				icon: "i-lucide-book-open",
				to: "/memulai/instalasi",
				color: "neutral" as const,
				variant: "ghost" as const
			},
			{
				label: "Rilis & Unduhan",
				icon: "i-lucide-download",
				to: "/rilis",
				color: "neutral" as const,
				variant: "ghost" as const
			},
			{
				label: "permadi.dev",
				icon: "i-lucide-globe",
				to: "https://permadi.dev/id/projek/niluji",
				target: "_blank",
				color: "neutral" as const,
				variant: "ghost" as const
			},
			{
				"icon": "i-simple-icons-github",
				"to": "https://github.com/narr07/niluji",
				"target": "_blank",
				"aria-label": "Repositori GitHub NILUJI",
				"color": "neutral" as const,
				"variant": "ghost" as const
			}
		]
	},
	footer: {
		credits: `NILUJI CBT Offline Sekolah • Lisensi MIT © ${new Date().getFullYear()}`,
		colorMode: false,
		links: [
			{
				label: "permadi.dev",
				icon: "i-lucide-globe",
				to: "https://permadi.dev",
				target: "_blank"
			},
			{
				label: "Studi Kasus",
				icon: "i-lucide-book-open-check",
				to: "https://permadi.dev/id/projek/niluji",
				target: "_blank"
			},
			{
				label: "Unduh Aplikasi",
				icon: "i-lucide-download",
				to: "https://github.com/narr07/niluji/releases/latest",
				target: "_blank"
			},
			{
				label: "Laporkan Masalah",
				icon: "i-lucide-bug",
				to: "https://github.com/narr07/niluji/issues",
				target: "_blank"
			},
			{
				"icon": "i-simple-icons-github",
				"to": "https://github.com/narr07/niluji",
				"target": "_blank",
				"aria-label": "NILUJI di GitHub"
			}
		]
	},
	toc: {
		title: "Daftar Isi Halaman",
		bottom: {
			title: "Tautan Cepat",
			edit: "https://github.com/narr07/niluji/edit/main/docs/content",
			links: [
				{
					icon: "i-lucide-download",
					label: "Unduh Installer Terbaru",
					to: "https://github.com/narr07/niluji/releases/latest",
					target: "_blank"
				},
				{
					icon: "i-lucide-history",
					label: "Catatan Rilis (Changelog)",
					to: "/rilis"
				},
				{
					icon: "i-lucide-globe",
					label: "Studi Kasus di permadi.dev",
					to: "https://permadi.dev/id/projek/niluji",
					target: "_blank"
				},
				{
					icon: "i-lucide-user",
					label: "Dinar Permadi (permadi.dev)",
					to: "https://permadi.dev",
					target: "_blank"
				},
				{
					icon: "i-lucide-star",
					label: "Beri Bintang di GitHub",
					to: "https://github.com/narr07/niluji",
					target: "_blank"
				}
			]
		}
	}
});
