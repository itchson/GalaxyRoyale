import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const lexicon = JSON.parse(readFileSync(new URL('../../assets/narrative/lore-lexicon.json', import.meta.url), 'utf8'));
const documentation = readFileSync(new URL('../../docs/LORE_LEXICON.md', import.meta.url), 'utf8');
const ascii = /^[\x20-\x7e]+$/;

test('canonical lore terms have distinct ASCII labels and functional English introductions', () => {
  assert.equal(lexicon.version, 1);
  assert.deepEqual(lexicon.terms.map((term) => term.id), ['kikai', 'kessho', 'raicho']);
  assert.deepEqual(lexicon.terms.map((term) => term.display), ['KIKAI', 'KESSHO', 'RAICHO']);
  assert.deepEqual(lexicon.terms.map((term) => term.introduction), [
    'KIKAI / technological species', 'KESSHO / crystal', 'RAICHO / flight craft',
  ]);
  for (const term of lexicon.terms) {
    assert.match(term.display, /^[A-Z]+$/);
    assert.match(term.introduction, ascii);
    assert.ok(term.introduction.length <= 64);
    assert.ok(term.lexical_sense && term.game_sense);
  }
});

test('source readings preserve long vowels while canonical labels stay user ASCII', () => {
  const byId = Object.fromEntries(lexicon.terms.map((term) => [term.id, term]));
  assert.deepEqual(byId.kessho.japanese, { written: '結晶', kana: 'けっしょう', romanization: 'kesshō', source_ascii: 'Kessho' });
  assert.deepEqual(byId.raicho.japanese, { written: '雷鳥', kana: 'らいちょう', romanization: 'raichō' });
  assert.equal(byId.kikai.japanese.romanization, 'kikai');
  assert.equal(byId.kikai.fictional_extension, true);
  assert.equal(byId.raicho.fictional_extension, true);
  assert.match(byId.raicho.lexical_sense, /rock ptarmigan/);
  assert.match(byId.raicho.usage_guard, /not the standard Japanese word for spacecraft/);
  assert.match(byId.raicho.usage_guard, /mythological Thunderbird/);
});

test('starter display models retain the exact manufacturer, hull and ability mapping', () => {
  assert.deepEqual(lexicon.starter_models.map((model) => [
    model.corporation, model.display_name, model.starter_hull_id, model.starter_ability_id, model.role,
  ]), [
    ['pan', 'PAN HIKARI', 0, 'flux-drive', 'Passive'],
    ['renko', 'RENKO HIBANA', 0, 'pulse-lens', 'Offensive'],
    ['hugo', 'HUGO MAMORI', 0, 'guard-coil', 'Defensive'],
  ]);
  for (const model of lexicon.starter_models) {
    assert.equal(model.display_name, `${model.corporation.toUpperCase()} ${model.model}`);
    assert.equal(model.japanese.romanization, model.model.toLowerCase());
    assert.match(model.introduction, ascii);
    assert.ok(model.introduction.length <= 64);
    assert.match(model.introduction, /RAICHO flight craft$/);
  }
});

test('historical aliases are exact prefixed labels and never rename champion Bulwark', () => {
  assert.deepEqual(lexicon.starter_models.map((model) => model.historical_display_aliases), [
    ['PAN Pathfinder', 'PAN PATHFINDER'],
    ['RENKO Lancer', 'RENKO LANCER'],
    ['HUGO Bulwark', 'HUGO BULWARK'],
  ]);
  const aliases = new Map();
  for (const model of lexicon.starter_models) {
    for (const alias of model.historical_display_aliases) {
      assert.ok(alias.startsWith(`${model.corporation.toUpperCase()} `));
      const normalized = alias.toUpperCase();
      const prior = aliases.get(normalized);
      assert.ok(prior === undefined || prior === model.corporation);
      aliases.set(normalized, model.corporation);
    }
  }
  assert.equal(aliases.has('BULWARK'), false);
  assert.deepEqual(lexicon.identity_rules.protected_champion_display_names, ['Bulwark']);
  assert.match(lexicon.display_policy.alias_scope, /not substring replacements or save identifiers/);
});

test('each lexical claim resolves to an identified checked government or publisher dictionary source', () => {
  const sources = new Map(lexicon.sources.map((source) => [source.id, source]));
  assert.equal(sources.size, lexicon.sources.length);
  const used = new Set();
  for (const entry of [...lexicon.terms, ...lexicon.starter_models]) {
    assert.ok(entry.sources.length > 0);
    for (const id of entry.sources) {
      assert.ok(sources.has(id), `Missing source ${id}`);
      used.add(id);
    }
  }
  assert.equal(used.size, sources.size);
  for (const source of sources.values()) {
    const url = new URL(source.url);
    assert.equal(url.protocol, 'https:');
    assert.ok(['kotobank.jp', 'id.ndl.go.jp', 'www.mlit.go.jp'].includes(url.hostname));
    assert.equal(source.checked_on, '2026-10-11');
    assert.ok(source.publisher && source.supports);
    if (url.hostname === 'kotobank.jp') assert.match(source.publisher, /Shogakukan/);
    assert.ok(documentation.includes(source.url), `Documentation omits ${source.id}`);
  }
});

test('display wording preserves license/progression identifiers and sponsor independence', () => {
  assert.equal(lexicon.identity_rules.save_migration_required, false);
  assert.deepEqual(lexicon.identity_rules.unchanged_identifiers, [
    'pan', 'renko', 'hugo', 'dora', 'doro', 'talo', 'hex_id', 'selected_ship',
    'ship_xp', 'loadouts', 'pulse-lens', 'guard-coil', 'flux-drive',
  ]);
  assert.match(lexicon.identity_rules.sub_manufacturer, /no extra starter ability or combat bonus/);
  assert.match(lexicon.identity_rules.sponsor, /independent of main\/sub manufacturers, appearance, license and ability grants/);
  assert.match(lexicon.identity_rules.rename_scope, /separate authored review/);
  assert.match(lexicon.status, /runtime integration is owned separately/);
});

test('documentation agrees with data and distinguishes authoring tests from runtime acceptance', () => {
  for (const entry of [...lexicon.terms, ...lexicon.starter_models]) {
    assert.ok(documentation.includes(entry.display ?? entry.display_name));
    assert.ok(documentation.includes(entry.japanese.romanization));
  }
  assert.match(documentation, /does not prove that runtime menus have adopted these labels/);
  assert.match(documentation, /not substring replacement rules/);
  assert.match(documentation, /No save migration is required/);
  assert.match(documentation, /User canon governs the fiction/);
});
