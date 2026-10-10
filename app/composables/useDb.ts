import { invoke } from "@tauri-apps/api/core";

export interface SqlStatement {
	sql: string
	params?: unknown[]
}

// Pembungkus perintah database di sisi Rust (src-tauri/src/db.rs).
export function useDb() {
	function query<T = Record<string, any>>(sql: string, params: unknown[] = []): Promise<T[]> {
		return invoke<T[]>("db_query", { sql, params });
	}

	async function first<T = Record<string, any>>(sql: string, params: unknown[] = []): Promise<T | undefined> {
		return (await query<T>(sql, params))[0];
	}

	async function scalar<T = number>(sql: string, params: unknown[] = []): Promise<T> {
		const row = await first(sql, params);
		return (row ? Object.values(row)[0] : undefined) as T;
	}

	function execute(sql: string, params: unknown[] = []): Promise<number> {
		return invoke<number>("db_execute", { sql, params });
	}

	function batch(statements: SqlStatement[]): Promise<number> {
		return invoke<number>("db_batch", { statements: statements.map(s => ({ sql: s.sql, params: s.params ?? [] })) });
	}

	async function getSetting(key: string): Promise<string | undefined> {
		return (await first<{ value: string }>("SELECT value FROM settings WHERE key = ?", [key]))?.value;
	}

	function setSetting(key: string, value: string) {
		return execute("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value", [key, value]);
	}

	return { query, first, scalar, execute, batch, getSetting, setSetting };
}
