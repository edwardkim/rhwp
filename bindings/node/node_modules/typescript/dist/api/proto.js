import { documentURIToFileName, fileNameToDocumentURI, } from "./path.js";
/**
 * Resolves a DocumentIdentifier to a file name.
 * If the identifier contains a URI, it is converted to a file name.
 */
export function resolveFileName(identifier) {
    if (typeof identifier === "string") {
        return identifier;
    }
    return documentURIToFileName(identifier.uri);
}
/**
 * Resolves a DocumentIdentifier to a document URI.
 * If the identifier contains a file name, it is converted to a URI.
 */
export function resolveDocumentURI(identifier) {
    if (typeof identifier === "string") {
        return fileNameToDocumentURI(identifier);
    }
    return identifier.uri;
}
/**
 * Builds the wire request for updateSnapshot, applying the deprecated `openProject`
 * compatibility shim: a single `openProject` is folded into `openProjects` and is
 * never sent on the wire.
 */
export function toUpdateSnapshotRequest(params) {
    const { openProject, openProjects, ...rest } = params ?? {};
    const mergedOpenProjects = openProject !== undefined
        ? [resolveFileName(openProject), ...(openProjects ?? [])]
        : openProjects;
    return {
        ...rest,
        ...(mergedOpenProjects !== undefined ? { openProjects: mergedOpenProjects } : {}),
    };
}
//# sourceMappingURL=proto.js.map