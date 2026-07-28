import assert from "node:assert/strict";
import test from "node:test";
import { platformPackage } from "../lib/platform.js";

test("maps every published native package", () => {
  assert.equal(platformPackage("linux", "x64"), "@linguini/cli-linux-x64");
  assert.equal(platformPackage("linux", "arm64"), "@linguini/cli-linux-arm64");
  assert.equal(platformPackage("darwin", "x64"), "@linguini/cli-darwin-x64");
  assert.equal(platformPackage("darwin", "arm64"), "@linguini/cli-darwin-arm64");
  assert.equal(platformPackage("win32", "x64"), "@linguini/cli-win32-x64");
  assert.equal(platformPackage("win32", "arm64"), "@linguini/cli-win32-arm64");
});

test("rejects unsupported native targets", () => {
  assert.throws(() => platformPackage("freebsd", "x64"), /no native package/);
});
