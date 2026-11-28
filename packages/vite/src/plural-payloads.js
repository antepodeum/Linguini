import { realpathSync } from "node:fs";
import { toVitePath } from "./paths.js";

const categoryNames = new Set(["zero", "one", "two", "few", "many"]);
const originalQuery = "?linguini-plural-original";
const virtualPrefix = "\0linguini-page-plural/";
const key = categories => categories.join(",");
const record = value => value !== null && typeof value === "object" && !Array.isArray(value);

function categories(value, field, available = categoryNames) {
  if (!Array.isArray(value) || value.some(item => !available.has(item)) ||
      new Set(value).size !== value.length || key(value) !== key([...value].sort())) {
    throw new Error(`${field} must contain unique sorted plural categories`);
  }
  return Object.freeze([...value]);
}

/** Validate optional compiler metadata; older prebuilt projects retain full classifiers. */
export function validatePluralRuntimes(raw, { messages, runtimes, resolvePath, context }) {
  const plans = new Map();
  if (raw === undefined) return plans;
  if (!record(raw)) throw new Error(`${context}.plural_runtimes must be an object`);
  const modules = new Set([...runtimes.values()].map(runtime => runtime.module));
  for (const [locale, entry] of Object.entries(raw)) {
    const field = `${context}.plural_runtimes.${locale}`;
    if (!runtimes.has(locale) || !record(entry)) throw new Error(`${field} is not a locale runtime`);
    const module = resolvePath(entry.module, field + ".module");
    if (module !== runtimes.get(locale).module) throw new Error(`${field}.module must match its locale runtime`);
    if (typeof entry.binding !== "string" || !/^[$A-Z_a-z][$\w]*$/.test(entry.binding)) {
      throw new Error(`${field}.binding must be an ECMAScript identifier`);
    }
    const available = new Set(categories(entry.categories, field + ".categories"));
    if (!available.size || !record(entry.messages) || !Array.isArray(entry.variants)) {
      throw new Error(`${field} must contain categories, messages, and variants`);
    }
    const demands = new Map();
    for (const [name, demand] of Object.entries(entry.messages)) {
      if (!messages.get(name)?.locales.has(locale)) throw new Error(`${field}.messages.${name} has no locale implementation`);
      demands.set(name, demand === null ? null : categories(demand, field + ".messages." + name, available));
    }
    for (const [name, message] of messages) {
      if (message.locales.has(locale) && !demands.has(name)) throw new Error(`${field}.messages is missing ${name}`);
    }
    const variants = new Map();
    for (const [index, variant] of entry.variants.entries()) {
      const variantField = `${field}.variants.${index}`;
      if (!record(variant)) throw new Error(`${variantField} must be an object`);
      const demand = categories(variant.categories, variantField + ".categories", available);
      const module = resolvePath(variant.module, variantField + ".module");
      if (!module.endsWith(".js") || variants.has(key(demand)) || modules.has(module)) {
        throw new Error(`${variantField} must have unique categories and a unique JavaScript module`);
      }
      modules.add(module);
      variants.set(key(demand), module);
    }
    // At most 32 category unions exist. Validate all reachable subsets once at build setup.
    const unions = new Map();
    for (const demand of demands.values()) {
      if (demand === null) continue;
      for (const prior of [...unions.values(), []]) {
        const union = [...new Set([...prior, ...demand])].sort();
        unions.set(key(union), union);
      }
    }
    for (const [union, demand] of unions) {
      if (demand.length < available.size && !variants.has(union)) {
        throw new Error(`${field}.variants is missing category union ${union || "(fallback)"}`);
      }
    }
    plans.set(locale, Object.freeze({ module, binding: entry.binding, messages: demands, variants }));
  }
  return plans;
}

/** Override just the classifier export; all formatter exports retain their original owner. */
export function pagePluralPlugin(manifest, names, locale) {
  const selected = new Map();
  const wrappers = new Map();
  for (const [origin, plan] of manifest.pluralRuntimes ?? []) {
    const messages = names.filter(name => {
      const implementations = manifest.messages.get(name)?.locales;
      return (implementations?.has(locale) ? locale : manifest.baseLocale) === origin;
    });
    if (!messages.length || messages.some(name => plan.messages.get(name) === null)) continue;
    const demand = [...new Set(messages.flatMap(name => {
      const categories = plan.messages.get(name);
      if (!categories) throw new Error(`Linguini plural metadata is missing ${name}/${origin}`);
      return categories;
    }))].sort();
    const variant = plan.variants.get(key(demand));
    // A union containing all numeric categories uses the complete runtime.
    if (!variant) continue;
    const id = virtualPrefix + wrappers.size;
    selected.set(toVitePath(plan.module), id);
    selected.set(toVitePath(realpathSync(plan.module)), id);
    wrappers.set(id, `export * from ${JSON.stringify(toVitePath(plan.module) + originalQuery)};\nexport { ${plan.binding} } from ${JSON.stringify(toVitePath(variant))};\n`);
  }
  return {
    name: "linguini-page-plurals",
    enforce: "pre",
    async resolveId(source, importer) {
      if (wrappers.has(source)) return source;
      if (!selected.size || source.includes(originalQuery)) return;
      const resolved = await this.resolve(source, importer, { skipSelf: true });
      if (resolved && !resolved.external) return selected.get(toVitePath(resolved.id));
    },
    load(id) { return wrappers.get(id); },
  };
}
