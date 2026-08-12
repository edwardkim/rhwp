//
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// !!! THIS FILE IS AUTO-GENERATED - DO NOT EDIT !!!
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
//
// Source: src/api/async/api.ts
// Regenerate: npm run generate (from _packages/native-preview)
//
/// <reference path="../node/node.ts" preserve="true" />
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
import { ModifierFlags, unescapeLeadingUnderscores, } from "../../ast/index.js";
import { encodeNode, uint8ArrayToBase64, } from "../node/encoder.js";
import { decodeNode, getNodeId, parseNodeHandle, readParseOptionsKey, readSourceFileHash, RemoteSourceFile, } from "../node/node.js";
import { Wtf8Decoder } from "../node/wtf8.js";
import { createGetCanonicalFileName, toPath, } from "../path.js";
import { resolveFileName, toUpdateSnapshotRequest, } from "../proto.js";
import { SourceFileCache } from "../sourceFileCache.js";
import { Client, } from "./client.js";
export { documentURIToFileName, fileNameToDocumentURI } from "../path.js";
export { CompletionItemKind, DiagnosticCategory, ElementFlags, ModifierFlags, ModuleKind, NodeBuilderFlags, ObjectFlags, SignatureFlags, SignatureKind, SymbolFlags, TypeFlags, TypePredicateKind };
export class API {
    client;
    sourceFileCache;
    toPath;
    initialized = false;
    activeSnapshots = new Set();
    latestSnapshot;
    internal;
    constructor(options = {}) {
        this.client = new Client(options);
        this.sourceFileCache = new SourceFileCache();
        this.internal = new InternalAPI(this.client, () => this.ensureInitialized());
    }
    /**
     * Create an API instance from an existing LSP connection's API session.
     * Use this when connecting to an API pipe provided by an LSP server via custom/initializeAPISession.
     */
    static fromLSPConnection(options) {
        const api = new API(options);
        api.ensureInitialized();
        return api;
    }
    ensureInitialized() {
        if (!this.initialized) {
            const response = this.client.apiRequest("initialize", null);
            const getCanonicalFileName = createGetCanonicalFileName(response.useCaseSensitiveFileNames);
            const currentDirectory = response.currentDirectory;
            this.toPath = (fileName) => toPath(fileName, currentDirectory, getCanonicalFileName);
            this.initialized = true;
        }
    }
    parseConfigFile(file) {
        this.ensureInitialized();
        return this.client.apiRequest("parseConfigFile", { file });
    }
    updateSnapshot(params) {
        this.ensureInitialized();
        const requestParams = toUpdateSnapshotRequest(params);
        const data = this.client.apiRequest("updateSnapshot", requestParams);
        // Retain cached source files from previous snapshot for unchanged files
        if (this.latestSnapshot) {
            this.sourceFileCache.retainForSnapshot(data.snapshot, this.latestSnapshot.id, data.changes);
            if (this.latestSnapshot.isDisposed()) {
                this.sourceFileCache.releaseSnapshot(this.latestSnapshot.id);
            }
        }
        const snapshot = new Snapshot(data, this.client, this.sourceFileCache, this.toPath, () => {
            this.activeSnapshots.delete(snapshot);
            if (snapshot !== this.latestSnapshot) {
                this.sourceFileCache.releaseSnapshot(snapshot.id);
            }
        });
        this.latestSnapshot = snapshot;
        this.activeSnapshots.add(snapshot);
        return snapshot;
    }
    close() {
        // Dispose all active snapshots
        for (const snapshot of [...this.activeSnapshots]) {
            snapshot.dispose();
        }
        // Release the latest snapshot's cache refs if still held
        if (this.latestSnapshot) {
            this.sourceFileCache.releaseSnapshot(this.latestSnapshot.id);
            this.latestSnapshot = undefined;
        }
        this.client.close();
        this.sourceFileCache.clear();
    }
    clearSourceFileCache() {
        this.sourceFileCache.clear();
    }
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
    getTimingInfo() {
        return this.client.getTimingInfo();
    }
    /** Clears all accumulated timing totals and recent-request history, on both the client and the server. */
    resetTimingInfo() {
        return this.client.resetTimingInfo();
    }
}
export class InternalAPI {
    client;
    ensureInitialized;
    /** @internal */
    constructor(client, ensureInitialized) {
        this.client = client;
        this.ensureInitialized = ensureInitialized;
    }
    startCPUProfile(dir) {
        this.ensureInitialized();
        this.client.apiRequest("startCPUProfile", { dir });
    }
    stopCPUProfile() {
        this.ensureInitialized();
        const result = this.client.apiRequest("stopCPUProfile", null);
        return result.file;
    }
    saveHeapProfile(dir) {
        this.ensureInitialized();
        const result = this.client.apiRequest("saveHeapProfile", { dir });
        return result.file;
    }
}
export class Snapshot {
    id;
    projectMap;
    toPath;
    client;
    disposed = false;
    onDispose;
    snapshotRegistry;
    constructor(data, client, sourceFileCache, toPath, onDispose) {
        this.id = data.snapshot;
        this.client = client;
        this.toPath = toPath;
        this.onDispose = onDispose;
        this.projectMap = new Map();
        this.snapshotRegistry = new SnapshotObjectRegistry(client, this.id, projectId => this.projectMap.get(projectId));
        for (const projData of data.projects) {
            const project = new Project(projData, this.id, client, sourceFileCache, toPath, this.snapshotRegistry);
            this.projectMap.set(toPath(projData.configFileName), project);
        }
    }
    getProjects() {
        this.ensureNotDisposed();
        return [...this.projectMap.values()];
    }
    getProject(configFileName) {
        this.ensureNotDisposed();
        return this.projectMap.get(this.toPath(configFileName));
    }
    getDefaultProjectForFile(file) {
        this.ensureNotDisposed();
        const data = this.client.apiRequest("getDefaultProjectForFile", {
            snapshot: this.id,
            file,
        });
        if (!data)
            return undefined;
        return this.projectMap.get(this.toPath(data.configFileName));
    }
    [globalThis.Symbol.dispose]() {
        this.dispose();
    }
    dispose() {
        if (this.disposed)
            return;
        this.disposed = true;
        for (const project of this.projectMap.values()) {
            project.dispose();
        }
        this.projectMap.clear();
        this.snapshotRegistry.clear();
        this.onDispose();
        this.client.apiRequest("release", { snapshot: this.id });
    }
    isDisposed() {
        return this.disposed;
    }
    ensureNotDisposed() {
        if (this.disposed) {
            throw new Error("Snapshot is disposed");
        }
    }
}
class SnapshotObjectRegistry {
    symbols = new Map();
    client;
    snapshotId;
    resolveProject;
    constructor(client, snapshotId, resolveProject) {
        this.client = client;
        this.snapshotId = snapshotId;
        this.resolveProject = resolveProject;
    }
    /** Resolve a project id (a config file path) to its Project within this snapshot. */
    getProject(projectId) {
        return this.resolveProject(projectId);
    }
    getOrCreateSymbol(data) {
        let symbol = this.symbols.get(data.id);
        if (!symbol) {
            symbol = new Symbol(data, this);
            this.symbols.set(data.id, symbol);
        }
        return symbol;
    }
    getSymbol(id) {
        return this.symbols.get(id);
    }
    clear() {
        this.symbols.clear();
    }
    fetchSymbol(source, method, handle, projectId) {
        if (!handle)
            return undefined;
        const cached = this.getSymbol(handle);
        if (cached)
            return cached;
        const data = this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: projectId,
            objectId: source.id,
        });
        if (!data)
            throw new Error(`${method} returned null symbol for ${source.constructor.name} ${source.id}`);
        return this.getOrCreateSymbol(data);
    }
    fetchSymbols(source, method, handles, projectId) {
        if (handles) {
            const result = new Array(handles.length);
            let allCached = true;
            for (let i = 0; i < handles.length; i++) {
                const cached = this.getSymbol(handles[i]);
                if (!cached) {
                    allCached = false;
                    break;
                }
                result[i] = cached;
            }
            if (allCached)
                return result;
        }
        const symbolData = this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: projectId,
            objectId: source.id,
        });
        if (symbolData == null)
            return [];
        else
            return symbolData.map(data => this.getOrCreateSymbol(data));
    }
}
class ProjectObjectRegistry {
    client;
    snapshotId;
    project;
    snapshotRegistry;
    types = new Map();
    signatures = new Map();
    constructor(client, snapshotId, project, snapshotRegistry) {
        this.client = client;
        this.snapshotId = snapshotId;
        this.project = project;
        this.snapshotRegistry = snapshotRegistry;
    }
    getOrCreateSymbol(data) {
        return this.snapshotRegistry.getOrCreateSymbol(data);
    }
    getSymbol(id) {
        return this.snapshotRegistry.getSymbol(id);
    }
    getOrCreateType(data) {
        let type = this.types.get(data.id);
        if (!type) {
            type = new TypeObject(data, this);
            this.types.set(data.id, type);
        }
        return type;
    }
    getType(id) {
        return this.types.get(id);
    }
    getOrCreateSignature(data) {
        let sig = this.signatures.get(data.id);
        if (!sig) {
            sig = new Signature(data, this.project, this);
            this.signatures.set(data.id, sig);
        }
        return sig;
    }
    getSignature(id) {
        return this.signatures.get(id);
    }
    clear() {
        this.types.clear();
        this.signatures.clear();
    }
    fetchType(source, method, handle) {
        if (handle !== false) {
            if (!handle)
                return undefined;
            const cached = this.getType(handle);
            if (cached)
                return cached;
        }
        const data = this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: this.project.id,
            objectId: source.id,
        });
        if (!data)
            throw new Error(`${method} returned null type for ${source.constructor.name} ${source.id}`);
        return this.getOrCreateType(data);
    }
    fetchSymbol(source, method, handle) {
        return this.snapshotRegistry.fetchSymbol(source, method, handle, this.project.id);
    }
    fetchSignature(source, method, handle) {
        if (!handle)
            return undefined;
        const cached = this.getSignature(handle);
        if (cached)
            return cached;
        const data = this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: this.project.id,
            objectId: source.id,
        });
        if (!data)
            throw new Error(`${method} returned null signature for ${source.constructor.name} ${source.id}`);
        return this.getOrCreateSignature(data);
    }
    fetchTypes(source, method, handles) {
        if (handles) {
            const result = new Array(handles.length);
            let allCached = true;
            for (let i = 0; i < handles.length; i++) {
                const cached = this.getType(handles[i]);
                if (!cached) {
                    allCached = false;
                    break;
                }
                result[i] = cached;
            }
            if (allCached)
                return result;
        }
        const typesData = this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: this.project.id,
            objectId: source.id,
        });
        if (typesData == null)
            return [];
        else
            return typesData.map(data => this.getOrCreateType(data));
    }
    fetchSymbols(source, method, handles) {
        return this.snapshotRegistry.fetchSymbols(source, method, handles, this.project.id);
    }
    // getBaseTypes is a checker-level endpoint keyed by `type` (not `objectId`),
    // so it cannot go through fetchTypes. This helper reuses that server method.
    fetchBaseTypes(source) {
        const typesData = this.client.apiRequest("getBaseTypes", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: source.id,
        });
        if (typesData == null)
            return [];
        return typesData.map(data => this.getOrCreateType(data));
    }
}
export class Project {
    id;
    configFileName;
    compilerOptions;
    rootFiles;
    program;
    checker;
    emitter;
    client;
    constructor(data, snapshotId, client, sourceFileCache, toPath, snapshotRegistry) {
        this.id = data.id;
        this.configFileName = data.configFileName;
        this.compilerOptions = data.compilerOptions;
        this.rootFiles = data.rootFiles;
        this.client = client;
        this.program = new Program(snapshotId, this, client, sourceFileCache, toPath);
        const objectRegistry = new ProjectObjectRegistry(client, snapshotId, this, snapshotRegistry);
        this.checker = new Checker(snapshotId, this, client, objectRegistry);
        this.emitter = new Emitter(client);
    }
    dispose() {
        this.checker.dispose();
    }
}
export class Program {
    snapshotId;
    project;
    client;
    sourceFileCache;
    toPath;
    decoder = new Wtf8Decoder();
    sourceFileMetadataCache = new Map();
    constructor(snapshotId, project, client, sourceFileCache, toPath) {
        this.snapshotId = snapshotId;
        this.project = project;
        this.client = client;
        this.sourceFileCache = sourceFileCache;
        this.toPath = toPath;
    }
    getCompilerOptions() {
        return this.project.compilerOptions;
    }
    getSourceFile(file) {
        const fileName = resolveFileName(file);
        const path = this.toPath(fileName);
        // Check if we already have a retained cache entry for this (snapshot, project) pair
        const retained = this.sourceFileCache.getRetained(path, this.snapshotId, this.project.id);
        if (retained) {
            return retained;
        }
        // Fetch from server
        const binaryData = this.client.apiRequestBinary("getSourceFile", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file,
        });
        if (!binaryData) {
            return undefined;
        }
        const view = new DataView(binaryData.buffer, binaryData.byteOffset, binaryData.byteLength);
        const contentHash = readSourceFileHash(view);
        const parseOptionsKey = readParseOptionsKey(view);
        // Create a new RemoteSourceFile and cache it (set returns existing if hash matches)
        const sourceFile = new RemoteSourceFile(binaryData, this.decoder, this.client.getTimingCollector());
        return this.sourceFileCache.set(path, sourceFile, parseOptionsKey, contentHash, this.snapshotId, this.project.id);
    }
    getSourceFileNames() {
        const data = this.client.apiRequest("getSourceFileNames", {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
        return data ?? [];
    }
    /**
     * Returns program-stored metadata for the given source file, or `undefined` if the file
     * is not part of the program. Metadata is fetched lazily per file and cached on this
     * `Program` instance.
     */
    getSourceFileMetadata(fileName) {
        return this.getSourceFileMetadataByPath(this.toPath(fileName));
    }
    /**
     * Returns program-stored metadata for the source file at the given path, or `undefined`
     * if the file is not part of the program. Like {@link getSourceFileMetadata}, but skips
     * the file name to path conversion. Metadata is fetched lazily per file and cached on
     * this `Program` instance.
     */
    getSourceFileMetadataByPath(path) {
        let metadata = this.sourceFileMetadataCache.get(path);
        if (metadata === undefined) {
            metadata = this.fetchSourceFileMetadata(path);
            this.sourceFileMetadataCache.set(path, metadata);
        }
        return metadata;
    }
    fetchSourceFileMetadata(path) {
        const data = this.client.apiRequest("getSourceFileMetadata", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file: path,
        });
        return data ?? undefined;
    }
    /**
     * Returns whether the given source file was loaded as part of an external library
     * (e.g. a dependency resolved from `node_modules`). The underlying program metadata is
     * fetched lazily per file and cached on this `Program` instance.
     */
    isSourceFileFromExternalLibrary(file) {
        const metadata = this.getSourceFileMetadataByPath(file.path);
        return metadata?.isFromExternalLibrary ?? false;
    }
    /**
     * Returns whether the given source file is a default library file (e.g. `lib.d.ts`).
     * The underlying program metadata is fetched lazily per file and cached on this
     * `Program` instance.
     */
    isSourceFileDefaultLibrary(file) {
        const metadata = this.getSourceFileMetadataByPath(file.path);
        return metadata?.isDefaultLibrary ?? false;
    }
    /**
     * Get syntactic (parse) diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getSyntacticDiagnostics(file) {
        const data = this.client.apiRequest("getSyntacticDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
            ...(file !== undefined ? { file } : {}),
        });
        return data ?? [];
    }
    /**
     * Get binder diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getBindDiagnostics(file) {
        const data = this.client.apiRequest("getBindDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
            ...(file !== undefined ? { file } : {}),
        });
        return data ?? [];
    }
    /**
     * Get semantic (type-check) diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getSemanticDiagnostics(file) {
        const data = this.client.apiRequest("getSemanticDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
            ...(file !== undefined ? { file } : {}),
        });
        return data ?? [];
    }
    /**
     * Get suggestion diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getSuggestionDiagnostics(file) {
        const data = this.client.apiRequest("getSuggestionDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
            ...(file !== undefined ? { file } : {}),
        });
        return data ?? [];
    }
    /**
     * Get declaration emit diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    getDeclarationDiagnostics(file) {
        const data = this.client.apiRequest("getDeclarationDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
            ...(file !== undefined ? { file } : {}),
        });
        return data ?? [];
    }
    /**
     * Get program-wide diagnostics for the project, including compiler options diagnostics.
     */
    getProgramDiagnostics() {
        const data = this.client.apiRequest("getProgramDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
        return data ?? [];
    }
    /**
     * Get global (non-file-specific) semantic diagnostics for the project.
     */
    getGlobalDiagnostics() {
        const data = this.client.apiRequest("getGlobalDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
        return data ?? [];
    }
    /**
     * Get config file parsing diagnostics for the project.
     */
    getConfigFileParsingDiagnostics() {
        const data = this.client.apiRequest("getConfigFileParsingDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
        return data ?? [];
    }
}
export class Checker {
    snapshotId;
    project;
    client;
    objectRegistry;
    wellKnownSymbols;
    constructor(snapshotId, project, client, objectRegistry) {
        this.snapshotId = snapshotId;
        this.project = project;
        this.client = client;
        this.objectRegistry = objectRegistry;
    }
    dispose() {
        this.objectRegistry.clear();
    }
    getSymbolAtLocation(nodeOrNodes) {
        if (Array.isArray(nodeOrNodes)) {
            const data = this.client.apiRequest("getSymbolsAtLocations", {
                snapshot: this.snapshotId,
                project: this.project.id,
                locations: nodeOrNodes.map(node => getNodeId(node)),
            });
            return data.map(d => d ? this.objectRegistry.getOrCreateSymbol(d) : undefined);
        }
        const data = this.client.apiRequest("getSymbolAtLocation", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(nodeOrNodes),
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    getSymbolAtPosition(file, positionOrPositions) {
        if (typeof positionOrPositions === "number") {
            const data = this.client.apiRequest("getSymbolAtPosition", {
                snapshot: this.snapshotId,
                project: this.project.id,
                file,
                position: positionOrPositions,
            });
            return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
        }
        const data = this.client.apiRequest("getSymbolsAtPositions", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file,
            positions: positionOrPositions,
        });
        return data.map(d => d ? this.objectRegistry.getOrCreateSymbol(d) : undefined);
    }
    getTypeOfSymbol(symbolOrSymbols) {
        if (Array.isArray(symbolOrSymbols)) {
            const data = this.client.apiRequest("getTypesOfSymbols", {
                snapshot: this.snapshotId,
                project: this.project.id,
                symbols: symbolOrSymbols.map(s => s.id),
            });
            return data.map(d => d ? this.objectRegistry.getOrCreateType(d) : undefined);
        }
        const data = this.client.apiRequest("getTypeOfSymbol", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbolOrSymbols.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    /**
     * Get the declared type of a symbol. Always returns a type; for symbols whose
     * declared type cannot be determined the checker yields the error type (use
     * {@link Type.isErrorType} to detect it).
     */
    getDeclaredTypeOfSymbol(symbol) {
        const data = this.client.apiRequest("getDeclaredTypeOfSymbol", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        if (!data)
            throw new Error(`getDeclaredTypeOfSymbol returned no type for symbol ${symbol.id}`);
        return this.objectRegistry.getOrCreateType(data);
    }
    getReferencesToSymbolInFile(file, symbol) {
        const data = this.client.apiRequest("getReferencesToSymbolInFile", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file,
            symbol: symbol.id,
        });
        return (data ?? []).map(h => new NodeHandle(h, this.project));
    }
    getReferencedSymbolsForNode(node, position) {
        const data = this.client.apiRequest("getReferencedSymbolsForNode", {
            snapshot: this.snapshotId,
            project: this.project.id,
            node: getNodeId(node),
            position,
        });
        return (data ?? []).map(entry => ({
            definition: new NodeHandle(entry.definition, this.project),
            symbol: entry.symbol ? this.objectRegistry.getOrCreateSymbol(entry.symbol) : undefined,
            references: (entry.references ?? []).map(h => new NodeHandle(h, this.project)),
        }));
    }
    getSignatureUsage(signatureDecl) {
        const data = this.client.apiRequest("getSignatureUsages", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signatureDecl: getNodeId(signatureDecl),
        });
        return (data ?? []).map(entry => ({
            name: new NodeHandle(entry.name, this.project),
            call: entry.call ? new NodeHandle(entry.call, this.project) : undefined,
        }));
    }
    getCompletionsAtPosition(document, position, options) {
        const data = this.client.apiRequest("getCompletionsAtPosition", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file: document,
            position,
            triggerCharacter: options?.triggerCharacter,
            includeSymbol: options?.includeSymbol,
        });
        if (!data)
            return undefined;
        return {
            isIncomplete: data.isIncomplete,
            entries: data.entries.map(e => ({
                ...e,
                symbol: e.symbol ? this.objectRegistry.getOrCreateSymbol(e.symbol) : undefined,
            })),
        };
    }
    getTypeAtLocation(nodeOrNodes) {
        if (Array.isArray(nodeOrNodes)) {
            const data = this.client.apiRequest("getTypeAtLocations", {
                snapshot: this.snapshotId,
                project: this.project.id,
                locations: nodeOrNodes.map(node => getNodeId(node)),
            });
            return data.map(d => d ? this.objectRegistry.getOrCreateType(d) : undefined);
        }
        const data = this.client.apiRequest("getTypeAtLocation", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(nodeOrNodes),
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getSignaturesOfType(type, kind) {
        const data = this.client.apiRequest("getSignaturesOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
            kind,
        });
        return data.map(d => this.objectRegistry.getOrCreateSignature(d));
    }
    getResolvedSignature(node) {
        const data = this.client.apiRequest("getResolvedSignature", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateSignature(data) : undefined;
    }
    getTypeAtPosition(file, positionOrPositions) {
        if (typeof positionOrPositions === "number") {
            const data = this.client.apiRequest("getTypeAtPosition", {
                snapshot: this.snapshotId,
                project: this.project.id,
                file,
                position: positionOrPositions,
            });
            return data ? this.objectRegistry.getOrCreateType(data) : undefined;
        }
        const data = this.client.apiRequest("getTypesAtPositions", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file,
            positions: positionOrPositions,
        });
        return data.map(d => d ? this.objectRegistry.getOrCreateType(d) : undefined);
    }
    resolveName(name, meaning, location, excludeGlobals) {
        // Distinguish Node (has `kind`) from DocumentPosition (has `document` and `position`)
        const isNode = location && "kind" in location;
        const data = this.client.apiRequest("resolveName", {
            snapshot: this.snapshotId,
            project: this.project.id,
            name,
            meaning,
            location: isNode ? getNodeId(location) : undefined,
            file: !isNode && location ? location.document : undefined,
            position: !isNode && location ? location.position : undefined,
            excludeGlobals,
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    getResolvedSymbol(node) {
        const text = node.text;
        if (!text)
            return undefined;
        return this.resolveName(text, SymbolFlags.Value | SymbolFlags.ExportValue, node);
    }
    getContextualType(node) {
        const data = this.client.apiRequest("getContextualType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getBaseTypeOfLiteralType(type) {
        const data = this.client.apiRequest("getBaseTypeOfLiteralType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getNonNullableType(type) {
        const data = this.client.apiRequest("getNonNullableType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getTypeFromTypeNode(node) {
        const data = this.client.apiRequest("getTypeFromTypeNode", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getWidenedType(type) {
        const data = this.client.apiRequest("getWidenedType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getParameterType(signature, index) {
        const data = this.client.apiRequest("getParameterType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signature: signature.id,
            index,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    isArrayLikeType(type) {
        return this.client.apiRequest("isArrayLikeType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
    }
    isTypeAssignableTo(source, target) {
        return this.client.apiRequest("isTypeAssignableTo", {
            snapshot: this.snapshotId,
            project: this.project.id,
            source: source.id,
            target: target.id,
        });
    }
    getShorthandAssignmentValueSymbol(node) {
        const data = this.client.apiRequest("getShorthandAssignmentValueSymbol", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    /**
     * Get the type of a symbol as narrowed at a specific location. Always returns
     * a type; for symbols whose type cannot be determined the checker yields the
     * error type (use {@link Type.isErrorType} to detect it).
     */
    getTypeOfSymbolAtLocation(symbol, location) {
        const data = this.client.apiRequest("getTypeOfSymbolAtLocation", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
            location: getNodeId(location),
        });
        if (!data)
            throw new Error(`getTypeOfSymbolAtLocation returned no type for symbol ${symbol.id}`);
        return this.objectRegistry.getOrCreateType(data);
    }
    getIntrinsicType(method) {
        const data = this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
        return this.objectRegistry.getOrCreateType(data);
    }
    getAnyType() {
        return this.getIntrinsicType("getAnyType");
    }
    getStringType() {
        return this.getIntrinsicType("getStringType");
    }
    getNumberType() {
        return this.getIntrinsicType("getNumberType");
    }
    getBooleanType() {
        return this.getIntrinsicType("getBooleanType");
    }
    getVoidType() {
        return this.getIntrinsicType("getVoidType");
    }
    getUndefinedType() {
        return this.getIntrinsicType("getUndefinedType");
    }
    getNullType() {
        return this.getIntrinsicType("getNullType");
    }
    getNeverType() {
        return this.getIntrinsicType("getNeverType");
    }
    getUnknownType() {
        return this.getIntrinsicType("getUnknownType");
    }
    getBigIntType() {
        return this.getIntrinsicType("getBigIntType");
    }
    getESSymbolType() {
        return this.getIntrinsicType("getESSymbolType");
    }
    typeToTypeNode(type, enclosingDeclaration, flags) {
        const binaryData = this.client.apiRequestBinary("typeToTypeNode", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
            location: enclosingDeclaration ? getNodeId(enclosingDeclaration) : undefined,
            flags,
        });
        if (!binaryData)
            return undefined;
        return decodeNode(binaryData);
    }
    signatureToSignatureDeclaration(signature, kind, enclosingDeclaration, flags) {
        const binaryData = this.client.apiRequestBinary("signatureToSignatureDeclaration", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signature: signature.id,
            kind,
            location: enclosingDeclaration ? getNodeId(enclosingDeclaration) : undefined,
            flags,
        });
        if (!binaryData)
            return undefined;
        return decodeNode(binaryData);
    }
    typeToString(type, enclosingDeclaration, flags) {
        return this.client.apiRequest("typeToString", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
            location: enclosingDeclaration ? getNodeId(enclosingDeclaration) : undefined,
            flags,
        });
    }
    isContextSensitive(node) {
        return this.client.apiRequest("isContextSensitive", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
    }
    isArrayType(type) {
        return this.client.apiRequest("isArrayType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
    }
    isTupleType(type) {
        return this.client.apiRequest("isTupleType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
    }
    getReturnTypeOfSignature(signature) {
        const data = this.client.apiRequest("getReturnTypeOfSignature", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signature: signature.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getRestTypeOfSignature(signature) {
        const data = this.client.apiRequest("getRestTypeOfSignature", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signature: signature.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getTypePredicateOfSignature(signature) {
        const data = this.client.apiRequest("getTypePredicateOfSignature", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signature: signature.id,
        });
        if (!data)
            return undefined;
        return {
            kind: data.kind,
            parameterIndex: data.parameterIndex,
            parameterName: data.parameterName,
            type: data.type ? this.objectRegistry.getOrCreateType(data.type) : undefined,
        };
    }
    /**
     * Get the base types of a class or interface type. A type with no base types
     * yields an empty array.
     */
    getBaseTypes(type) {
        const data = this.client.apiRequest("getBaseTypes", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? data.map(d => this.objectRegistry.getOrCreateType(d)) : [];
    }
    getApparentType(type) {
        const data = this.client.apiRequest("getApparentType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getPropertiesOfType(type) {
        const data = this.client.apiRequest("getPropertiesOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? data.map(d => this.objectRegistry.getOrCreateSymbol(d)) : [];
    }
    getIndexInfosOfType(type) {
        const data = this.client.apiRequest("getIndexInfosOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        if (!data)
            return [];
        return data.map(d => ({
            keyType: this.objectRegistry.getOrCreateType(d.keyType),
            valueType: this.objectRegistry.getOrCreateType(d.valueType),
            isReadonly: d.isReadonly ?? false,
            declaration: d.declaration ? new NodeHandle(d.declaration, this.project) : undefined,
        }));
    }
    /**
     * Get the constraint of a type parameter (the `T` in `<U extends T>`), or
     * undefined if it has none.
     */
    getConstraintOfTypeParameter(type) {
        const data = this.client.apiRequest("getConstraintOfTypeParameter", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getBaseConstraintOfType(type) {
        const data = this.client.apiRequest("getBaseConstraintOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    getPropertyOfType(type, name) {
        const data = this.client.apiRequest("getPropertyOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
            name,
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    getConstantValue(node) {
        const data = this.client.apiRequest("getConstantValue", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ?? undefined;
    }
    getSignatureFromDeclaration(node) {
        const data = this.client.apiRequest("getSignatureFromDeclaration", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateSignature(data) : undefined;
    }
    getExportSpecifierLocalTargetSymbol(node) {
        const data = this.client.apiRequest("getExportSpecifierLocalTargetSymbol", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    /**
     * Follow all aliases to get the original symbol. Always returns a symbol; for
     * an unresolved alias the checker yields the unknown symbol (use
     * {@link Checker.isUnknownSymbol} to detect it).
     */
    getAliasedSymbol(symbol) {
        const data = this.client.apiRequest("getAliasedSymbol", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        if (!data)
            throw new Error(`getAliasedSymbol returned no symbol for symbol ${symbol.id}`);
        return this.objectRegistry.getOrCreateSymbol(data);
    }
    getImmediateAliasedSymbol(symbol) {
        const data = this.client.apiRequest("getImmediateAliasedSymbol", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    /**
     * Fetch (once, then cache) the handle ids of the per-checker singleton
     * symbols (unknown, undefined, arguments). These ids are stable for the life
     * of the project's checker, so identity checks against them are local after
     * the first call.
     */
    getWellKnownSymbols() {
        return this.wellKnownSymbols ??= this.client.apiRequest("getWellKnownSymbols", {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
    }
    /**
     * Returns `true` if the symbol is the checker's "unknown" symbol (e.g. the
     * result of {@link Checker.getAliasedSymbol} on an unresolved alias).
     */
    isUnknownSymbol(symbol) {
        return symbol.id === (this.getWellKnownSymbols()).unknown;
    }
    /**
     * Returns `true` if the symbol is the checker's "undefined" symbol.
     */
    isUndefinedSymbol(symbol) {
        return symbol.id === (this.getWellKnownSymbols()).undefined;
    }
    /**
     * Returns `true` if the symbol is the checker's "arguments" symbol.
     */
    isArgumentsSymbol(symbol) {
        return symbol.id === (this.getWellKnownSymbols()).arguments;
    }
    getExportsOfModule(symbol) {
        const data = this.client.apiRequest("getExportsOfModule", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        return data ? data.map(d => this.objectRegistry.getOrCreateSymbol(d)) : [];
    }
    getMemberInModuleExports(symbol, name) {
        const data = this.client.apiRequest("getMemberInModuleExports", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
            name,
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    getJsDocTagsOfSymbol(symbol) {
        const data = this.client.apiRequest("getJsDocTags", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        return data ?? [];
    }
    getDocumentationCommentOfSymbol(symbol) {
        return this.client.apiRequest("getDocumentationComment", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
    }
    /**
     * Get the type arguments of a type reference (e.g. the `string` in `Array<string>`).
     */
    getTypeArguments(type) {
        const data = this.client.apiRequest("getTypeArguments", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? data.map(d => this.objectRegistry.getOrCreateType(d)) : [];
    }
}
export class Emitter {
    client;
    constructor(client) {
        this.client = client;
    }
    printNode(node, options = {}) {
        const encoded = encodeNode(node);
        const base64 = uint8ArrayToBase64(encoded);
        return this.client.apiRequest("printNode", {
            data: base64,
            ...options,
        });
    }
}
export class NodeHandle {
    /**
     * The project this handle was produced in, used as the default for {@link resolve}.
     * Node handles are only meaningful within a project's program, so the producing project
     * is remembered so callers don't have to pass it explicitly.
     */
    canonicalProject;
    index;
    kind;
    path;
    constructor(handle, canonicalProject) {
        const parsed = parseNodeHandle(handle);
        this.index = parsed.index;
        this.kind = parsed.kind;
        this.path = parsed.path;
        this.canonicalProject = canonicalProject;
    }
    /**
     * Resolve this handle to the actual AST node by fetching the source file from a project
     * and looking up the node by index. If no project is passed, the project that produced
     * the handle is used.
     */
    resolve(project = this.canonicalProject) {
        const sourceFile = project.program.getSourceFile(this.path);
        if (!sourceFile) {
            return undefined;
        }
        return sourceFile.getOrCreateNodeAtIndex(this.index);
    }
}
export class Symbol {
    objectRegistry;
    /**
     * The project this symbol was first observed in, used as the default project for
     * lookups that need a project context (members/exports/parent). Symbols are shared
     * snapshot-wide, so these lookups can otherwise be ambiguous about which project to use.
     */
    canonicalProject;
    id;
    /** The escaped (`__String`) name, used as the key in member/export tables. */
    escapedName;
    /** The display name (escaped underscores removed). */
    name;
    flags;
    checkFlags;
    declarations;
    valueDeclaration;
    parent;
    exportSymbol;
    membersCache;
    exportsCache;
    constructor(data, objectRegistry) {
        this.objectRegistry = objectRegistry;
        this.id = data.id;
        this.escapedName = data.name;
        this.name = unescapeLeadingUnderscores(data.name);
        this.flags = data.flags;
        this.checkFlags = data.checkFlags;
        const canonicalProject = objectRegistry.getProject(data.project);
        if (!canonicalProject) {
            throw new Error(`Symbol ${data.id} references unknown canonical project '${data.project}'`);
        }
        this.canonicalProject = canonicalProject;
        this.declarations = (data.declarations ?? []).map(d => new NodeHandle(d, canonicalProject));
        this.valueDeclaration = data.valueDeclaration ? new NodeHandle(data.valueDeclaration, canonicalProject) : undefined;
        if (data.parent !== undefined)
            this.parent = data.parent;
        if (data.exportSymbol !== undefined)
            this.exportSymbol = data.exportSymbol;
    }
    getParent() {
        return this.objectRegistry.fetchSymbol(this, "getParentOfSymbol", this.parent, this.canonicalProject.id);
    }
    /**
     * Get this symbol's members keyed by escaped name. The result is cached on
     * the symbol, so repeated calls do not round-trip to the server.
     */
    getMembers() {
        return this.membersCache ??= this.fetchSymbolTable("getMembersOfSymbol");
    }
    /**
     * Get this symbol's exports keyed by escaped name. The result is cached on
     * the symbol, so repeated calls do not round-trip to the server.
     */
    getExports() {
        return this.exportsCache ??= this.fetchSymbolTable("getExportsOfSymbol");
    }
    fetchSymbolTable(method) {
        const symbols = this.objectRegistry.fetchSymbols(this, method, undefined, this.canonicalProject.id);
        const table = new Map();
        for (const symbol of symbols) {
            table.set(symbol.escapedName, symbol);
        }
        return table;
    }
    getExportSymbol() {
        if (!this.exportSymbol)
            return this;
        return this.objectRegistry.fetchSymbol(this, "getExportSymbolOfSymbol", this.exportSymbol, this.canonicalProject.id);
    }
    getJsDocTags(checker) {
        return checker.getJsDocTagsOfSymbol(this);
    }
    getDocumentationComment(checker) {
        return checker.getDocumentationCommentOfSymbol(this);
    }
}
class TypeObject {
    objectRegistry;
    id;
    flags;
    objectFlags;
    symbol;
    value;
    intrinsicName;
    isThisType;
    freshType;
    regularType;
    target;
    typeParameters;
    outerTypeParameters;
    localTypeParameters;
    aliasTypeArguments;
    aliasSymbol;
    elementFlags;
    fixedLength;
    readonly;
    texts;
    objectType;
    indexType;
    checkType;
    extendsType;
    baseType;
    substConstraint;
    trueType; // false if not yet loaded
    falseType; // false if not yet loaded
    constructor(data, objectRegistry) {
        this.objectRegistry = objectRegistry;
        this.id = data.id;
        this.flags = data.flags;
        if (data.objectFlags !== undefined)
            this.objectFlags = data.objectFlags;
        if (data.symbol !== undefined)
            this.symbol = data.symbol;
        if (data.value != null) {
            // BigInt literal values are serialized as decimal strings (e.g. "-123") because
            // JSON cannot represent bigint. Decode them back into a real bigint here.
            this.value = (data.flags & TypeFlags.BigIntLiteral) ? BigInt(data.value) : data.value;
        }
        if (data.intrinsicName !== undefined)
            this.intrinsicName = data.intrinsicName;
        if (data.isThisType !== undefined)
            this.isThisType = data.isThisType;
        if (data.freshType !== undefined)
            this.freshType = data.freshType;
        if (data.regularType !== undefined)
            this.regularType = data.regularType;
        if (data.target !== undefined)
            this.target = data.target;
        this.typeParameters = data.typeParameters ?? [];
        this.outerTypeParameters = data.outerTypeParameters ?? [];
        this.localTypeParameters = data.localTypeParameters ?? [];
        this.aliasTypeArguments = data.aliasTypeArguments ?? [];
        if (data.aliasSymbol !== undefined)
            this.aliasSymbol = data.aliasSymbol;
        if (data.elementFlags !== undefined)
            this.elementFlags = data.elementFlags;
        if (data.fixedLength !== undefined)
            this.fixedLength = data.fixedLength;
        if (data.readonly !== undefined)
            this.readonly = data.readonly;
        if (data.texts !== undefined)
            this.texts = data.texts;
        if (data.objectType !== undefined)
            this.objectType = data.objectType;
        if (data.indexType !== undefined)
            this.indexType = data.indexType;
        if (data.checkType !== undefined)
            this.checkType = data.checkType;
        if (data.extendsType !== undefined)
            this.extendsType = data.extendsType;
        if (data.baseType !== undefined)
            this.baseType = data.baseType;
        if (data.substConstraint !== undefined)
            this.substConstraint = data.substConstraint;
        this.trueType = false;
        this.falseType = false;
    }
    getSymbol() {
        return this.objectRegistry.fetchSymbol(this, "getSymbolOfType", this.symbol);
    }
    getAliasSymbol() {
        return this.objectRegistry.fetchSymbol(this, "getAliasSymbolOfType", this.aliasSymbol);
    }
    getTarget() {
        return this.objectRegistry.fetchType(this, "getTargetOfType", this.target);
    }
    getFreshType() {
        return this.objectRegistry.fetchType(this, "getFreshTypeOfType", this.freshType);
    }
    getRegularType() {
        return this.objectRegistry.fetchType(this, "getRegularTypeOfType", this.regularType);
    }
    getTypes() {
        // Only union, intersection, and template literal types have constituent
        // types; any other kind has none, so return undefined rather than sending
        // a request the server cannot satisfy.
        if (!(this.flags & (TypeFlags.UnionOrIntersection | TypeFlags.TemplateLiteral))) {
            return undefined;
        }
        return this.objectRegistry.fetchTypes(this, "getTypesOfType");
    }
    getTypeParameters() {
        return this.objectRegistry.fetchTypes(this, "getTypeParametersOfType", this.typeParameters);
    }
    getOuterTypeParameters() {
        return this.objectRegistry.fetchTypes(this, "getOuterTypeParametersOfType", this.outerTypeParameters);
    }
    getLocalTypeParameters() {
        return this.objectRegistry.fetchTypes(this, "getLocalTypeParametersOfType", this.localTypeParameters);
    }
    getAliasTypeArguments() {
        return this.objectRegistry.fetchTypes(this, "getAliasTypeArgumentsOfType", this.aliasTypeArguments);
    }
    getObjectType() {
        return this.objectRegistry.fetchType(this, "getObjectTypeOfType", this.objectType);
    }
    getIndexType() {
        return this.objectRegistry.fetchType(this, "getIndexTypeOfType", this.indexType);
    }
    getCheckType() {
        return this.objectRegistry.fetchType(this, "getCheckTypeOfType", this.checkType);
    }
    getExtendsType() {
        return this.objectRegistry.fetchType(this, "getExtendsTypeOfType", this.extendsType);
    }
    getBaseType() {
        return this.objectRegistry.fetchType(this, "getBaseTypeOfType", this.baseType);
    }
    getConstraint() {
        return this.objectRegistry.fetchType(this, "getConstraintOfType", this.substConstraint);
    }
    getTrueType() {
        const result = this.objectRegistry.fetchType(this, "getTrueTypeOfConditionalType", this.trueType);
        this.trueType = result.id;
        return result;
    }
    getFalseType() {
        const result = this.objectRegistry.fetchType(this, "getFalseTypeOfConditionalType", this.falseType);
        this.falseType = result.id;
        return result;
    }
    /**
     * Get the base types of this type. Returns `undefined` for any type that is
     * not a class or interface.
     */
    getBaseTypes() {
        if (!this.isClassOrInterface()) {
            return undefined;
        }
        return this.objectRegistry.fetchBaseTypes(this);
    }
    isClassOrInterface() {
        return isClassOrInterfaceType(this);
    }
    isUnionType() {
        return isUnionType(this);
    }
    isIntersectionType() {
        return isIntersectionType(this);
    }
    isObjectType() {
        return isObjectType(this);
    }
    isIntrinsicType() {
        return isIntrinsicType(this);
    }
    isErrorType() {
        return isErrorType(this);
    }
    isLiteralType() {
        return isLiteralType(this);
    }
    isStringLiteralType() {
        return isStringLiteralType(this);
    }
    isNumberLiteralType() {
        return isNumberLiteralType(this);
    }
    isBigIntLiteralType() {
        return isBigIntLiteralType(this);
    }
    isBooleanLiteralType() {
        return isBooleanLiteralType(this);
    }
    isTypeReference() {
        return isTypeReference(this);
    }
    isTupleType() {
        return isTupleType(this);
    }
    isIndexType() {
        return isIndexType(this);
    }
    isIndexedAccessType() {
        return isIndexedAccessType(this);
    }
    isConditionalType() {
        return isConditionalType(this);
    }
    isSubstitutionType() {
        return isSubstitutionType(this);
    }
    isTemplateLiteralType() {
        return isTemplateLiteralType(this);
    }
    isStringMappingType() {
        return isStringMappingType(this);
    }
    isTypeParameter() {
        return isTypeParameter(this);
    }
}
export function isUnionType(type) {
    return (type.flags & TypeFlags.Union) !== 0;
}
export function isIntersectionType(type) {
    return (type.flags & TypeFlags.Intersection) !== 0;
}
export function isObjectType(type) {
    return (type.flags & TypeFlags.Object) !== 0;
}
export function isClassOrInterfaceType(type) {
    return isObjectType(type) && (type.objectFlags & ObjectFlags.ClassOrInterface) !== 0;
}
export function isIntrinsicType(type) {
    return (type.flags & TypeFlags.Intrinsic) !== 0;
}
/**
 * Whether this is the error type — the placeholder the checker produces when a
 * type cannot be determined (e.g. an unresolved reference). It is an intrinsic
 * type named `"error"` (this covers both the singleton error type and the
 * per-alias error types manufactured for unresolved type alias references).
 */
export function isErrorType(type) {
    return isIntrinsicType(type) && type.intrinsicName === "error";
}
export function isLiteralType(type) {
    return (type.flags & TypeFlags.Literal) !== 0;
}
export function isStringLiteralType(type) {
    return (type.flags & TypeFlags.StringLiteral) !== 0;
}
export function isNumberLiteralType(type) {
    return (type.flags & TypeFlags.NumberLiteral) !== 0;
}
export function isBigIntLiteralType(type) {
    return (type.flags & TypeFlags.BigIntLiteral) !== 0;
}
export function isBooleanLiteralType(type) {
    return (type.flags & TypeFlags.BooleanLiteral) !== 0;
}
export function isTypeReference(type) {
    return isObjectType(type) && (type.objectFlags & ObjectFlags.Reference) !== 0;
}
export function isTupleType(type) {
    return isObjectType(type) && (type.objectFlags & ObjectFlags.Tuple) !== 0;
}
export function isIndexType(type) {
    return (type.flags & TypeFlags.Index) !== 0;
}
export function isIndexedAccessType(type) {
    return (type.flags & TypeFlags.IndexedAccess) !== 0;
}
export function isConditionalType(type) {
    return (type.flags & TypeFlags.Conditional) !== 0;
}
export function isSubstitutionType(type) {
    return (type.flags & TypeFlags.Substitution) !== 0;
}
export function isTemplateLiteralType(type) {
    return (type.flags & TypeFlags.TemplateLiteral) !== 0;
}
export function isStringMappingType(type) {
    return (type.flags & TypeFlags.StringMapping) !== 0;
}
export function isTypeParameter(type) {
    return (type.flags & TypeFlags.TypeParameter) !== 0;
}
export class Signature {
    flags;
    objectRegistry;
    id;
    declaration;
    typeParameters;
    parameters;
    thisParameter;
    target;
    constructor(data, project, objectRegistry) {
        this.id = data.id;
        this.flags = data.flags;
        this.objectRegistry = objectRegistry;
        this.declaration = data.declaration ? new NodeHandle(data.declaration, project) : undefined;
        this.typeParameters = data.typeParameters ?? [];
        this.parameters = data.parameters ?? [];
        this.thisParameter = data.thisParameter;
        this.target = data.target;
    }
    getTypeParameters() {
        return this.objectRegistry.fetchTypes(this, "getTypeParametersOfSignature", this.typeParameters);
    }
    getParameters() {
        return this.objectRegistry.fetchSymbols(this, "getParametersOfSignature", this.parameters);
    }
    getThisParameter() {
        return this.objectRegistry.fetchSymbol(this, "getThisParameterOfSignature", this.thisParameter);
    }
    getTarget() {
        return this.objectRegistry.fetchSignature(this, "getTargetOfSignature", this.target);
    }
    get hasRestParameter() {
        return (this.flags & SignatureFlags.HasRestParameter) !== 0;
    }
    get isConstruct() {
        return (this.flags & SignatureFlags.Construct) !== 0;
    }
    get isAbstract() {
        return (this.flags & SignatureFlags.Abstract) !== 0;
    }
}
//# sourceMappingURL=api.js.map