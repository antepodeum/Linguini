import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import {validatePluralRuntimes} from "../src/plural-payloads.js";

test("plural metadata validates complete demand and reachable unions before bundling", () => {
  const raw = {ru:{
    module:"runtime.js",binding:"pluralRu",categories:["few","many","one"],
    messages:{one:["one"],few:["few"],category:null,literal:[]},
    variants:[
      {categories:[],module:"fallback.js"},
      {categories:["one"],module:"one.js"},
      {categories:["few"],module:"few.js"},
      {categories:["few","one"],module:"few-one.js"},
    ],
  }};
  const options = {
    messages:new Map(Object.keys(raw.ru.messages).map(name => [name,{locales:new Map([["ru",{}]])}])),
    runtimes:new Map([["ru",{module:path.resolve("runtime.js")}]]),
    resolvePath:value => path.resolve(value),
    context:"fixture",
  };
  const valid = validatePluralRuntimes(raw,options).get("ru");
  assert.equal(valid.variants.size,4);
  assert.equal(valid.messages.get("category"),null);
  assert.equal(validatePluralRuntimes(undefined,options).size,0,"older prebuilt manifests remain usable");
  const mutations = [
    value => {value.ru.module="another.js";},
    value => {value.ru.binding="bad-binding";},
    value => {value.ru.categories=["one","one"];},
    value => {value.ru.categories=["one","few"];},
    value => {value.ru.categories=["other"];},
    value => {delete value.ru.messages.one;},
    value => {value.ru.messages.extra=[];},
    value => {value.ru.messages.one=["two"];},
    value => {value.ru.messages.one="one";},
    value => {value.ru.variants.pop();},
    value => {value.ru.variants[1].module="runtime.js";},
    value => {value.ru.variants[1].module="fallback.js";},
    value => {value.ru.variants[1].module="one.ts";},
    value => {value.ru.variants[1].categories=[];},
    value => {value.ru.variants[1].categories=["other"];},
    value => {value.unknown=value.ru;delete value.ru;},
  ];
  for (const mutate of mutations) {
    const invalid = structuredClone(raw);
    mutate(invalid);
    assert.throws(() => validatePluralRuntimes(invalid,options),/plural_runtimes/);
  }
  assert.deepEqual(valid.messages.get("one"),["one"],"validation does not mutate demands");
});
