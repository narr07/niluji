// Status "sudah ada data Dapodik atau belum" — menentukan menu admin yang tampil.
export function useAppData() {
	const hasData = useState<boolean>("has-data", () => false);

	async function refresh() {
		try {
			hasData.value = (await useDb().scalar<number>("SELECT COUNT(*) FROM rombel")) > 0;
		}
		catch {
			hasData.value = false;
		}
	}

	return { hasData, refresh };
}
