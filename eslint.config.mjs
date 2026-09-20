// @ts-check
import withNuxt from "./.nuxt/eslint.config.mjs";
import stylistic from "@stylistic/eslint-plugin";
import betterTailwindcss from "eslint-plugin-better-tailwindcss";
import { getDefaultAttributes } from "eslint-plugin-better-tailwindcss/api/defaults";

export default withNuxt(
	{
		plugins: { style: stylistic },
		rules: {
			"vue/no-multiple-template-root": "off",
			"vue/max-attributes-per-line": ["error", { singleline: 3 }],
			"vue/block-order": ["error", { order: ["script", "template", "style"] }],
			"style/indent": ["error", "tab"],
			"style/quotes": ["error", "double"],
			"style/semi": ["error", "always"],
			"vue/script-indent": ["error", "tab", { baseIndent: 1 }],
			"vue/html-indent": ["error", "tab"]
		}
	},
	{
		files: ["**/*.vue"],
		rules: {
			"style/indent": "off"
		}
	},
	betterTailwindcss.configs["correctness-error"],
	{
		settings: {
			"better-tailwindcss": {
				entryPoint: "app/assets/css/main.css",
				attributes: [
					...getDefaultAttributes(),
					["^v-bind:ui$", [{ match: "objectValues" }]]
				]
			}
		}
	}
);
