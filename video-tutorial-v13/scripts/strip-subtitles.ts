import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const dir = join(process.cwd(), "compositions");
for (const f of readdirSync(dir).filter((x) => x.endsWith(".html"))) {
  const p = join(dir, f);
  let s = readFileSync(p, "utf8");
  s = s.replace(/\s*<div id="sub-\d+" class="subtitle-wrap[\s\S]*?<\/div>\s*<\/div>/g, "");
  s = s.replace(/^[ \t]*tl\.(?:fromTo|to)\("#sub-\d+".*\r?\n/gm, "");
  writeFileSync(p, s, "utf8");
  console.log("Cleaned:", f);
}
