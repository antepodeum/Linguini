//! Schema helper bodies shared by both ECMAScript targets.

use crate::ecmascript::EcmaScriptTarget;

use super::runtime_code::RuntimeCode;

pub(super) fn shared_runtime(target: EcmaScriptTarget) -> String {
    let mut code = RuntimeCode::default();
    code.typed(
        "export function selectBranch<T>(\n  key: string,\n  branches: Record<string, T>,\n): T {\n",
        "/**\n * @template T\n * @param {string} key\n * @param {Record<string, T>} branches\n * @returns {T}\n */\nexport function selectBranch(key, branches) {\n",
    );
    code.shared(
        r#"  if (Object.prototype.hasOwnProperty.call(branches, key)) {
    return branches[key];
  }
  if (Object.prototype.hasOwnProperty.call(branches, "_")) {
    return branches._;
  }
  if (Object.prototype.hasOwnProperty.call(branches, "other")) {
    return branches.other;
  }
  throw new Error(`Linguini dispatch has no branch for key ${JSON.stringify(key)}`);
}

"#,
    );
    code.typed(
        "export function normalizeMessageArgs(\n  values: readonly unknown[],\n  keys: readonly string[],\n): unknown[] {\n",
        "/**\n * @param {readonly unknown[]} values\n * @param {readonly string[]} keys\n * @returns {unknown[]}\n */\nexport function normalizeMessageArgs(values, keys) {\n",
    );
    code.shared(
        r#"  if (values.length !== 1) {
    return [...values];
  }
  const candidate = values[0];
  if (typeof candidate !== "object" || candidate === null) {
    return [...values];
  }
"#,
    );
    code.typed(
        "  const record = candidate as Record<string, unknown>;\n",
        "  const record = /** @type {Record<string, unknown>} */ (candidate);\n",
    );
    code.shared(
        r#"  if (!keys.every((key) => Object.prototype.hasOwnProperty.call(record, key))) {
    return [...values];
  }
  const unknown = Reflect.ownKeys(record)
    .filter((key) => typeof key !== "string" || !keys.includes(key))
    .map(String)
    .sort();
  if (unknown.length > 0) {
    throw new TypeError(
      `Linguini message arguments contain unknown keys: ${unknown.join(", ")}`,
    );
  }
  return keys.map((key) => record[key]);
}
"#,
    );
    code.render(target)
}

pub(super) fn shared_declarations() -> &'static str {
    r#"export declare function selectBranch<T>(
  key: string,
  branches: Record<string, T>,
): T;

export declare function normalizeMessageArgs(
  values: readonly unknown[],
  keys: readonly string[],
): unknown[];
"#
}
