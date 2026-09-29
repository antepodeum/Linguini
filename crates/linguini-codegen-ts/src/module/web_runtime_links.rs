//! Owned, bounded browser link-localization runtime.

use super::runtime_code::RuntimeCode;
use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaModuleOutput, EcmaNamedImport,
    EcmaScriptTarget, EcmaStatement, RenderedEcmaModule,
};

pub(super) fn generate_typescript_web_runtime_links_module() -> String {
    runtime_links_module(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/runtime-links.ts",
        Some("web/runtime-links.d.ts".to_owned()),
    ))
    .render_code()
}

pub(super) fn generate_web_runtime_links_declaration() -> String {
    let mut module = EcmaModule::new(EcmaModuleOutput::new(
        EcmaScriptTarget::TypeScript,
        "web/runtime-links.d.ts",
        None,
    ));
    module.push_import(web_type_import());
    module.push_statement(EcmaStatement::type_declaration(
        r#"export declare function startRuntimeLinkLocalization<Locale extends string>(
  web: LinguiniWebLocale<Locale>,
  getLocale: () => Locale,
): { refresh(): void; destroy(): void } | undefined;
"#,
        None,
    ));
    module.render_code()
}

/// Compile the owned browser link-localization runtime as TypeScript.
pub fn compile_typescript_web_runtime_links_module() -> RenderedEcmaModule {
    compile_runtime_links(EcmaScriptTarget::TypeScript)
}

/// Compile the owned browser link-localization runtime as checked JavaScript.
pub fn compile_javascript_web_runtime_links_module() -> RenderedEcmaModule {
    compile_runtime_links(EcmaScriptTarget::JavaScript)
}

fn compile_runtime_links(target: EcmaScriptTarget) -> RenderedEcmaModule {
    let path = if target.is_typescript() {
        "web/runtime-links.ts"
    } else {
        "web/runtime-links.js"
    };
    runtime_links_module(EcmaModuleOutput::new(target, path, None)).render(&[])
}

fn web_type_import() -> EcmaImport {
    EcmaImport {
        specifier: "../web".to_owned(),
        bindings: EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
            "LinguiniWebLocale",
            "LinguiniWebLocale",
        )]),
    }
}

