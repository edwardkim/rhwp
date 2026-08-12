/// <reference path="../node/node.d.ts" preserve="true" />
import { CompletionItemKind } from "#enums/completionItemKind";
import { DiagnosticCategory } from "#enums/diagnosticCategory";
import { ElementFlags } from "#enums/elementFlags";
import { ModuleKind } from "#enums/moduleKind";
import { NodeBuilderFlags } from "#enums/nodeBuilderFlags";
import { ObjectFlags } from "#enums/objectFlags";
import { SignatureFlags } from "#enums/signatureFlags";
import { SignatureKind } from "#enums/signatureKind";
import { SymbolFlags } from "#enums/symbolFlags";
import { TypeFlags } from "#enums/typeFlags";
import { TypePredicateKind } from "#enums/typePredicateKind";
import { type __String, type Expression, type Identifier, ModifierFlags, type Node, type Path, type SourceFile, type SyntaxKind, type TypeNode } from "../../ast/index.ts";
import type { APIOptions, LSPConnectionOptions } from "../options.ts";
import type { CompilerOptions, ConfigResponse, DocumentIdentifier, DocumentPosition, LSPUpdateSnapshotParams, ProjectResponse, SignatureResponse, SourceFileMetadata, SymbolResponse, TypeResponse, UpdateSnapshotParams, UpdateSnapshotResponse } from "../proto.ts";
import { SourceFileCache } from "../sourceFileCache.ts";
import type { RequestTiming, TimingAccumulators, TimingInfo } from "../timing.ts";
import { Client, type ClientSocketOptions, type ClientSpawnOptions } from "./client.ts";
import type { AssertsIdentifierTypePredicate, AssertsThisTypePredicate, BigIntLiteralType, BooleanLiteralType, CompletionEntry, CompletionInfo, CompletionOptions, ConditionalType, Diagnostic, FreshableType, IdentifierTypePredicate, IndexedAccessType, IndexInfo, IndexType, InterfaceType, IntersectionType, IntrinsicType, JSDocTagInfo, LiteralType, NumberLiteralType, ObjectType, StringLiteralType, StringMappingType, SubstitutionType, TemplateLiteralType, ThisTypePredicate, TupleType, Type, TypeParameter, TypePredicate, TypePredicateBase, TypeReference, UnionOrIntersectionType, UnionType } from "./types.ts";
export { documentURIToFileName, fileNameToDocumentURI } from "../path.ts";
export { CompletionItemKind, DiagnosticCategory, ElementFlags, ModifierFlags, ModuleKind, NodeBuilderFlags, ObjectFlags, SignatureFlags, SignatureKind, SymbolFlags, TypeFlags, TypePredicateKind };
export type { APIOptions, AssertsIdentifierTypePredicate, AssertsThisTypePredicate, BigIntLiteralType, BooleanLiteralType, ClientSocketOptions, ClientSpawnOptions, CompilerOptions, CompletionEntry, CompletionInfo, CompletionOptions, ConditionalType, Diagnostic, DocumentIdentifier, DocumentPosition, FreshableType, IdentifierTypePredicate, IndexedAccessType, IndexInfo, IndexType, InterfaceType, IntersectionType, IntrinsicType, JSDocTagInfo, LiteralType, LSPConnectionOptions, NumberLiteralType, ObjectType, RequestTiming, SourceFileMetadata, StringLiteralType, StringMappingType, SubstitutionType, TemplateLiteralType, ThisTypePredicate, TimingAccumulators, TimingInfo, TupleType, Type, TypeParameter, TypePredicate, TypePredicateBase, TypeReference, UnionOrIntersectionType, UnionType };
export declare class API<FromLSP extends boolean = false> {
    private client;
    private sourceFileCache;
    private toPath;
    private initialized;
    private activeSnapshots;
    private latestSnapshot;
    readonly internal: InternalAPI;
    constructor(options?: APIOptions | LSPConnectionOptions);
    /**
     * Create an API instance from an existing LSP connection's API session.
     * Use this when connecting to an API pipe provided by an LSP server via custom/initializeAPISession.
     */
    static fromLSPConnection(options: LSPConnectionOptions): Promise<API<true>>;
    private ensureInitialized;
    parseConfigFile(file: DocumentIdentifier): Promise<ConfigResponse>;
    updateSnapshot(params?: FromLSP extends true ? LSPUpdateSnapshotParams : UpdateSnapshotParams): Promise<Snapshot>;
    close(): Promise<void>;
    clearSourceFileCache(): void;
    /**
     * Returns a snapshot of collected timing information for requests made
     * through this API instance: client-measured round-trip latency and bytes
     * transferred, folded together with the server's own per-request processing
     * time and an estimated transport overhead (round-trip minus server time).
     *
     * Fetching the snapshot issues a lightweight request to the server to
     * retrieve its timing collection. Collection must be enabled via the
     * `collectTiming` option; when it is not, the returned snapshot has
     * `enabled: false` and zeroed totals.
     */
    getTimingInfo(): Promise<TimingInfo>;
    /** Clears all accumulated timing totals and recent-request history, on both the client and the server. */
    resetTimingInfo(): Promise<void>;
}
export declare class InternalAPI {
    private client;
    private ensureInitialized;
    /** @internal */
    constructor(client: Client, ensureInitialized: () => Promise<void>);
    startCPUProfile(dir: string): Promise<void>;
    stopCPUProfile(): Promise<string>;
    saveHeapProfile(dir: string): Promise<string>;
}
export declare class Snapshot {
    readonly id: number;
    private projectMap;
    private toPath;
    private client;
    private disposed;
    private onDispose;
    private snapshotRegistry;
    constructor(data: UpdateSnapshotResponse, client: Client, sourceFileCache: SourceFileCache, toPath: (fileName: string) => Path, onDispose: () => void);
    getProjects(): readonly Project[];
    getProject(configFileName: string): Project | undefined;
    getDefaultProjectForFile(file: DocumentIdentifier): Promise<Project | undefined>;
    [globalThis.Symbol.dispose](): void;
    dispose(): Promise<void>;
    isDisposed(): boolean;
    private ensureNotDisposed;
}
declare class SnapshotObjectRegistry {
    private readonly symbols;
    private readonly client;
    private readonly snapshotId;
    private readonly resolveProject;
    constructor(client: Client, snapshotId: number, resolveProject: (projectId: Path) => Project | undefined);
    /** Resolve a project id (a config file path) to its Project within this snapshot. */
    getProject(projectId: Path): Project | undefined;
    getOrCreateSymbol(data: SymbolResponse): Symbol;
    getSymbol(id: number): Symbol | undefined;
    clear(): void;
    fetchSymbol(source: Symbol | Signature | Type, method: string, handle: number | undefined, projectId?: Path): Promise<Symbol>;
    fetchSymbols(source: Symbol | Signature | Type, method: string, handles?: readonly number[], projectId?: Path): Promise<readonly Symbol[]>;
}
declare class ProjectObjectRegistry {
    private client;
    private snapshotId;
    private project;
    private snapshotRegistry;
    private types;
    private signatures;
    constructor(client: Client, snapshotId: number, project: Project, snapshotRegistry: SnapshotObjectRegistry);
    getOrCreateSymbol(data: SymbolResponse): Symbol;
    getSymbol(id: number): Symbol | undefined;
    getOrCreateType(data: TypeResponse): TypeObject;
    getType(id: number): TypeObject | undefined;
    getOrCreateSignature(data: SignatureResponse): Signature;
    getSignature(id: number): Signature | undefined;
    clear(): void;
    fetchType<T extends Type>(source: Symbol | Signature | Type, method: string, handle: number | false | undefined): Promise<T>;
    fetchSymbol(source: Symbol | Signature | Type, method: string, handle: number | undefined): Promise<Symbol>;
    fetchSignature(source: Symbol | Signature | Type, method: string, handle: number | undefined): Promise<Signature>;
    fetchTypes(source: Symbol | Signature | Type, method: string, handles?: readonly number[]): Promise<readonly Type[]>;
    fetchSymbols(source: Symbol | Signature | Type, method: string, handles?: readonly number[]): Promise<readonly Symbol[]>;
    fetchBaseTypes(source: Type): Promise<readonly Type[]>;
}
export declare class Project {
    readonly id: Path;
    readonly configFileName: string;
    readonly compilerOptions: CompilerOptions;
    readonly rootFiles: readonly string[];
    readonly program: Program;
    readonly checker: Checker;
    readonly emitter: Emitter;
    private client;
    constructor(data: ProjectResponse, snapshotId: number, client: Client, sourceFileCache: SourceFileCache, toPath: (fileName: string) => Path, snapshotRegistry: SnapshotObjectRegistry);
    dispose(): void;
}
export declare class Program {
    private snapshotId;
    private project;
    private client;
    private sourceFileCache;
    private toPath;
    private decoder;
    private sourceFileMetadataCache;
    constructor(snapshotId: number, project: Project, client: Client, sourceFileCache: SourceFileCache, toPath: (fileName: string) => Path);
    getCompilerOptions(): CompilerOptions;
    getSourceFile(file: DocumentIdentifier): Promise<SourceFile | undefined>;
    getSourceFileNames(): Promise<readonly string[]>;
    /**
     * Returns program-stored metadata for the given source file, or `undefined` if the file
     * is not part of the program. Metadata is fetched lazily per file and cached on this
     * `Program` instance.
     */
    getSourceFileMetadata(fileName: string): Promise<SourceFileMetadata | undefined>;
    /**
     * Returns program-stored metadata for the source file at the given path, or `undefined`
     * if the file is not part of the program. Like {@link getSourceFileMetadata}, but skips
     * the file name to path conversion. Metadata is fetched lazily per file and cached on
     * this `Program` instance.
     */
    getSourceFileMetadataByPath(path: Path): Promise<SourceFileMetadata | undefined>;
    private fetchSourceFileMetadata;
    /**
     * Returns whether the given source file was loaded as part of an external library
     * (e.g. a dependency resolved from `node_modules`). The underlying program metadata is
     * fetched lazily per file and cached on this `Program` instance.
     */
    isSourceFileFromExternalLibrary(file: SourceFile): Promise<boolean>;
    /**
     * Returns whether the given source file is a default library file (e.g. `lib.d.ts`).
     * The underlying program metadata is fetched lazily per file and cached on this
     * `Program` instance.
     */
    isSourceFileDefaultLibrary(file: SourceFile): Promise<boolean>;
    /**
     * Get syntactic (parse) diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getSyntacticDiagnostics(file?: DocumentIdentifier): Promise<readonly Diagnostic[]>;
    /**
     * Get binder diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getBindDiagnostics(file?: DocumentIdentifier): Promise<readonly Diagnostic[]>;
    /**
     * Get semantic (type-check) diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getSemanticDiagnostics(file?: DocumentIdentifier): Promise<readonly Diagnostic[]>;
    /**
     * Get suggestion diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getSuggestionDiagnostics(file?: DocumentIdentifier): Promise<readonly Diagnostic[]>;
    /**
     * Get declaration emit diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getDeclarationDiagnostics(file?: DocumentIdentifier): Promise<readonly Diagnostic[]>;
    /**
     * Get program-wide diagnostics for the project, including compiler options diagnostics.
     */
    getProgramDiagnostics(): Promise<readonly Diagnostic[]>;
    /**
     * Get global (non-file-specific) semantic diagnostics for the project.
     */
    getGlobalDiagnostics(): Promise<readonly Diagnostic[]>;
    /**
     * Get config file parsing diagnostics for the project.
     */
    getConfigFileParsingDiagnostics(): Promise<readonly Diagnostic[]>;
}
export declare class Checker {
    private snapshotId;
    private project;
    private client;
    private objectRegistry;
    private wellKnownSymbols;
    constructor(snapshotId: number, project: Project, client: Client, objectRegistry: ProjectObjectRegistry);
    dispose(): void;
    getSymbolAtLocation(node: Node): Promise<Symbol | undefined>;
    getSymbolAtLocation(nodes: readonly Node[]): Promise<(Symbol | undefined)[]>;
    getSymbolAtPosition(file: DocumentIdentifier, position: number): Promise<Symbol | undefined>;
    getSymbolAtPosition(file: DocumentIdentifier, positions: readonly number[]): Promise<(Symbol | undefined)[]>;
    getTypeOfSymbol(symbol: Symbol): Promise<Type | undefined>;
    getTypeOfSymbol(symbols: readonly Symbol[]): Promise<(Type | undefined)[]>;
    /**
     * Get the declared type of a symbol. Always returns a type; for symbols whose
     * declared type cannot be determined the checker yields the error type (use
     * {@link Type.isErrorType} to detect it).
     */
    getDeclaredTypeOfSymbol(symbol: Symbol): Promise<Type>;
    getReferencesToSymbolInFile(file: DocumentIdentifier, symbol: Symbol): Promise<NodeHandle[]>;
    getReferencedSymbolsForNode(node: Node, position: number): Promise<ReferencedSymbolEntry[]>;
    getSignatureUsage(signatureDecl: Node): Promise<SignatureUsage[]>;
    getCompletionsAtPosition(document: string, position: number, options?: CompletionOptions): Promise<CompletionInfo | undefined>;
    getTypeAtLocation(node: Node): Promise<Type | undefined>;
    getTypeAtLocation(nodes: readonly Node[]): Promise<(Type | undefined)[]>;
    getSignaturesOfType(type: Type, kind: SignatureKind): Promise<readonly Signature[]>;
    getResolvedSignature(node: Node): Promise<Signature | undefined>;
    getTypeAtPosition(file: DocumentIdentifier, position: number): Promise<Type | undefined>;
    getTypeAtPosition(file: DocumentIdentifier, positions: readonly number[]): Promise<(Type | undefined)[]>;
    resolveName(name: string, meaning: SymbolFlags, location?: Node | DocumentPosition, excludeGlobals?: boolean): Promise<Symbol | undefined>;
    getResolvedSymbol(node: Identifier): Promise<Symbol | undefined>;
    getContextualType(node: Expression): Promise<Type | undefined>;
    getBaseTypeOfLiteralType(type: Type): Promise<Type | undefined>;
    getNonNullableType(type: Type): Promise<Type | undefined>;
    getTypeFromTypeNode(node: TypeNode): Promise<Type | undefined>;
    getWidenedType(type: Type): Promise<Type | undefined>;
    getParameterType(signature: Signature, index: number): Promise<Type | undefined>;
    isArrayLikeType(type: Type): Promise<boolean>;
    isTypeAssignableTo(source: Type, target: Type): Promise<boolean>;
    getShorthandAssignmentValueSymbol(node: Node): Promise<Symbol | undefined>;
    /**
     * Get the type of a symbol as narrowed at a specific location. Always returns
     * a type; for symbols whose type cannot be determined the checker yields the
     * error type (use {@link Type.isErrorType} to detect it).
     */
    getTypeOfSymbolAtLocation(symbol: Symbol, location: Node): Promise<Type>;
    private getIntrinsicType;
    getAnyType(): Promise<Type>;
    getStringType(): Promise<Type>;
    getNumberType(): Promise<Type>;
    getBooleanType(): Promise<Type>;
    getVoidType(): Promise<Type>;
    getUndefinedType(): Promise<Type>;
    getNullType(): Promise<Type>;
    getNeverType(): Promise<Type>;
    getUnknownType(): Promise<Type>;
    getBigIntType(): Promise<Type>;
    getESSymbolType(): Promise<Type>;
    typeToTypeNode(type: Type, enclosingDeclaration?: Node, flags?: number): Promise<TypeNode | undefined>;
    signatureToSignatureDeclaration(signature: Signature, kind: SyntaxKind, enclosingDeclaration?: Node, flags?: NodeBuilderFlags): Promise<Node | undefined>;
    typeToString(type: Type, enclosingDeclaration?: Node, flags?: number): Promise<string>;
    isContextSensitive(node: Node): Promise<boolean>;
    isArrayType(type: Type): Promise<boolean>;
    isTupleType(type: Type): Promise<boolean>;
    getReturnTypeOfSignature(signature: Signature): Promise<Type | undefined>;
    getRestTypeOfSignature(signature: Signature): Promise<Type | undefined>;
    getTypePredicateOfSignature(signature: Signature): Promise<TypePredicate | undefined>;
    /**
     * Get the base types of a class or interface type. A type with no base types
     * yields an empty array.
     */
    getBaseTypes(type: InterfaceType): Promise<readonly Type[]>;
    getApparentType(type: Type): Promise<Type | undefined>;
    getPropertiesOfType(type: Type): Promise<readonly Symbol[]>;
    getIndexInfosOfType(type: Type): Promise<readonly IndexInfo[]>;
    /**
     * Get the constraint of a type parameter (the `T` in `<U extends T>`), or
     * undefined if it has none.
     */
    getConstraintOfTypeParameter(type: TypeParameter): Promise<Type | undefined>;
    getBaseConstraintOfType(type: Type): Promise<Type | undefined>;
    getPropertyOfType(type: Type, name: string): Promise<Symbol | undefined>;
    getConstantValue(node: Node): Promise<string | number | undefined>;
    getSignatureFromDeclaration(node: Node): Promise<Signature | undefined>;
    getExportSpecifierLocalTargetSymbol(node: Node): Promise<Symbol | undefined>;
    /**
     * Follow all aliases to get the original symbol. Always returns a symbol; for
     * an unresolved alias the checker yields the unknown symbol (use
     * {@link Checker.isUnknownSymbol} to detect it).
     */
    getAliasedSymbol(symbol: Symbol): Promise<Symbol>;
    getImmediateAliasedSymbol(symbol: Symbol): Promise<Symbol | undefined>;
    /**
     * Fetch (once, then cache) the handle ids of the per-checker singleton
     * symbols (unknown, undefined, arguments). These ids are stable for the life
     * of the project's checker, so identity checks against them are local after
     * the first call.
     */
    private getWellKnownSymbols;
    /**
     * Returns `true` if the symbol is the checker's "unknown" symbol (e.g. the
     * result of {@link Checker.getAliasedSymbol} on an unresolved alias).
     */
    isUnknownSymbol(symbol: Symbol): Promise<boolean>;
    /**
     * Returns `true` if the symbol is the checker's "undefined" symbol.
     */
    isUndefinedSymbol(symbol: Symbol): Promise<boolean>;
    /**
     * Returns `true` if the symbol is the checker's "arguments" symbol.
     */
    isArgumentsSymbol(symbol: Symbol): Promise<boolean>;
    getExportsOfModule(symbol: Symbol): Promise<readonly Symbol[]>;
    getMemberInModuleExports(symbol: Symbol, name: string): Promise<Symbol | undefined>;
    getJsDocTagsOfSymbol(symbol: Symbol): Promise<readonly JSDocTagInfo[]>;
    getDocumentationCommentOfSymbol(symbol: Symbol): Promise<string>;
    /**
     * Get the type arguments of a type reference (e.g. the `string` in `Array<string>`).
     */
    getTypeArguments(type: TypeReference): Promise<readonly Type[]>;
}
export interface PrintNodeOptions {
    preserveSourceNewlines?: boolean | undefined;
    neverAsciiEscape?: boolean | undefined;
    terminateUnterminatedLiterals?: boolean | undefined;
}
export declare class Emitter {
    private client;
    constructor(client: Client);
    printNode(node: Node, options?: PrintNodeOptions): Promise<string>;
}
export declare class NodeHandle {
    /**
     * The project this handle was produced in, used as the default for {@link resolve}.
     * Node handles are only meaningful within a project's program, so the producing project
     * is remembered so callers don't have to pass it explicitly.
     */
    private readonly canonicalProject;
    readonly index: number;
    readonly kind: SyntaxKind;
    readonly path: Path;
    constructor(handle: string, canonicalProject: Project);
    /**
     * Resolve this handle to the actual AST node by fetching the source file from a project
     * and looking up the node by index. If no project is passed, the project that produced
     * the handle is used.
     */
    resolve(project?: Project): Promise<Node | undefined>;
}
/** A symbol definition paired with all of its reference nodes. */
export interface ReferencedSymbolEntry {
    /** The node handle for the symbol's definition. */
    definition: NodeHandle;
    /** The resolved symbol for the definition, if available. */
    symbol?: Symbol | undefined;
    /** The node handles for each reference to the symbol. */
    references: NodeHandle[];
}
/** A single usage of a signature, pairing the reference name with its call expression (if any). */
export interface SignatureUsage {
    /** The node handle for the name reference. */
    name: NodeHandle;
    /** The node handle for the call expression, if the reference is invoked. */
    call?: NodeHandle | undefined;
}
export declare class Symbol {
    private objectRegistry;
    /**
     * The project this symbol was first observed in, used as the default project for
     * lookups that need a project context (members/exports/parent). Symbols are shared
     * snapshot-wide, so these lookups can otherwise be ambiguous about which project to use.
     */
    private readonly canonicalProject;
    readonly id: number;
    /** The escaped (`__String`) name, used as the key in member/export tables. */
    readonly escapedName: __String;
    /** The display name (escaped underscores removed). */
    readonly name: string;
    readonly flags: SymbolFlags;
    readonly checkFlags: number;
    readonly declarations: readonly NodeHandle[];
    readonly valueDeclaration: NodeHandle | undefined;
    private readonly parent;
    private readonly exportSymbol;
    private membersCache;
    private exportsCache;
    constructor(data: SymbolResponse, objectRegistry: SnapshotObjectRegistry);
    getParent(): Promise<Symbol | undefined>;
    /**
     * Get this symbol's members keyed by escaped name. The result is cached on
     * the symbol, so repeated calls do not round-trip to the server.
     */
    getMembers(): Promise<ReadonlyMap<__String, Symbol>>;
    /**
     * Get this symbol's exports keyed by escaped name. The result is cached on
     * the symbol, so repeated calls do not round-trip to the server.
     */
    getExports(): Promise<ReadonlyMap<__String, Symbol>>;
    private fetchSymbolTable;
    getExportSymbol(): Promise<Symbol>;
    getJsDocTags(checker: Checker): Promise<readonly JSDocTagInfo[]>;
    getDocumentationComment(checker: Checker): Promise<string>;
}
declare class TypeObject implements Type {
    private objectRegistry;
    readonly id: number;
    readonly flags: TypeFlags;
    readonly objectFlags: ObjectFlags;
    readonly symbol: number;
    readonly value: string | number | boolean | bigint;
    readonly intrinsicName: string;
    readonly isThisType: boolean;
    readonly freshType: number;
    readonly regularType: number;
    readonly target: number;
    readonly typeParameters: readonly number[];
    readonly outerTypeParameters: readonly number[];
    readonly localTypeParameters: readonly number[];
    readonly aliasTypeArguments: readonly number[];
    readonly aliasSymbol: number;
    readonly elementFlags: readonly ElementFlags[];
    readonly fixedLength: number;
    readonly readonly: boolean;
    readonly texts: readonly string[];
    readonly objectType: number;
    readonly indexType: number;
    readonly checkType: number;
    readonly extendsType: number;
    readonly baseType: number;
    readonly substConstraint: number;
    private trueType;
    private falseType;
    constructor(data: TypeResponse, objectRegistry: ProjectObjectRegistry);
    getSymbol(): Promise<Symbol | undefined>;
    getAliasSymbol(): Promise<Symbol | undefined>;
    getTarget(): Promise<Type>;
    getFreshType(): Promise<FreshableType | undefined>;
    getRegularType(): Promise<FreshableType | undefined>;
    getTypes(): Promise<readonly Type[] | undefined>;
    getTypeParameters(): Promise<readonly TypeParameter[]>;
    getOuterTypeParameters(): Promise<readonly TypeParameter[]>;
    getLocalTypeParameters(): Promise<readonly TypeParameter[]>;
    getAliasTypeArguments(): Promise<readonly Type[]>;
    getObjectType(): Promise<Type>;
    getIndexType(): Promise<Type>;
    getCheckType(): Promise<Type>;
    getExtendsType(): Promise<Type>;
    getBaseType(): Promise<Type>;
    getConstraint(): Promise<Type>;
    getTrueType(): Promise<Type>;
    getFalseType(): Promise<Type>;
    /**
     * Get the base types of this type. Returns `undefined` for any type that is
     * not a class or interface.
     */
    getBaseTypes(): Promise<readonly Type[] | undefined>;
    isClassOrInterface(): this is InterfaceType;
    isUnionType(): this is UnionType;
    isIntersectionType(): this is IntersectionType;
    isObjectType(): this is ObjectType;
    isIntrinsicType(): this is IntrinsicType;
    isErrorType(): boolean;
    isLiteralType(): this is LiteralType;
    isStringLiteralType(): this is StringLiteralType;
    isNumberLiteralType(): this is NumberLiteralType;
    isBigIntLiteralType(): this is BigIntLiteralType;
    isBooleanLiteralType(): this is BooleanLiteralType;
    isTypeReference(): this is TypeReference;
    isTupleType(): this is TupleType;
    isIndexType(): this is IndexType;
    isIndexedAccessType(): this is IndexedAccessType;
    isConditionalType(): this is ConditionalType;
    isSubstitutionType(): this is SubstitutionType;
    isTemplateLiteralType(): this is TemplateLiteralType;
    isStringMappingType(): this is StringMappingType;
    isTypeParameter(): this is TypeParameter;
}
export declare function isUnionType(type: Type): type is UnionType;
export declare function isIntersectionType(type: Type): type is IntersectionType;
export declare function isObjectType(type: Type): type is ObjectType;
export declare function isClassOrInterfaceType(type: Type): type is InterfaceType;
export declare function isIntrinsicType(type: Type): type is IntrinsicType;
/**
 * Whether this is the error type — the placeholder the checker produces when a
 * type cannot be determined (e.g. an unresolved reference). It is an intrinsic
 * type named `"error"` (this covers both the singleton error type and the
 * per-alias error types manufactured for unresolved type alias references).
 */
