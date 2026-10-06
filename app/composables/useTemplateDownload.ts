import { invoke } from "@tauri-apps/api/core";

// Unduh file template dari /public/templates ke folder Downloads lewat Rust (bukan <a download>),
// supaya kita tahu path persisnya dan toast bisa menawarkan tombol "Buka Folder".
export const useTemplateDownload = () => {
	const toast = useToast();

	const revealInFolder = async (path: string) => {
		try {
			await invoke("reveal_in_folder", { path });
		} catch (error) {
			toast.add({ title: "Gagal membuka folder", description: String(error), icon: "lucide:x", color: "error" });
		}
	};

	const downloadTemplate = async (url: string, fileName: string, label: string) => {
		try {
			const response = await fetch(url);
			if (!response.ok) throw new Error(`File template tidak ditemukan (status ${response.status}).`);
			const bytes = new Uint8Array(await response.arrayBuffer());
			const path = await invoke<string>("save_to_downloads", { fileName, bytes: Array.from(bytes) });

			toast.add({
				title: "Template diunduh",
				description: `${label} disimpan di ${path}`,
				icon: "lucide:download",
				color: "success",
				duration: 10000,
				actions: [{ label: "Buka Folder", icon: "lucide:folder-open", color: "neutral", variant: "outline", onClick: () => revealInFolder(path) }]
			});
		} catch (error) {
			toast.add({ title: "Gagal mengunduh template", description: error instanceof Error ? error.message : String(error), icon: "lucide:x", color: "error" });
		}
	};

	return { downloadTemplate, revealInFolder };
};
