import { invoke, isTauri } from "@tauri-apps/api/core";

export interface DapodikConfig {
	host: string
	port: number
	token: string
	npsn: string
	semesterId: string
	timeoutSecs: number
	mock: boolean
}

export interface DapodikResult {
	path: string
	url: string
	status: number
	elapsedMs: number
	sizeBytes: number
	contentType: string
	body: string
	json: any
	message?: string
	error?: string
	at: string
	mock: boolean
}

interface RawResponse {
	url: string
	status: number
	elapsedMs: number
	sizeBytes: number
	contentType: string
	body: string
}

const STORAGE_KEY = "nuxt-erapor:config";

const DEFAULT_CONFIG: DapodikConfig = {
	host: "localhost",
	port: 5774,
	token: "",
	npsn: "",
	semesterId: "",
	timeoutSecs: 30,
	mock: false
};

function loadConfig(): DapodikConfig {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		return raw ? { ...DEFAULT_CONFIG, ...JSON.parse(raw) } : { ...DEFAULT_CONFIG };
	}
	catch {
		return { ...DEFAULT_CONFIG };
	}
}

// Web Service Dapodik selalu menjawab HTTP 200 di level transport, lalu menaruh status asli
// sebagai baris "HTTP/1.0 403 Forbidden" + header di awal body, baru JSON-nya setelah baris kosong.
// Ambil status dari baris itu dan buang header-nya supaya body tinggal JSON.
export function unwrapDapodikBody(transportStatus: number, body: string): { status: number, body: string } {
	const m = body.match(/^HTTP\/\d(?:\.\d)?\s+(\d{3})/);
	if (!m)
		return { status: transportStatus, body };
	const split = body.search(/\r?\n\r?\n/);
	return {
		status: Number(m[1]),
		body: split === -1 ? "" : body.slice(split).trimStart()
	};
}

// getPengguna ikut mengirim hash password (bcrypt) — jangan ditampilkan atau ikut tersimpan di sampel.
const SECRET_KEYS = new Set(["password"]);

export function maskSecrets<T>(value: T): T {
	if (Array.isArray(value))
		return value.map(maskSecrets) as T;
	if (value && typeof value === "object") {
		return Object.fromEntries(Object.entries(value).map(([k, v]) =>
			[k, SECRET_KEYS.has(k) && v ? "•••" : maskSecrets(v)]
		)) as T;
	}
	return value;
}

// Penjelasan singkat status HTTP yang sering muncul saat uji Web Service Dapodik.
export function explainStatus(status: number): string {
	if (status >= 200 && status < 300)
		return "Berhasil";
	if (status === 401)
		return "Token ditolak — cek ulang token Web Service di Dapodik";
	if (status === 403)
		return "Ditolak Dapodik — cek NPSN, token, dan IP di Pengaturan → Web Service";
	if (status === 404)
		return "Endpoint tidak ditemukan — nama endpoint mungkin beda di versi Dapodik ini";
	if (status >= 500)
		return "Error di server Dapodik";
	return `HTTP ${status}`;
}

const config = ref<DapodikConfig>(loadConfig());
const results = ref<Record<string, DapodikResult>>({});
const loading = ref<Record<string, boolean>>({});

watch(config, (val) => {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
	}
	catch {}
}, { deep: true });

export function useDapodik() {
	const baseUrl = computed(() => {
		const host = config.value.host.trim().replace(/^https?:\/\//, "").replace(/\/+$/, "");
		return `http://${host}:${config.value.port}`;
	});

	const inTauri = isTauri();

	async function request(path: string): Promise<DapodikResult> {
		loading.value[path] = true;
		const at = new Date().toLocaleTimeString("id-ID");
		try {
			let raw: RawResponse;
			if (config.value.mock) {
				await new Promise(r => setTimeout(r, 300));
				const data = MOCK_DAPODIK[path];
				const body = data ? JSON.stringify(data) : JSON.stringify({ error: "Endpoint ini tidak ada di data mock" });
				raw = {
					url: `mock://${path}?npsn=${config.value.npsn}`,
					status: data ? 200 : 404,
					elapsedMs: 300,
					sizeBytes: body.length,
					contentType: "application/json",
					body
				};
			}
			else {
				if (!inTauri)
					throw new Error("Request asli hanya jalan di jendela Tauri (bun run tauri:dev). Di browser biasa, nyalakan Mode Mock.");
				raw = await invoke<RawResponse>("dapodik_request", {
					req: {
						baseUrl: baseUrl.value,
						path,
						token: config.value.token,
						npsn: config.value.npsn,
						semesterId: config.value.semesterId,
						timeoutSecs: config.value.timeoutSecs
					}
				});
			}

			const unwrapped = unwrapDapodikBody(raw.status, raw.body);
			const body = unwrapped.body;
			let status = unwrapped.status;
			let json: any = null;
			try {
				json = JSON.parse(body);
			}
			catch {}

			// Pesan error Dapodik ada di field "message" (mis. "Parameter NPSN harap diisi").
			// Kadang status error cuma ada di JSON-nya, tanpa baris "HTTP/1.0 ..." di depan.
			const message = json && json.success === false ? json.message : undefined;
			if (message && status < 300)
				status = Number(json.http_code) || 400;
			const result: DapodikResult = { ...raw, status, body, path, json, message, at, mock: config.value.mock };
			results.value[path] = result;
			return result;
		}
		catch (e: any) {
			const result: DapodikResult = {
				path,
				url: `${baseUrl.value}/${path}`,
				status: 0,
				elapsedMs: 0,
				sizeBytes: 0,
				contentType: "",
				body: "",
				json: null,
				error: typeof e === "string" ? e : e?.message ?? pesanError(e),
				at,
				mock: config.value.mock
			};
			results.value[path] = result;
			return result;
		}
		finally {
			loading.value[path] = false;
		}
	}

	async function saveSample(result: DapodikResult): Promise<string> {
		const name = result.path.split("/").pop() || "response";
		const content = result.json ? JSON.stringify(maskSecrets(result.json), null, 2) : result.body;
		if (!inTauri) {
			const blob = new Blob([content], { type: "application/json" });
			const a = document.createElement("a");
			a.href = URL.createObjectURL(blob);
			a.download = `${name}.json`;
			a.click();
			URL.revokeObjectURL(a.href);
			return `${name}.json (diunduh browser)`;
		}
		return invoke<string>("save_sample", { name, content });
	}

	async function openSamplesDir(): Promise<string> {
		return invoke<string>("open_samples_dir");
	}

	return { config, results, loading, baseUrl, inTauri, request, saveSample, openSamplesDir };
}