export declare function isErrorType(type: Type): boolean;
export declare function isLiteralType(type: Type): type is LiteralType;
export declare function isStringLiteralType(type: Type): type is StringLiteralType;
export declare function isNumberLiteralType(type: Type): type is NumberLiteralType;
export declare function isBigIntLiteralType(type: Type): type is BigIntLiteralType;
export declare function isBooleanLiteralType(type: Type): type is BooleanLiteralType;
export declare function isTypeReference(type: Type): type is TypeReference;
export declare function isTupleType(type: Type): type is TupleType;
export declare function isIndexType(type: Type): type is IndexType;
export declare function isIndexedAccessType(type: Type): type is IndexedAccessType;
export declare function isConditionalType(type: Type): type is ConditionalType;
export declare function isSubstitutionType(type: Type): type is SubstitutionType;
export declare function isTemplateLiteralType(type: Type): type is TemplateLiteralType;
export declare function isStringMappingType(type: Type): type is StringMappingType;
export declare function isTypeParameter(type: Type): type is TypeParameter;
export declare class Signature {
    private flags;
    private objectRegistry;
    readonly id: number;
    readonly declaration?: NodeHandle | undefined;
    readonly typeParameters?: readonly number[] | undefined;
    readonly parameters: readonly number[];
    readonly thisParameter?: number | undefined;
    readonly target?: number | undefined;
    constructor(data: SignatureResponse, project: Project, objectRegistry: ProjectObjectRegistry);
    getTypeParameters(): Promise<readonly TypeParameter[]>;
    getParameters(): Promise<readonly Symbol[]>;
    getThisParameter(): Promise<Symbol | undefined>;
    getTarget(): Promise<Signature | undefined>;
    get hasRestParameter(): boolean;
    get isConstruct(): boolean;
    get isAbstract(): boolean;
}
//# sourceMappingURL=api.d.ts.map