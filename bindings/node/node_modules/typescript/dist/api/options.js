/**
 * Shared utilities for the TypeScript API client.
 */
import getExePath from "#getExePath";
export function isSpawnOptions(options) {
    return !("pipe" in options);
}
export function resolveExePath(options) {
    return options.tsserverPath ?? getExePath();
}
//# sourceMappingURL=options.js.map