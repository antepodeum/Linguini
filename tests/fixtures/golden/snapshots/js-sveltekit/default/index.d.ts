export * from "./locale";
export type Linguini = { locale: import("./locale").Locale };
export declare function createLinguini(locale: import("./locale").Locale): Linguini;
export declare function prepareLinguini(locale: import("./locale").Locale): Promise<Linguini>;
