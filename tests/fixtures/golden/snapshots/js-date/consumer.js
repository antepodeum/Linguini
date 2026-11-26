import { formatFlexible, formatFixed, formatNarrow, formatYear, formatLocalized } from "./date.js";
import { formatDate_skeleton_yMMMd, formatDate_skeleton_Hm, formatDate_skeleton_GyMMMd, formatDate_skeleton_yQQQ } from "./skeletons.js";

/** @type {string[]} */
const results = [
  formatFlexible(new Date()),
  formatFixed(0),
  formatNarrow("2024-01-02T13:30:00Z"),
  formatYear(new Date(0)),
  formatLocalized("2024-01-02T13:04:05.006Z"),
  formatDate_skeleton_yMMMd("2024-01-02"),
  formatDate_skeleton_Hm(new Date(0)),
  formatDate_skeleton_GyMMMd(0),
  formatDate_skeleton_yQQQ("2024-04-01"),
];
// @ts-expect-error formatter inputs retain their public date contract
formatFlexible(false);
// @ts-expect-error an input date remains required
formatNarrow();
// @ts-expect-error skeleton-selected formatters preserve the same input contract
formatDate_skeleton_yMMMd(false);
// @ts-expect-error quarter skeletons preserve the Date input contract
formatDate_skeleton_yQQQ(false);
// @ts-expect-error localized digit output preserves the Date input contract
formatLocalized(false);
void results;
