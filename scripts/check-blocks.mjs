#!/usr/bin/env node
// Block-system gate (DESIGN.md "Block system").
// Fails on raw px/rem values in padding, margin, gap, height, min-height, font-size and
// border-radius anywhere in src/**/*.svelte and src/**/*.css except src/lib/tokens.css.
// Allowed: 0, 1px (borders/hairlines), anything written through var(--token).
// <script> blocks are skipped; <style> blocks, inline style="" and style:prop are checked.
// Usage: node scripts/check-blocks.mjs  -> prints file:line and a per-file count, exit 1 on any hit.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';

const ROOT = process.cwd();
const SRC = join(ROOT, 'src');
const EXEMPT = new Set(['src/lib/tokens.css']);

const PROPS = [
  'padding(?:-(?:top|right|bottom|left|inline|block)(?:-(?:start|end))?)?',
  'margin(?:-(?:top|right|bottom|left|inline|block)(?:-(?:start|end))?)?',
  'gap',
  'row-gap',
  'column-gap',
  'height',
  'min-height',
  'font-size',
  'border(?:-(?:top|bottom|start|end)-(?:left|right|start|end))?-radius'
].join('|');

// `prop: value` (CSS and inline style="") or `style:prop="value"` / style:prop={'value'}
const DECL = new RegExp(`(?<![\\w-])(${PROPS})\\s*:\\s*([^;{}"\\n]+)`, 'g');
const DIRECTIVE = new RegExp(`style:(${PROPS})\\s*=\\s*(?:"([^"]*)"|\\{([^}]*)\\})`, 'g');
const LENGTH = /(-?\d*\.?\d+)(px|rem)\b/g;

const walk = (dir) =>
  readdirSync(dir).flatMap((name) => {
    const p = join(dir, name);
    return statSync(p).isDirectory() ? walk(p) : [p];
  });

/** Blank out ranges (keeping newlines) so line numbers stay true. */
const blank = (text, re) => text.replace(re, (m) => m.replace(/[^\n]/g, ' '));

const prepare = (rel, text) => {
  let t = blank(text, /\/\*[\s\S]*?\*\//g); // CSS comments
  t = blank(t, /@(?:media|container|supports)[^{]*/g); // breakpoint preludes are not spacing
  if (rel.endsWith('.svelte')) {
    t = blank(t, /<script[\s\S]*?<\/script>/g);
    t = blank(t, /<!--[\s\S]*?-->/g);
  }
  return t;
};

const badValues = (value) =>
  [...value.matchAll(LENGTH)]
    .filter(([, n, unit]) => !(Number(n) === 0 || (unit === 'px' && Math.abs(Number(n)) === 1)))
    .map(([m]) => m);

const lineOf = (text, index) => text.slice(0, index).split('\n').length;

const files = walk(SRC)
  .map((p) => ({ abs: p, rel: relative(ROOT, p).split(sep).join('/') }))
  .filter((f) => /\.(svelte|css)$/.test(f.rel) && !EXEMPT.has(f.rel))
  .sort((a, b) => a.rel.localeCompare(b.rel));

const hits = [];
for (const f of files) {
  const text = prepare(f.rel, readFileSync(f.abs, 'utf8'));
  const check = (index, prop, value) => {
    const line = lineOf(text, index);
    const bad = badValues(value);
    if (bad.length) hits.push({ file: f.rel, line, text: `${prop}: ${value.trim()}`, bad });
  };
  for (const m of text.matchAll(DECL)) check(m.index, m[1], m[2]);
  for (const m of text.matchAll(DIRECTIVE)) check(m.index, `style:${m[1]}`, m[2] ?? m[3] ?? '');
}

if (!hits.length) {
  console.log(`check-blocks: OK (${files.length} files, 0 raw spacing/size values)`);
  process.exit(0);
}

for (const h of hits) console.log(`${h.file}:${h.line}  ${h.text}   <- ${h.bad.join(', ')}`);
const perFile = Object.entries(
  hits.reduce((acc, h) => ({ ...acc, [h.file]: (acc[h.file] ?? 0) + 1 }), {})
).sort((a, b) => b[1] - a[1]);
console.log('\ncheck-blocks: FAIL - raw values per file (use --space-*, --size-*, --fs-*, --radius-* from src/lib/tokens.css):');
for (const [file, n] of perFile) console.log(`  ${String(n).padStart(3)}  ${file}`);
console.log(`  ${String(hits.length).padStart(3)}  total in ${perFile.length} files`);
process.exit(1);
