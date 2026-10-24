export interface EcmascriptProjectLayout {
  readonly projectRoot: string;
  readonly configPath: string;
  readonly configExists: boolean;
  readonly targetConfigured: boolean;
  readonly schemaRoot: string;
  readonly localeRoot: string;
  readonly generatedRoot: string;
  readonly bundlerSourceRoots: readonly string[];
  readonly bundlerExcludeRoots: readonly string[];
  readonly localeLoading: "eager" | "dynamic";
  readonly bundlerEnabled: boolean;
}

export declare function readEcmascriptProject(root: string, configFile?: string): Promise<EcmascriptProjectLayout>;
export declare function readEcmascriptProjectSync(root: string, configFile?: string): EcmascriptProjectLayout;
export declare function validateLocaleLoading(value: unknown, field: string): "eager" | "dynamic";
export declare function isDynamicMessagePath(message: string): boolean;
