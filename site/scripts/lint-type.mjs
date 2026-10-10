// Type gate (spec LP2): typography is declared only in src/styles/tokens.css.
// Fails (exit 1) on raw font-size / line-height / letter-spacing / font-weight /
// font-family (plus the `font` shorthand and font-variation-settings) anywhere
// else, in any style language: CSS declarations, quoted or kebab object keys,
// `el.style.x =`, `style["x"] =`, `setProperty("x")`, CSS inside strings and
// template literals. Inside the token file only weights 400 and 600 may appear
// (font-weight, the font shorthand, the wght axis).
//
// Exempt: src/components/app-screen.module.css. It draws the app's own UI at
// 1440 x 900 inside the MacBook, a picture of another product's typography
// rather than page typography. It is the only exemption.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";

const root = new URL("../src", import.meta.url).pathname;
const TOKENS = "styles/tokens.css";
const EXEMPT = new Set(["components/app-screen.module.css"]);
const ALLOWED_WEIGHTS = new Set(["400", "600"]);

// Comment syntax per language. Strings are tokenized first so a `//` or `/*`
// inside a quote never counts as a comment.
const JS = { block: true, line: true, template: true };
const LANGS = new Map([
  ["css", { block: true }],
  ["scss", { block: true, line: true, urls: true }],
  ["sass", { block: true, line: true, urls: true }],
  ["less", { block: true, line: true, urls: true }],
  ["html", { block: true, html: true }],
  ["mdx", { block: true, html: true }],
  ...["js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts"].map((e) => [e, JS]),
]);
const extOf = (name) => name.slice(name.lastIndexOf(".") + 1).toLowerCase();

// Raw typography properties, kebab (CSS) and camel (JS style objects).
const KEBAB = "font-size|line-height|letter-spacing|font-weight|font-family|font-variation-settings";
const CAMEL = "fontSize|lineHeight|letterSpacing|fontWeight|fontFamily|fontVariationSettings";
const ANY = `${KEBAB}|${CAMEL}|font`;
const Q = "[\"'`]";
const RAW = [
  // CSS decl, bare/quoted/computed object key, CSS text inside a string.
  new RegExp(`(?<![\\w-])(?<q>${Q}?)(?<p>${ANY})\\k<q>\\]?\\s*:`, "gi"),
  // el.style.fontSize = ...
  new RegExp(`\\.\\s*(?<p>${CAMEL}|font)\\s*=(?!=)`, "gi"),
  // el.style["font-size"] = ...
  new RegExp(`\\[\\s*(?<q>${Q})(?<p>${ANY})\\k<q>\\s*\\]\\s*=(?!=)`, "gi"),
  // el.style.setProperty("font-size", ...)
  new RegExp(`setProperty\\s*\\(\\s*(?<q>${Q})(?<p>${ANY})\\k<q>`, "gi"),
];

function* walk(dir) {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) yield* walk(path);
    else if (LANGS.has(extOf(name))) yield path;
  }
}

// Blank out real comments (keep newlines so line numbers stay correct) and
// leave string contents untouched.
function stripComments(text, lang) {
  const blank = (s) => s.replace(/[^\n]/g, " ");
  const tpl = []; // open `${` depths, innermost last
  let out = "";
  let quote = null;
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    const two = text.slice(i, i + 2);
    if (quote) {
      if (c === "\\") { out += text.slice(i, i + 2); i += 2; continue; }
      if (c === quote) quote = null;
      else if (c === "\n" && quote !== "`") quote = null; // unterminated: bound the damage
      else if (quote === "`" && two === "${") { tpl.push(0); quote = null; out += two; i += 2; continue; }
      out += c; i++; continue;
    }
    if (c === '"' || c === "'" || (c === "`" && lang.template)) { quote = c; out += c; i++; continue; }
    if (tpl.length) {
      if (c === "{") tpl[tpl.length - 1]++;
      else if (c === "}") {
        if (tpl[tpl.length - 1] === 0) { tpl.pop(); quote = "`"; } else tpl[tpl.length - 1]--;
      }
    }
    const until = (marker, from, len) => {
      const j = text.indexOf(marker, from);
      return j === -1 ? text.length : j + len;
    };
    let end = -1;
    if (lang.block && two === "/*") end = until("*/", i + 2, 2);
    else if (lang.html && text.startsWith("<!--", i)) end = until("-->", i + 4, 3);
    else if (lang.line && two === "//" && !(lang.urls && /(?:url\(\s*|:)$/.test(out))) end = until("\n", i, 0);
    if (end !== -1) { out += blank(text.slice(i, end)); i = end; continue; }
    out += c; i++;
  }
  return out;
}

const lineAt = (text, index) => text.slice(0, index).split("\n").length;
const clean = (v) => v.replace(/\s*!\s*important\s*$/i, "").trim().toLowerCase();

const SIZE = /^(?:\d*\.?\d+[a-z%]+|(?:xx?x?-)?(?:small|large)|medium|smaller|larger)$/;
// Weight written inside a `font` shorthand: a bare number or keyword before the size.
function shorthandWeight(value) {
  for (const tok of value.split(/[\s/]+/)) {
    if (SIZE.test(tok)) return null;
    if (/^(?:\d+|bold|bolder|lighter)$/.test(tok)) return tok;
  }
  return null;
}

// Inside the token file: nothing but weights 400 / 600, however they are spelled.
function checkTokenWeights(rel, text, add) {
  const decl = (prop) => text.matchAll(new RegExp(`(?<![\\w-])${prop}\\s*:\\s*([^;}]+)`, "gi"));
  for (const m of decl("font-weight")) {
    const v = clean(m[1]);
    if (!ALLOWED_WEIGHTS.has(v)) add(rel, m.index, `font-weight ${v} (only 400 and 600 are allowed)`);
  }
  for (const m of decl("font")) {
    const v = clean(m[1]);
    const w = /var\(|env\(/.test(v) ? v : shorthandWeight(v);
    if (w !== null && !ALLOWED_WEIGHTS.has(w)) {
      add(rel, m.index, `font shorthand weight ${w} (only 400 and 600 are allowed)`);
    }
  }
  for (const m of decl("font-variation-settings")) {
    const v = clean(m[1]);
    if (/var\(|env\(/.test(v)) add(rel, m.index, `font-variation-settings ${v} cannot be verified (only wght 400 and 600)`);
    for (const a of v.matchAll(/["']wght["']\s*([^,\s;}]+)/g)) {
      if (!ALLOWED_WEIGHTS.has(a[1])) add(rel, m.index, `font-variation-settings wght ${a[1]} (only 400 and 600 are allowed)`);
    }
  }
}

const violations = [];
for (const file of walk(root)) {
  const rel = relative(root, file).split(sep).join("/");
  if (EXEMPT.has(rel)) continue;
  const text = stripComments(readFileSync(file, "utf8"), LANGS.get(extOf(file)));
  const add = (r, index, msg) => violations.push(`${r}:${lineAt(text, index)} ${msg}`);

  if (rel === TOKENS) {
    checkTokenWeights(rel, text, add);
    continue;
  }
  const seen = new Set();
  for (const re of RAW) {
    for (const m of text.matchAll(re)) {
      const key = `${m.index}:${m.groups.p.toLowerCase()}`;
      if (seen.has(key)) continue;
      seen.add(key);
      add(rel, m.index, `raw ${m.groups.p} outside ${TOKENS}`);
    }
  }
}

if (violations.length) {
  console.error(`lint:type failed (${violations.length})`);
  for (const v of violations) console.error(`  ${v}`);
  process.exit(1);
}
console.log("lint:type ok");