fn runtime_links_module(output: EcmaModuleOutput) -> EcmaModule {
    let target = output.target();
    let mut module = EcmaModule::new(output);
    module.push_import(web_type_import());
    let mut code = RuntimeCode::default();
    code.shared("const MAX_PENDING_ROOTS = 128;\nconst NODE_BUDGET = 256;\n\n");
    code.typed(
        "export function startRuntimeLinkLocalization<LocaleName extends string>(\n  web: LinguiniWebLocale<LocaleName>,\n  getLocale: () => LocaleName,\n) {\n",
        "/**\n * @template {string} LocaleName\n * @param {import(\"../web.js\").LinguiniWebLocale<LocaleName>} web\n * @param {() => LocaleName} getLocale\n * @returns {{refresh(): void, destroy(): void} | undefined}\n */\nexport function startRuntimeLinkLocalization(web, getLocale) {\n",
    );
    code.shared("  if (typeof document === \"undefined\") return undefined;\n");
    code.typed(
        "  const pendingRoots: Node[] = [];\n  const queuedRoots = new Set<Node>();\n  let activeTraversal: LinkTraversal | undefined;\n  let animationFrame: number | undefined;\n  let timer: ReturnType<typeof setTimeout> | undefined;\n",
        "  /** @type {Node[]} */ const pendingRoots = [];\n  /** @type {Set<Node>} */ const queuedRoots = new Set();\n  /** @type {LinkTraversal | undefined} */ let activeTraversal;\n  /** @type {number | undefined} */ let animationFrame;\n  /** @type {ReturnType<typeof setTimeout> | undefined} */ let timer;\n",
    );
    code.shared(
        r#"  let destroyed = false;

  const schedule = () => {
    if (destroyed || animationFrame !== undefined || timer !== undefined) return;
    if (typeof requestAnimationFrame === "function") {
      animationFrame = requestAnimationFrame(() => {
        animationFrame = undefined;
        flush();
      });
    } else {
      timer = setTimeout(() => {
        timer = undefined;
        flush();
      }, 0);
    }
  };

"#,
    );
    code.typed(
        "  const enqueueRoot = (root: Node) => {\n",
        "  /** @param {Node} root */\n  const enqueueRoot = (root) => {\n",
    );
    code.shared(r#"    if (destroyed || !isLinkTraversalRoot(root) || queuedRoots.has(root)) return;
    if (pendingRoots.length >= MAX_PENDING_ROOTS) {
      enqueueDocument();
      return;
    }
    pendingRoots.push(root);
    queuedRoots.add(root);
    schedule();
  };

  const enqueueDocument = () => {
    pendingRoots.length = 0;
    queuedRoots.clear();
    activeTraversal = undefined;
    const root = document.documentElement;
    if (root) {
      pendingRoots.push(root);
      queuedRoots.add(root);
      schedule();
    }
  };

  const flush = () => {
    if (destroyed) return;
    let remaining = NODE_BUDGET;
    while (remaining > 0) {
      if (!activeTraversal) {
        const root = pendingRoots.shift();
        if (!root) break;
        queuedRoots.delete(root);
        activeTraversal = createLinkTraversal(root);
        if (!activeTraversal) continue;
      }
      const element = nextTraversalElement(activeTraversal);
      if (!element) {
        activeTraversal = undefined;
        continue;
      }
      if (element.matches("a[href]")) localizeAnchorElement(web, getLocale(), element);
      remaining -= 1;
    }
    if (activeTraversal || pendingRoots.length > 0) schedule();
  };

  const observer = typeof MutationObserver === "undefined"
    ? undefined
    : new MutationObserver((mutations) => {
        let queued = 0;
        for (const mutation of mutations) {
          if (mutation.type === "attributes") {
            enqueueRoot(mutation.target);
            queued += 1;
          } else {
            for (const node of mutation.addedNodes) {
              enqueueRoot(node);
              queued += 1;
              if (queued >= MAX_PENDING_ROOTS) break;
            }
          }
          if (queued >= MAX_PENDING_ROOTS) {
            enqueueDocument();
            return;
          }
        }
      });
  if (document.documentElement) {
    observer?.observe(document.documentElement, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ["href", "download", "rel", "data-linguini-ignore", "data-linguini-no-localize"],
    });
  }
"#);
    code.typed(
        "  const onClick = (event: Event) => {\n    const anchor = (event.target as Element | null)?.closest?.(\"a[href]\");\n",
        "  /** @param {Event} event */\n  const onClick = (event) => {\n    const anchor = (/** @type {Element | null} */ (event.target))?.closest?.(\"a[href]\");\n",
    );
    code.shared(
        r#"    if (anchor) localizeAnchorElement(web, getLocale(), anchor);
  };
  document.addEventListener("click", onClick, true);
  const onReady = () => enqueueDocument();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", onReady, { once: true });
  } else {
    enqueueDocument();
  }

  return {
    refresh: enqueueDocument,
    destroy() {
      destroyed = true;
      observer?.disconnect();
      document.removeEventListener("click", onClick, true);
      document.removeEventListener("DOMContentLoaded", onReady);
      if (animationFrame !== undefined) cancelAnimationFrame(animationFrame);
      if (timer !== undefined) clearTimeout(timer);
      pendingRoots.length = 0;
      queuedRoots.clear();
      activeTraversal = undefined;
    },
  };
}

