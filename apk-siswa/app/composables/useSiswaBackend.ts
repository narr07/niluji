import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
	version: string;
	mobile: boolean;
}

export interface ServerCheck {
	baseUrl: string;
	examUrl: string;
}

export type ScanResult
	= | { status: "ok"; content: string }
		| { status: "denied" }
		| { status: "cancelled" };

// Satu-satunya tempat yang memanggil backend Tauri. Halaman cukup memakai fungsi di sini.
export const useSiswaBackend = () => {
	const appInfo = () => invoke<AppInfo>("app_info");

	const checkServer = (address: string) => invoke<ServerCheck>("check_server", { address });

	const openExam = (address: string) => invoke<null>("open_exam", { address });

	// Plugin pemindai hanya ada di Android/iOS, jadi dimuat saat dibutuhkan saja.
	const scanQr = async (): Promise<ScanResult> => {
		const scanner = await import("@tauri-apps/plugin-barcode-scanner");
		let permission = await scanner.checkPermissions();
		if (permission !== "granted") permission = await scanner.requestPermissions();
		if (permission !== "granted") return { status: "denied" };
		try {
			const result = await scanner.scan({ windowed: false, cameraDirection: "back", formats: [scanner.Format.QRCode] });
			return { status: "ok", content: result.content };
		} catch {
			return { status: "cancelled" };
		}
	};

	const openCameraSettings = async () => {
		const scanner = await import("@tauri-apps/plugin-barcode-scanner");
		await scanner.openAppSettings();
	};

	return { appInfo, checkServer, openExam, scanQr, openCameraSettings };
};
