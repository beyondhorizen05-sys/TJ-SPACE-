export type ManifestInput = {id:string;title:string;license:string;images?:unknown[];volumes?:unknown[];dependencies?:unknown[]};
export function setupManifest(input: ManifestInput) { return {...input, images:input.images??[], volumes:input.volumes??[], dependencies:input.dependencies??[]}; }
