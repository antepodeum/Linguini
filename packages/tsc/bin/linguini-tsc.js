#!/usr/bin/env node
import { runTypeScript } from "../src/index.js";

try {
  const result = runTypeScript(process.argv.slice(2), {
    command: process.env.LINGUINI_COMMAND ?? "linguini",
  });
  if (!result.close) process.exitCode = result.status;
} catch (error) {
  console.error(`[linguini-tsc] ${error.message}`);
  process.exitCode = 1;
}
