import { formatSyntaxKind } from "../ast/utils.js";
const hasOwnProperty = Object.prototype.hasOwnProperty;
/**
 * Indicates whether a map-like contains an own property with the specified key.
 *
 * @param map A map-like.
 * @param key A property key.
 */
export function hasProperty(map, key) {
    return hasOwnProperty.call(map, key);
}
export function assertNever(member, message = "Illegal value:", stackCrawlMark) {
    const detail = typeof member === "object" && hasProperty(member, "kind") && hasProperty(member, "pos") ? "SyntaxKind: " + formatSyntaxKind(member.kind) : JSON.stringify(member);
    return fail(`${message} ${detail}`, stackCrawlMark || assertNever);
}
export function fail(message, stackCrawlMark) {
    // eslint-disable-next-line no-debugger
    debugger;
    const e = new Error(message ? `Debug Failure. ${message}` : "Debug Failure.");
    if (Error.captureStackTrace) {
        Error.captureStackTrace(e, stackCrawlMark || fail);
    }
    throw e;
}
//# sourceMappingURL=utils.js.map