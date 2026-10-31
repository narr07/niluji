// @ts-check
import withNuxt from "./.nuxt/eslint.config.mjs";

export default withNuxt({
	rules: {
		"vue/no-multiple-template-root": "off",
		"vue/max-attributes-per-line": ["error", { singleline: 3 }],
		"vue/block-order": ["error", { order: ["script", "template", "style"] }],
		"@stylistic/indent": ["error", "tab"],
		"@stylistic/quotes": ["error", "double"],
		"@stylistic/semi": ["error", "always"],
		"vue/script-indent": ["error", "tab", { baseIndent: 1 }],
		"vue/html-indent": ["error", "tab"]
	}
}, {
	files: ["**/*.vue"],
	rules: {
		"@stylistic/indent": "off"
	}
});
