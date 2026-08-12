import { CharacterCodes } from "#enums/characterCodes";
import { SyntaxKind } from "#enums/syntaxKind";
let syntaxKindNames;
function getSyntaxKindNames() {
    if (!syntaxKindNames) {
        syntaxKindNames = new Map();
        for (const name of Object.keys(SyntaxKind)) {
            const val = SyntaxKind[name];
            if (typeof val === "number" && !syntaxKindNames.has(val)) {
                syntaxKindNames.set(val, name);
            }
        }
        syntaxKindNames.set(SyntaxKind.EndOfFile, "EndOfFileToken");
    }
    return syntaxKindNames;
}
export function formatSyntaxKind(kind) {
    return getSyntaxKindNames().get(kind) ?? `Unknown(${kind})`;
}
/**
 * Remove one extra leading underscore from an identifier name, recovering the
 * display form from its escaped {@link __String} key.
 */
export function unescapeLeadingUnderscores(identifier) {
    const id = identifier;
    return id.length >= 3 && id.charCodeAt(0) === CharacterCodes._ && id.charCodeAt(1) === CharacterCodes._ && id.charCodeAt(2) === CharacterCodes._
        ? id.slice(1)
        : id;
}
/**
 * Add an extra leading underscore to a display name that already begins with
 * `__`, producing its escaped {@link __String} key.
 */
export function escapeLeadingUnderscores(identifier) {
    return (identifier.length >= 2 && identifier.charCodeAt(0) === CharacterCodes._ && identifier.charCodeAt(1) === CharacterCodes._
        ? "_" + identifier
        : identifier);
}
export function tryCast(value, test) {
    return value !== undefined && test(value) ? value : undefined;
}
export function cast(value, test) {
    if (value !== undefined && test(value))
        return value;
    throw new Error(`Invalid cast. The supplied value ${value} did not pass the test '${test.name}'.`);
}
export function cloneSourceFileData(sourceFile) {
    return {
        statements: sourceFile.statements,
        endOfFileToken: sourceFile.endOfFileToken,
        text: sourceFile.text,
        fileName: sourceFile.fileName,
        path: sourceFile.path,
        languageVariant: sourceFile.languageVariant,
        scriptKind: sourceFile.scriptKind,
        isDeclarationFile: sourceFile.isDeclarationFile,
        referencedFiles: sourceFile.referencedFiles,
        typeReferenceDirectives: sourceFile.typeReferenceDirectives,
        libReferenceDirectives: sourceFile.libReferenceDirectives,
        imports: sourceFile.imports,
        moduleAugmentations: sourceFile.moduleAugmentations,
        ambientModuleNames: sourceFile.ambientModuleNames,
        externalModuleIndicator: sourceFile.externalModuleIndicator,
        tokenCache: undefined,
    };
}
//# sourceMappingURL=utils.js.map