"#,
    );
    code.typed(
        "interface LinkTraversal {\n  root: Element | undefined;\n  rootPending: boolean;\n  walker: TreeWalker;\n}\n\n",
        "/** @typedef {{root: Element | undefined, rootPending: boolean, walker: TreeWalker}} LinkTraversal */\n\n",
    );
    code.typed(
        "function isLinkTraversalRoot(node: Node) {\n",
        "/** @param {Node} node */\nfunction isLinkTraversalRoot(node) {\n",
    );
    code.shared(
        "  return node.nodeType === 1 || node.nodeType === 9 || node.nodeType === 11;\n}\n\n",
    );
    code.typed(
        "function createLinkTraversal(root: Node): LinkTraversal | undefined {\n  const ownerDocument = root.nodeType === 9 ? root as Document : root.ownerDocument;\n",
        "/**\n * @param {Node} root\n * @returns {LinkTraversal | undefined}\n */\nfunction createLinkTraversal(root) {\n  const ownerDocument = root.nodeType === 9 ? /** @type {Document} */ (root) : root.ownerDocument;\n",
    );
    code.shared("  if (!ownerDocument) return undefined;\n  return {\n");
    code.typed(
        "    root: root.nodeType === 1 ? root as Element : undefined,\n",
        "    root: root.nodeType === 1 ? /** @type {Element} */ (root) : undefined,\n",
    );
    code.shared("    rootPending: root.nodeType === 1,\n    walker: ownerDocument.createTreeWalker(root, 0x1),\n  };\n}\n\n");
    code.typed(
        "function nextTraversalElement(traversal: LinkTraversal): Element | undefined {\n",
        "/**\n * @param {LinkTraversal} traversal\n * @returns {Element | undefined}\n */\nfunction nextTraversalElement(traversal) {\n",
    );
    code.shared("  if (traversal.rootPending) {\n    traversal.rootPending = false;\n    return traversal.root;\n  }\n");
    code.typed(
        "  return (traversal.walker.nextNode() as Element | null) ?? undefined;\n",
        "  return (/** @type {Element | null} */ (traversal.walker.nextNode())) ?? undefined;\n",
    );
    code.shared("}\n\n");
    code.typed(
        "function localizeAnchorElement<LocaleName extends string>(\n  web: LinguiniWebLocale<LocaleName>,\n  locale: LocaleName,\n  anchor: Element,\n) {\n",
        "/**\n * @template {string} LocaleName\n * @param {import(\"../web.js\").LinguiniWebLocale<LocaleName>} web\n * @param {LocaleName} locale\n * @param {Element} anchor\n */\nfunction localizeAnchorElement(web, locale, anchor) {\n",
    );
    code.shared(r#"  const href = anchor.getAttribute("href");
  if (!href) return;
  const input = { currentUrl: window.location.href, origin: window.location.origin };
  const attributes = {
    download: anchor.hasAttribute("download"),
    ignored: anchor.hasAttribute("data-linguini-ignore") || anchor.hasAttribute("data-linguini-no-localize"),
    rel: anchor.getAttribute("rel"),
  };
  if (!web.shouldLocalizeLink(href, attributes, input)) return;
  const localized = web.localizeHref(href, locale, input);
  if (localized !== href) anchor.setAttribute("href", localized);
}
"#);
    module.push_statement(EcmaStatement::generated(code.render(target)));
    module
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn runtime_links_share_target_body_and_snapshots() {
        let typescript = compile_typescript_web_runtime_links_module();
        let javascript = compile_javascript_web_runtime_links_module();
        assert_eq!(
            typescript
                .code
                .strip_suffix("//# sourceMappingURL=runtime-links.ts.map\n")
                .unwrap(),
            generate_typescript_web_runtime_links_module()
        );
        assert!(!javascript.code.contains("import type"));
        assert!(javascript.code.contains("@type {Node[]}"));
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/golden/snapshots/js-web-runtime-links");
        let snapshots = [
            (root.join("web/runtime-links.js"), javascript.code),
            (root.join("web/runtime-links.js.map"), javascript.source_map),
            (
                root.join("typescript/web/runtime-links.ts"),
                typescript.code,
            ),
            (
                root.join("typescript/web/runtime-links.d.ts"),
                generate_web_runtime_links_declaration(),
            ),
        ];
        if std::env::var_os("LINGUINI_UPDATE_SNAPSHOTS").is_some() {
            for (path, contents) in &snapshots {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, contents).unwrap();
            }
        }
        for (path, contents) in snapshots {
            assert_eq!(contents, std::fs::read_to_string(path).unwrap());
        }
    }
}
