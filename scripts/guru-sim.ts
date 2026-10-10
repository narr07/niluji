// Jalankan jendela "laptop guru" kedua di komputer yang sama, dengan database terpisah,
// untuk mencoba alur Sesi Online → tarik data → kirim nilai tanpa laptop kedua.
// Syarat: `bun run tauri:dev` (laptop admin) sudah berjalan, karena jendela ini memakai dev server yang sama.
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

const root = join(import.meta.dir, "..");
const exe = join(root, "src-tauri", "target", "debug", process.platform === "win32" ? "niluji.exe" : "niluji");
const dataDir = join(root, ".guru-sim");

if (!existsSync(exe)) {
	console.error("Belum ada build debug. Jalankan dulu: bun run tauri:dev");
	process.exit(1);
}
if (process.argv.includes("--reset")) {
	rmSync(dataDir, { recursive: true, force: true });
	console.log("Database laptop guru simulasi dihapus.");
}
mkdirSync(dataDir, { recursive: true });
console.log(`Laptop guru simulasi — database: ${dataDir}`);
spawn(exe, [], { env: { ...process.env, ERAPOR_DATA_DIR: dataDir }, stdio: "inherit", detached: true }).unref();
