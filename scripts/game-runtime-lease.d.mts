import type { Plugin } from 'vite';
export const GAME_LEASE_PORT: number;
export function acquireGameLease(port?: number): Promise<{ port: number; close(): Promise<void> }>;
export function singleGameRuntime(port?: number): Plugin;
