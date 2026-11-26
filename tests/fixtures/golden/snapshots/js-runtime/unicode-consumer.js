import {formatNumber, formatCurrency} from './unicode.js';

/** @type {string[]} */
const results = [formatNumber(123), formatNumber(123n), formatCurrency('123.45')];
// @ts-expect-error Unicode digit substitution retains numeric input types
formatNumber(false);
// @ts-expect-error Unicode currency formatting retains numeric input types
formatCurrency(new Date());
void results;
