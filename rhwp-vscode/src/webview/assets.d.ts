declare module "*.wasm?url" {
  const url: string;
  export default url;
}

declare module "*.sksl?raw" {
  const source: string;
  export default source;
}
