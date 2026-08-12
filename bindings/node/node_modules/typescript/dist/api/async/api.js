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
    static async fromLSPConnection(options) {
        const api = new API(options);
        await api.ensureInitialized();
        return api;
    }
    async ensureInitialized() {
        if (!this.initialized) {
            const response = await this.client.apiRequest("initialize", null);
            const getCanonicalFileName = createGetCanonicalFileName(response.useCaseSensitiveFileNames);
            const currentDirectory = response.currentDirectory;
            this.toPath = (fileName) => toPath(fileName, currentDirectory, getCanonicalFileName);
            this.initialized = true;
        }
    }
    async parseConfigFile(file) {
        await this.ensureInitialized();
        return this.client.apiRequest("parseConfigFile", { file });
    }
    async updateSnapshot(params) {
        await this.ensureInitialized();
        const requestParams = toUpdateSnapshotRequest(params);
        const data = await this.client.apiRequest("updateSnapshot", requestParams);
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
    async close() {
        // Dispose all active snapshots
        for (const snapshot of [...this.activeSnapshots]) {
            await snapshot.dispose();
        }
        // Release the latest snapshot's cache refs if still held
        if (this.latestSnapshot) {
            this.sourceFileCache.releaseSnapshot(this.latestSnapshot.id);
            this.latestSnapshot = undefined;
        }
        await this.client.close();
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
    async startCPUProfile(dir) {
        await this.ensureInitialized();
        await this.client.apiRequest("startCPUProfile", { dir });
    }
    async stopCPUProfile() {
        await this.ensureInitialized();
        const result = await this.client.apiRequest("stopCPUProfile", null);
        return result.file;
    }
    async saveHeapProfile(dir) {
        await this.ensureInitialized();
        const result = await this.client.apiRequest("saveHeapProfile", { dir });
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
    async getDefaultProjectForFile(file) {
        this.ensureNotDisposed();
        const data = await this.client.apiRequest("getDefaultProjectForFile", {
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
    async dispose() {
        if (this.disposed)
            return;
        this.disposed = true;
        for (const project of this.projectMap.values()) {
            project.dispose();
        }
        this.projectMap.clear();
        this.snapshotRegistry.clear();
        this.onDispose();
        await this.client.apiRequest("release", { snapshot: this.id });
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
    async fetchSymbol(source, method, handle, projectId) {
        if (!handle)
            return undefined;
        const cached = this.getSymbol(handle);
        if (cached)
            return cached;
        const data = await this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: projectId,
            objectId: source.id,
        });
        if (!data)
            throw new Error(`${method} returned null symbol for ${source.constructor.name} ${source.id}`);
        return this.getOrCreateSymbol(data);
    }
    async fetchSymbols(source, method, handles, projectId) {
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
        const symbolData = await this.client.apiRequest(method, {
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
    async fetchType(source, method, handle) {
        if (handle !== false) {
            if (!handle)
                return undefined;
            const cached = this.getType(handle);
            if (cached)
                return cached;
        }
        const data = await this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: this.project.id,
            objectId: source.id,
        });
        if (!data)
            throw new Error(`${method} returned null type for ${source.constructor.name} ${source.id}`);
        return this.getOrCreateType(data);
    }
    async fetchSymbol(source, method, handle) {
        return this.snapshotRegistry.fetchSymbol(source, method, handle, this.project.id);
    }
    async fetchSignature(source, method, handle) {
        if (!handle)
            return undefined;
        const cached = this.getSignature(handle);
        if (cached)
            return cached;
        const data = await this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: this.project.id,
            objectId: source.id,
        });
        if (!data)
            throw new Error(`${method} returned null signature for ${source.constructor.name} ${source.id}`);
        return this.getOrCreateSignature(data);
    }
    async fetchTypes(source, method, handles) {
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
        const typesData = await this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: this.project.id,
            objectId: source.id,
        });
        if (typesData == null)
            return [];
        else
            return typesData.map(data => this.getOrCreateType(data));
    }
    async fetchSymbols(source, method, handles) {
        return this.snapshotRegistry.fetchSymbols(source, method, handles, this.project.id);
    }
    // getBaseTypes is a checker-level endpoint keyed by `type` (not `objectId`),
    // so it cannot go through fetchTypes. This helper reuses that server method.
    async fetchBaseTypes(source) {
        const typesData = await this.client.apiRequest("getBaseTypes", {
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
    async getSourceFile(file) {
        const fileName = resolveFileName(file);
        const path = this.toPath(fileName);
        // Check if we already have a retained cache entry for this (snapshot, project) pair
        const retained = this.sourceFileCache.getRetained(path, this.snapshotId, this.project.id);
        if (retained) {
            return retained;
        }
        // Fetch from server
        const binaryData = await this.client.apiRequestBinary("getSourceFile", {
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
    async getSourceFileNames() {
        const data = await this.client.apiRequest("getSourceFileNames", {
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
    async fetchSourceFileMetadata(path) {
        const data = await this.client.apiRequest("getSourceFileMetadata", {
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
    async isSourceFileFromExternalLibrary(file) {
        const metadata = await this.getSourceFileMetadataByPath(file.path);
        return metadata?.isFromExternalLibrary ?? false;
    }
    /**
     * Returns whether the given source file is a default library file (e.g. `lib.d.ts`).
     * The underlying program metadata is fetched lazily per file and cached on this
     * `Program` instance.
     */
    async isSourceFileDefaultLibrary(file) {
        const metadata = await this.getSourceFileMetadataByPath(file.path);
        return metadata?.isDefaultLibrary ?? false;
    }
    /**
     * Get syntactic (parse) diagnostics for a specific file or all files.
     * @param file - Optional file to get diagnostics for. If omitted, returns diagnostics for all files.
     */
    async getSyntacticDiagnostics(file) {
        const data = await this.client.apiRequest("getSyntacticDiagnostics", {
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
    async getBindDiagnostics(file) {
        const data = await this.client.apiRequest("getBindDiagnostics", {
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
    async getSemanticDiagnostics(file) {
        const data = await this.client.apiRequest("getSemanticDiagnostics", {
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
    async getSuggestionDiagnostics(file) {
        const data = await this.client.apiRequest("getSuggestionDiagnostics", {
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
    async getDeclarationDiagnostics(file) {
        const data = await this.client.apiRequest("getDeclarationDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
            ...(file !== undefined ? { file } : {}),
        });
        return data ?? [];
    }
    /**
     * Get program-wide diagnostics for the project, including compiler options diagnostics.
     */
    async getProgramDiagnostics() {
        const data = await this.client.apiRequest("getProgramDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
        return data ?? [];
    }
    /**
     * Get global (non-file-specific) semantic diagnostics for the project.
     */
    async getGlobalDiagnostics() {
        const data = await this.client.apiRequest("getGlobalDiagnostics", {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
        return data ?? [];
    }
    /**
     * Get config file parsing diagnostics for the project.
     */
    async getConfigFileParsingDiagnostics() {
        const data = await this.client.apiRequest("getConfigFileParsingDiagnostics", {
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
    async getSymbolAtLocation(nodeOrNodes) {
        if (Array.isArray(nodeOrNodes)) {
            const data = await this.client.apiRequest("getSymbolsAtLocations", {
                snapshot: this.snapshotId,
                project: this.project.id,
                locations: nodeOrNodes.map(node => getNodeId(node)),
            });
            return data.map(d => d ? this.objectRegistry.getOrCreateSymbol(d) : undefined);
        }
        const data = await this.client.apiRequest("getSymbolAtLocation", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(nodeOrNodes),
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    async getSymbolAtPosition(file, positionOrPositions) {
        if (typeof positionOrPositions === "number") {
            const data = await this.client.apiRequest("getSymbolAtPosition", {
                snapshot: this.snapshotId,
                project: this.project.id,
                file,
                position: positionOrPositions,
            });
            return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
        }
        const data = await this.client.apiRequest("getSymbolsAtPositions", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file,
            positions: positionOrPositions,
        });
        return data.map(d => d ? this.objectRegistry.getOrCreateSymbol(d) : undefined);
    }
    async getTypeOfSymbol(symbolOrSymbols) {
        if (Array.isArray(symbolOrSymbols)) {
            const data = await this.client.apiRequest("getTypesOfSymbols", {
                snapshot: this.snapshotId,
                project: this.project.id,
                symbols: symbolOrSymbols.map(s => s.id),
            });
            return data.map(d => d ? this.objectRegistry.getOrCreateType(d) : undefined);
        }
        const data = await this.client.apiRequest("getTypeOfSymbol", {
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
    async getDeclaredTypeOfSymbol(symbol) {
        const data = await this.client.apiRequest("getDeclaredTypeOfSymbol", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        if (!data)
            throw new Error(`getDeclaredTypeOfSymbol returned no type for symbol ${symbol.id}`);
        return this.objectRegistry.getOrCreateType(data);
    }
    async getReferencesToSymbolInFile(file, symbol) {
        const data = await this.client.apiRequest("getReferencesToSymbolInFile", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file,
            symbol: symbol.id,
        });
        return (data ?? []).map(h => new NodeHandle(h, this.project));
    }
    async getReferencedSymbolsForNode(node, position) {
        const data = await this.client.apiRequest("getReferencedSymbolsForNode", {
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
    async getSignatureUsage(signatureDecl) {
        const data = await this.client.apiRequest("getSignatureUsages", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signatureDecl: getNodeId(signatureDecl),
        });
        return (data ?? []).map(entry => ({
            name: new NodeHandle(entry.name, this.project),
            call: entry.call ? new NodeHandle(entry.call, this.project) : undefined,
        }));
    }
    async getCompletionsAtPosition(document, position, options) {
        const data = await this.client.apiRequest("getCompletionsAtPosition", {
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
    async getTypeAtLocation(nodeOrNodes) {
        if (Array.isArray(nodeOrNodes)) {
            const data = await this.client.apiRequest("getTypeAtLocations", {
                snapshot: this.snapshotId,
                project: this.project.id,
                locations: nodeOrNodes.map(node => getNodeId(node)),
            });
            return data.map(d => d ? this.objectRegistry.getOrCreateType(d) : undefined);
        }
        const data = await this.client.apiRequest("getTypeAtLocation", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(nodeOrNodes),
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getSignaturesOfType(type, kind) {
        const data = await this.client.apiRequest("getSignaturesOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
            kind,
        });
        return data.map(d => this.objectRegistry.getOrCreateSignature(d));
    }
    async getResolvedSignature(node) {
        const data = await this.client.apiRequest("getResolvedSignature", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateSignature(data) : undefined;
    }
    async getTypeAtPosition(file, positionOrPositions) {
        if (typeof positionOrPositions === "number") {
            const data = await this.client.apiRequest("getTypeAtPosition", {
                snapshot: this.snapshotId,
                project: this.project.id,
                file,
                position: positionOrPositions,
            });
            return data ? this.objectRegistry.getOrCreateType(data) : undefined;
        }
        const data = await this.client.apiRequest("getTypesAtPositions", {
            snapshot: this.snapshotId,
            project: this.project.id,
            file,
            positions: positionOrPositions,
        });
        return data.map(d => d ? this.objectRegistry.getOrCreateType(d) : undefined);
    }
    async resolveName(name, meaning, location, excludeGlobals) {
        // Distinguish Node (has `kind`) from DocumentPosition (has `document` and `position`)
        const isNode = location && "kind" in location;
        const data = await this.client.apiRequest("resolveName", {
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
    async getResolvedSymbol(node) {
        const text = node.text;
        if (!text)
            return undefined;
        return this.resolveName(text, SymbolFlags.Value | SymbolFlags.ExportValue, node);
    }
    async getContextualType(node) {
        const data = await this.client.apiRequest("getContextualType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getBaseTypeOfLiteralType(type) {
        const data = await this.client.apiRequest("getBaseTypeOfLiteralType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getNonNullableType(type) {
        const data = await this.client.apiRequest("getNonNullableType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getTypeFromTypeNode(node) {
        const data = await this.client.apiRequest("getTypeFromTypeNode", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getWidenedType(type) {
        const data = await this.client.apiRequest("getWidenedType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getParameterType(signature, index) {
        const data = await this.client.apiRequest("getParameterType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signature: signature.id,
            index,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async isArrayLikeType(type) {
        return this.client.apiRequest("isArrayLikeType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
    }
    async isTypeAssignableTo(source, target) {
        return this.client.apiRequest("isTypeAssignableTo", {
            snapshot: this.snapshotId,
            project: this.project.id,
            source: source.id,
            target: target.id,
        });
    }
    async getShorthandAssignmentValueSymbol(node) {
        const data = await this.client.apiRequest("getShorthandAssignmentValueSymbol", {
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
    async getTypeOfSymbolAtLocation(symbol, location) {
        const data = await this.client.apiRequest("getTypeOfSymbolAtLocation", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
            location: getNodeId(location),
        });
        if (!data)
            throw new Error(`getTypeOfSymbolAtLocation returned no type for symbol ${symbol.id}`);
        return this.objectRegistry.getOrCreateType(data);
    }
    async getIntrinsicType(method) {
        const data = await this.client.apiRequest(method, {
            snapshot: this.snapshotId,
            project: this.project.id,
        });
        return this.objectRegistry.getOrCreateType(data);
    }
    async getAnyType() {
        return this.getIntrinsicType("getAnyType");
    }
    async getStringType() {
        return this.getIntrinsicType("getStringType");
    }
    async getNumberType() {
        return this.getIntrinsicType("getNumberType");
    }
    async getBooleanType() {
        return this.getIntrinsicType("getBooleanType");
    }
    async getVoidType() {
        return this.getIntrinsicType("getVoidType");
    }
    async getUndefinedType() {
        return this.getIntrinsicType("getUndefinedType");
    }
    async getNullType() {
        return this.getIntrinsicType("getNullType");
    }
    async getNeverType() {
        return this.getIntrinsicType("getNeverType");
    }
    async getUnknownType() {
        return this.getIntrinsicType("getUnknownType");
    }
    async getBigIntType() {
        return this.getIntrinsicType("getBigIntType");
    }
    async getESSymbolType() {
        return this.getIntrinsicType("getESSymbolType");
    }
    async typeToTypeNode(type, enclosingDeclaration, flags) {
        const binaryData = await this.client.apiRequestBinary("typeToTypeNode", {
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
    async signatureToSignatureDeclaration(signature, kind, enclosingDeclaration, flags) {
        const binaryData = await this.client.apiRequestBinary("signatureToSignatureDeclaration", {
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
    async typeToString(type, enclosingDeclaration, flags) {
        return this.client.apiRequest("typeToString", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
            location: enclosingDeclaration ? getNodeId(enclosingDeclaration) : undefined,
            flags,
        });
    }
    async isContextSensitive(node) {
        return this.client.apiRequest("isContextSensitive", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
    }
    async isArrayType(type) {
        return this.client.apiRequest("isArrayType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
    }
    async isTupleType(type) {
        return this.client.apiRequest("isTupleType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
    }
    async getReturnTypeOfSignature(signature) {
        const data = await this.client.apiRequest("getReturnTypeOfSignature", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signature: signature.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getRestTypeOfSignature(signature) {
        const data = await this.client.apiRequest("getRestTypeOfSignature", {
            snapshot: this.snapshotId,
            project: this.project.id,
            signature: signature.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getTypePredicateOfSignature(signature) {
        const data = await this.client.apiRequest("getTypePredicateOfSignature", {
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
    async getBaseTypes(type) {
        const data = await this.client.apiRequest("getBaseTypes", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? data.map(d => this.objectRegistry.getOrCreateType(d)) : [];
    }
    async getApparentType(type) {
        const data = await this.client.apiRequest("getApparentType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getPropertiesOfType(type) {
        const data = await this.client.apiRequest("getPropertiesOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? data.map(d => this.objectRegistry.getOrCreateSymbol(d)) : [];
    }
    async getIndexInfosOfType(type) {
        const data = await this.client.apiRequest("getIndexInfosOfType", {
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
    async getConstraintOfTypeParameter(type) {
        const data = await this.client.apiRequest("getConstraintOfTypeParameter", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getBaseConstraintOfType(type) {
        const data = await this.client.apiRequest("getBaseConstraintOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
        });
        return data ? this.objectRegistry.getOrCreateType(data) : undefined;
    }
    async getPropertyOfType(type, name) {
        const data = await this.client.apiRequest("getPropertyOfType", {
            snapshot: this.snapshotId,
            project: this.project.id,
            type: type.id,
            name,
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    async getConstantValue(node) {
        const data = await this.client.apiRequest("getConstantValue", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ?? undefined;
    }
    async getSignatureFromDeclaration(node) {
        const data = await this.client.apiRequest("getSignatureFromDeclaration", {
            snapshot: this.snapshotId,
            project: this.project.id,
            location: getNodeId(node),
        });
        return data ? this.objectRegistry.getOrCreateSignature(data) : undefined;
    }
    async getExportSpecifierLocalTargetSymbol(node) {
        const data = await this.client.apiRequest("getExportSpecifierLocalTargetSymbol", {
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
    async getAliasedSymbol(symbol) {
        const data = await this.client.apiRequest("getAliasedSymbol", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        if (!data)
            throw new Error(`getAliasedSymbol returned no symbol for symbol ${symbol.id}`);
        return this.objectRegistry.getOrCreateSymbol(data);
    }
    async getImmediateAliasedSymbol(symbol) {
        const data = await this.client.apiRequest("getImmediateAliasedSymbol", {
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
    async isUnknownSymbol(symbol) {
        return symbol.id === (await this.getWellKnownSymbols()).unknown;
    }
    /**
     * Returns `true` if the symbol is the checker's "undefined" symbol.
     */
    async isUndefinedSymbol(symbol) {
        return symbol.id === (await this.getWellKnownSymbols()).undefined;
    }
    /**
     * Returns `true` if the symbol is the checker's "arguments" symbol.
     */
    async isArgumentsSymbol(symbol) {
        return symbol.id === (await this.getWellKnownSymbols()).arguments;
    }
    async getExportsOfModule(symbol) {
        const data = await this.client.apiRequest("getExportsOfModule", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        return data ? data.map(d => this.objectRegistry.getOrCreateSymbol(d)) : [];
    }
    async getMemberInModuleExports(symbol, name) {
        const data = await this.client.apiRequest("getMemberInModuleExports", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
            name,
        });
        return data ? this.objectRegistry.getOrCreateSymbol(data) : undefined;
    }
    async getJsDocTagsOfSymbol(symbol) {
        const data = await this.client.apiRequest("getJsDocTags", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
        return data ?? [];
    }
    async getDocumentationCommentOfSymbol(symbol) {
        return this.client.apiRequest("getDocumentationComment", {
            snapshot: this.snapshotId,
            project: this.project.id,
            symbol: symbol.id,
        });
    }
    /**
     * Get the type arguments of a type reference (e.g. the `string` in `Array<string>`).
     */
    async getTypeArguments(type) {
        const data = await this.client.apiRequest("getTypeArguments", {
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
    async printNode(node, options = {}) {
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
    async resolve(project = this.canonicalProject) {
        const sourceFile = await project.program.getSourceFile(this.path);
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
    async getParent() {
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
    async fetchSymbolTable(method) {
        const symbols = await this.objectRegistry.fetchSymbols(this, method, undefined, this.canonicalProject.id);
        const table = new Map();
        for (const symbol of symbols) {
            table.set(symbol.escapedName, symbol);
        }
        return table;
    }
    async getExportSymbol() {
        if (!this.exportSymbol)
            return this;
        return this.objectRegistry.fetchSymbol(this, "getExportSymbolOfSymbol", this.exportSymbol, this.canonicalProject.id);
    }
    async getJsDocTags(checker) {
        return checker.getJsDocTagsOfSymbol(this);
    }
    async getDocumentationComment(checker) {
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
    async getSymbol() {
        return this.objectRegistry.fetchSymbol(this, "getSymbolOfType", this.symbol);
    }
    async getAliasSymbol() {
        return this.objectRegistry.fetchSymbol(this, "getAliasSymbolOfType", this.aliasSymbol);
    }
    async getTarget() {
        return this.objectRegistry.fetchType(this, "getTargetOfType", this.target);
    }
    async getFreshType() {
        return this.objectRegistry.fetchType(this, "getFreshTypeOfType", this.freshType);
    }
    async getRegularType() {
        return this.objectRegistry.fetchType(this, "getRegularTypeOfType", this.regularType);
    }
    async getTypes() {
        // Only union, intersection, and template literal types have constituent
        // types; any other kind has none, so return undefined rather than sending
        // a request the server cannot satisfy.
        if (!(this.flags & (TypeFlags.UnionOrIntersection | TypeFlags.TemplateLiteral))) {
            return undefined;
        }
        return this.objectRegistry.fetchTypes(this, "getTypesOfType");
    }
    async getTypeParameters() {
        return this.objectRegistry.fetchTypes(this, "getTypeParametersOfType", this.typeParameters);
    }
    async getOuterTypeParameters() {
        return this.objectRegistry.fetchTypes(this, "getOuterTypeParametersOfType", this.outerTypeParameters);
    }
    async getLocalTypeParameters() {
        return this.objectRegistry.fetchTypes(this, "getLocalTypeParametersOfType", this.localTypeParameters);
    }
    async getAliasTypeArguments() {
        return this.objectRegistry.fetchTypes(this, "getAliasTypeArgumentsOfType", this.aliasTypeArguments);
    }
    async getObjectType() {
        return this.objectRegistry.fetchType(this, "getObjectTypeOfType", this.objectType);
    }
    async getIndexType() {
        return this.objectRegistry.fetchType(this, "getIndexTypeOfType", this.indexType);
    }
    async getCheckType() {
        return this.objectRegistry.fetchType(this, "getCheckTypeOfType", this.checkType);
    }
    async getExtendsType() {
        return this.objectRegistry.fetchType(this, "getExtendsTypeOfType", this.extendsType);
    }
    async getBaseType() {
        return this.objectRegistry.fetchType(this, "getBaseTypeOfType", this.baseType);
    }
    async getConstraint() {
        return this.objectRegistry.fetchType(this, "getConstraintOfType", this.substConstraint);
    }
    async getTrueType() {
        const result = await this.objectRegistry.fetchType(this, "getTrueTypeOfConditionalType", this.trueType);
        this.trueType = result.id;
        return result;
    }
    async getFalseType() {
        const result = await this.objectRegistry.fetchType(this, "getFalseTypeOfConditionalType", this.falseType);
        this.falseType = result.id;
        return result;
    }
    /**
     * Get the base types of this type. Returns `undefined` for any type that is
     * not a class or interface.
     */
    async getBaseTypes() {
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
    async getTypeParameters() {
        return this.objectRegistry.fetchTypes(this, "getTypeParametersOfSignature", this.typeParameters);
    }
    async getParameters() {
        return this.objectRegistry.fetchSymbols(this, "getParametersOfSignature", this.parameters);
    }
    async getThisParameter() {
        return this.objectRegistry.fetchSymbol(this, "getThisParameterOfSignature", this.thisParameter);
    }
    async getTarget() {
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