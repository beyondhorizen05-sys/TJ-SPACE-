export type ActionInput = {name:string; input?:unknown};
export const Action = {withInput:(input:unknown) => ({input})};
