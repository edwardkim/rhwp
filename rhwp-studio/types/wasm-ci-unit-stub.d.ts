/**
 * Stage 3 unit-lane boundary.
 *
 * Studio unit CI type-checks application code without paying for a fresh Rust/WASM build.
 * The package lane still runs wasm-pack first and type-checks against pkg/rhwp.d.ts, so
 * generated binding compatibility remains a package-level contract.
 * Only src/core/wasm-bridge.ts and src/hwpctl/index.ts import this module directly; the
 * impact classifier routes both directories to the package lane with real bindings.
 */
declare module '@wasm/rhwp.js' {
  export default function init(input?: unknown): Promise<unknown>;

  export class HwpDocument {
    constructor(data: Uint8Array);
    static createEmpty(): HwpDocument;
    static openWithPassword(data: Uint8Array, password: string): HwpDocument;
    // 인덱스 시그니처는 이름 있는 프로퍼티를 대신하지 못한다 — 구조적 할당 검사에서
    // `exportHwp` 를 요구하는 쪽(document-agent 의 ExportableDocument)에 닿지 않는다.
    // 생성된 `pkg/rhwp.d.ts` 와 같은 시그니처로 명시한다.
    exportHwp(): Uint8Array;
    exportHwpx(): Uint8Array;
    [name: string]: any;
  }

  export function version(): string;
  export const __wasm: Record<string, unknown>;
}
