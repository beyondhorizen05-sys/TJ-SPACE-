export type Manifest = {
  id: string; title: string; license: string;
  images?: ImageSpec[]; volumes?: unknown[]; dependencies?: unknown[];
};
export type ImageSpec = { name: string; path: string; architectures?: string[] };

export function setupManifest(input: Manifest): Manifest { return {...input, images: input.images ?? [], volumes: input.volumes ?? [], dependencies: input.dependencies ?? []}; }

export class StartSdk {
  private manifest?: Manifest;
  static of(): StartSdk { return new StartSdk(); }
  withManifest(manifest: Manifest): StartSdk { this.manifest = setupManifest(manifest); return this; }
  build(): Manifest { if (!this.manifest) throw new Error("manifest required"); return this.manifest; }
  Daemons = { of: () => ({ addDaemon: (daemon: unknown) => ({...this, daemon}) }) };
  Action = { withInput: (input: unknown) => ({...this, input}) };
  Backups = { ofVolumes: () => ({withPgDump: (spec?: unknown) => ({withMysqlDump: (mysql?: unknown) => ({addSync: (sync?: unknown) => ({...this, mysql, sync})})})})};
  MultiHost = { of: () => ({bindPort: (port: number) => ({createInterface: (iface: unknown) => ({...this, port, iface})})})};
}
export default StartSdk;
