export interface WasmBuilds {
  readonly webgpu: string;
  readonly webgl2: string;
}
export const WASM_BACKENDS: readonly ["webgpu", "webgl2"];
export const WASM_FILES: readonly string[];
export function wasmContentHash(directory: string): string;
export function readWasmBuilds(projectRoot: string): WasmBuilds;
export function createWasmStage(projectRoot: string): string;
export function removeWasmStage(projectRoot: string, stage: string): void;
export function publishWasmBuilds(
  projectRoot: string,
  stage: string,
): WasmBuilds;
