import { message as physical } from "./bundler/messages/x726f6f74/x656e.js";
import { message as standalone } from "./standalone.js";

const date = new Date("2026-01-02T00:00:00Z");
physical(date, date);
physical({ value: date, direct: date });
standalone(date, date);
standalone({ value: date, direct: date });
// @ts-expect-error Physical message inputs retain their date domain.
physical(true, date);
// @ts-expect-error Standalone message inputs retain their date domain.
standalone(date, 1n);
