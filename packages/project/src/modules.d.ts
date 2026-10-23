export type ModuleData = null | boolean | number | string | readonly ModuleData[] | { readonly [key: string]: ModuleData };
export interface ModuleDefinition<Id extends string = string, Options extends object = object> {
  readonly id: Id;
  readonly options: Readonly<Options>;
}
export interface ModulePlan {
  readonly version: 1;
  readonly modules: readonly ModuleDefinition[];
}
export declare const MODULE_CONFIG_FILE: "linguini.config.mjs";
export declare const MODULE_PLAN_FILE: "linguini.modules.json";
export declare function defineModule<Id extends string, Options extends object>(id: Id, options?: Options): ModuleDefinition<Id, Options>;
export declare function defineModules(modules: readonly ModuleDefinition[]): ModulePlan;
export declare function normalizeModulePlan(value: unknown): ModulePlan;
export interface ModuleConfiguration {
  readonly plan: ModulePlan;
  readonly configPath?: string;
  readonly dependencies: readonly string[];
}
export declare function readModuleConfiguration(root: string): Promise<ModuleConfiguration>;
export declare function readModuleConfigurationSync(root: string): ModuleConfiguration;
export declare function moduleConfigPaths(root: string): string[];
