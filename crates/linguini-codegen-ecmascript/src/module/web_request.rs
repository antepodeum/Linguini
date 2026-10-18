//! Shared request statements for the web facade.

use crate::ecmascript::RuntimeCode;

pub(super) fn append(code: &mut RuntimeCode) {
    code.shared("\n");
    code.typed("function inputFromRequest(request: Request, input: Record<string, unknown> = {}) {", "/** @param {Request} request @param {Record<string, unknown>} [input] @returns {Record<string, unknown>} */\nfunction inputFromRequest(request, input = {}) {");
    code.shared("\n");
    code.typed(
        "  const requestInput: Record<string, unknown> =",
        "  /** @type {Record<string, unknown>} */\n  const requestInput =",
    );
    code.shared(" { ...input };\n  requestInput.url ??= request.url;\n  requestInput.currentUrl ??= requestInput.url;\n  requestInput.headers ??= request.headers;\n  requestInput.cookie ??= readHeader(requestInput.headers, \"cookie\") ?? readHeader(request.headers, \"cookie\") ?? undefined;\n  return requestInput;\n}\n\n");
    code.typed("function readHeader(headers: unknown, name: string) {", "/** @param {unknown} headers @param {string} name @returns {string | undefined} */\nfunction readHeader(headers, name) {");
    code.shared("\n  if (!headers) return undefined;\n");
    code.typed("  const getter = (headers as { get?: (header: string) => string | null | undefined }).get;", "  const getter = (/** @type {{ get?: (header: string) => string | null | undefined }} */ (headers)).get;");
    code.shared("\n  if (typeof getter !== \"function\") return undefined;\n  try {\n    return getter.call(headers, name) ?? undefined;\n  } catch {\n    return undefined;\n  }\n}\n");
}
