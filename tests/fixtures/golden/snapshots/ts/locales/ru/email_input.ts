import type { Fruit, Size, Money, ShortDate } from "../../shared";

import { formatCurrency, formatDate } from "./_runtime";
import { cart_label, __lgl_form_4672756974, Delivered, SizeAdj, DeliveryNote } from "./_globals";
export type { Fruit, Size, Money, ShortDate } from "../../shared";

export const email_input = {
  label: "Email",
  placeholder: "name@example.com",
  aria: "Адрес электронной почты",
} as const;

const lgl = {
  email_input,
} as const;

export default lgl;
