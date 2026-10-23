export declare const DEFAULT_CONFIG_FILE = "linguini.toml";
export declare const DEFAULT_PATHS: Readonly<{ schema: "linguini/schema"; locale: "linguini/locale" }>;

export interface ProjectConfigDocument {
  readonly projectRoot: string;
  readonly configPath: string;
  readonly configExists: boolean;
  readonly document: Readonly<Record<string, unknown>>;
}

export declare function resolveProjectPath(root: string, value: string, field: string): string;
export declare function readProjectConfig(root: string, configFile?: string): Promise<ProjectConfigDocument>;
export declare function readProjectConfigSync(root: string, configFile?: string): ProjectConfigDocument;
