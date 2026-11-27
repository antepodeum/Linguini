import type { __lgl_type_symbol as ScopedSymbol } from "./enum/locales/en.js";
import { root as enumRoot } from "./enum/locales/en.js";
import { root as aliasRoot } from "./alias/locales/en.js";
import type { LinguiniMessages as EnumMessages } from "./enum/messages.js";
import type { LinguiniMessages as AliasMessages } from "./alias/messages.js";
import type { __lgl_type_string as KeywordString, __lgl_name_5F5F6C676C5F747970655F737472696E67 as SourcePrefix, intrinsic as Intrinsic, type as TypeName, abstract as AbstractName, constructor as ConstructorName } from "./enum/shared.js";

const args = {
  string: "entry",
  number: "entry",
  arg_never: "entry",
  arg_number: "entry",
  arg_bigint: "entry",
  arg_boolean: "entry",
  arg_string: "entry",
  arg_symbol: "entry",
  arg_object: "entry",
  arg_undefined: "entry",
  arg_keyof: "entry",
  arg_unique: "entry",
  arg_readonly: "entry",
  arg_infer: "entry",
  arg_as: "entry",
  arg_void: "entry",
  arg_intrinsic: "entry",
  arg_type: "entry",
  arg_abstract: "entry",
  arg_constructor: "entry",
  arg_custom: "other",
} as const;
enumRoot(args);
aliasRoot(args);
enumRoot(args.string, args.number, args.arg_never, args.arg_number, args.arg_bigint, args.arg_boolean, args.arg_string, args.arg_symbol, args.arg_object, args.arg_undefined, args.arg_keyof, args.arg_unique, args.arg_readonly, args.arg_infer, args.arg_as, args.arg_void, args.arg_intrinsic, args.arg_type, args.arg_abstract, args.arg_constructor, args.arg_custom);
aliasRoot({ ...args, string: "free text" });
declare const enumMessages: EnumMessages;
declare const aliasMessages: AliasMessages;
enumMessages.root(args);
aliasMessages.root(args);
const labels: [KeywordString, SourcePrefix, Intrinsic, TypeName, AbstractName, ConstructorName, ScopedSymbol] = ["entry", "other", "entry", "entry", "entry", "entry", "entry"];
void labels;
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, string: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, string: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, number: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, number: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_never: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_never: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_number: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_number: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_bigint: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_bigint: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_boolean: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_boolean: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_string: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_string: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_symbol: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_symbol: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_object: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_object: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_undefined: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_undefined: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_keyof: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_keyof: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_unique: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_unique: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_readonly: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_readonly: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_infer: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_infer: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_as: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_as: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_void: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_void: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_intrinsic: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_intrinsic: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_type: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_type: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_abstract: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_abstract: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_constructor: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_constructor: 1 });
// @ts-expect-error Every enum slot retains its string-literal type.
enumRoot({ ...args, arg_custom: 1 });
// @ts-expect-error Every alias still resolves to source String.
aliasRoot({ ...args, arg_custom: 1 });
// @ts-expect-error A type named string is still a finite enum.
enumRoot({ ...args, arg_string: "wrong" });
// @ts-expect-error The escaped source prefix retains a distinct enum.
enumRoot({ ...args, arg_custom: "entry" });
// @ts-expect-error Compiler naming cannot widen the schema-owned contract.
enumMessages.root({ ...args, string: 1 });
