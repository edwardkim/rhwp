import { computeLineStarts, NodeFlags, SyntaxKind, TokenFlags, } from "../../ast/index.js";
import { MsgpackReader } from "./msgpack.js";
import { RemoteNode, RemoteNodeList, } from "./node.generated.js";
import { NODE_EXTENDED_DATA_MASK, } from "./node.infrastructure.js";
import { HEADER_OFFSET_EXTENDED_DATA, HEADER_OFFSET_NODES, HEADER_OFFSET_STRING_TABLE, HEADER_OFFSET_STRING_TABLE_OFFSETS, HEADER_OFFSET_STRUCTURED_DATA, KIND_NODE_LIST, NODE_LEN, NODE_OFFSET_KIND, NODE_OFFSET_PARENT, } from "./protocol.js";
import { Wtf8Decoder } from "./wtf8.js";
// Re-export everything consumers need from the other two files.
export { RemoteNode, RemoteNodeList } from "./node.generated.js";
export { readParseOptionsKey, readSourceFileHash, RemoteNodeBase } from "./node.infrastructure.js";
// ═══════════════════════════════════════════════════════════════════════════
// RemoteSourceFile
// ═══════════════════════════════════════════════════════════════════════════
const NO_STRUCTURED_DATA = 0xFFFFFFFF;
export class RemoteSourceFile extends RemoteNode {
    nodes;
    _offsetNodes;
    _offsetStringTableOffsets;
    _offsetStringTable;
    _offsetExtendedData;
    _offsetStructuredData;
    _decoder;
    _timing;
    _lineStarts;
    _cachedText;
    _cachedReferencedFiles;
    _cachedTypeReferenceDirectives;
    _cachedLibReferenceDirectives;
    _cachedImports;
    _cachedModuleAugmentations;
    _cachedAmbientModuleNames;
    constructor(data, decoder, timing) {
        const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
        const offsetNodes = view.getUint32(HEADER_OFFSET_NODES, true);
        super(view, 1, undefined, undefined, offsetNodes);
        this._sourceFile = this;
        this._offsetNodes = offsetNodes;
        this._offsetStringTableOffsets = view.getUint32(HEADER_OFFSET_STRING_TABLE_OFFSETS, true);
        this._offsetStringTable = view.getUint32(HEADER_OFFSET_STRING_TABLE, true);
        this._offsetExtendedData = view.getUint32(HEADER_OFFSET_EXTENDED_DATA, true);
        this._offsetStructuredData = view.getUint32(HEADER_OFFSET_STRUCTURED_DATA, true);
        this._decoder = decoder;
        this._timing = timing;
        this.nodes = Array((view.byteLength - offsetNodes) / NODE_LEN);
        this.nodes[1] = this;
        // Every node slot is materializable on demand except the nil sentinel at
        // index 0 and the source-file node at index 1, which is pre-materialized.
        timing?.recordSourceFileFetched(Math.max(0, this.nodes.length - 2));
    }
    readFileReferences(structuredDataOffset) {
        if (structuredDataOffset === NO_STRUCTURED_DATA) {
            return [];
        }
        const buf = new Uint8Array(this.view.buffer, this.view.byteOffset, this.view.byteLength);
        const reader = new MsgpackReader(buf, this._offsetStructuredData + structuredDataOffset);
        const count = reader.readArrayHeader();
        const result = [];
        for (let i = 0; i < count; i++) {
            reader.readArrayHeader(); // 5-element tuple
            const pos = reader.readUint();
            const end = reader.readUint();
            const fileName = reader.readString();
            const resolutionMode = reader.readUint();
            const preserve = reader.readBool();
            result.push({ pos, end, fileName, resolutionMode, preserve });
        }
        return result;
    }
    readNodeIndexArray(structuredDataOffset) {
        if (structuredDataOffset === NO_STRUCTURED_DATA) {
            return [];
        }
        const buf = new Uint8Array(this.view.buffer, this.view.byteOffset, this.view.byteLength);
        const reader = new MsgpackReader(buf, this._offsetStructuredData + structuredDataOffset);
        const count = reader.readArrayHeader();
        const result = [];
        for (let i = 0; i < count; i++) {
            const nodeIndex = reader.readUint();
            result.push(this.getOrCreateNodeAtIndex(nodeIndex));
        }
        return result;
    }
    readStringArray(structuredDataOffset) {
        if (structuredDataOffset === NO_STRUCTURED_DATA) {
            return [];
        }
        const buf = new Uint8Array(this.view.buffer, this.view.byteOffset, this.view.byteLength);
        const reader = new MsgpackReader(buf, this._offsetStructuredData + structuredDataOffset);
        const count = reader.readArrayHeader();
        const result = [];
        for (let i = 0; i < count; i++) {
            result.push(reader.readString());
        }
        return result;
    }
    /** @internal */
    getOrCreateNodeAtIndex(index) {
        let node = this.nodes[index];
        if (!node) {
            // Resolve the real parent so that nodes looked up directly by index (e.g. via
            // NodeHandle.resolve) report the correct `parent`, rather than always pointing at
            // the source file. The stored parent index can refer to a synthetic NodeList
            // container; skip those to mirror normal traversal, where list elements take the
            // list's parent. The walk terminates at the source file, which occupies index 1
            // and is already cached.
            let parentIndex = this.view.getUint32(this._offsetNodes + index * NODE_LEN + NODE_OFFSET_PARENT, true);
            while (parentIndex !== index &&
                this.view.getUint32(this._offsetNodes + parentIndex * NODE_LEN + NODE_OFFSET_KIND, true) === KIND_NODE_LIST) {
                parentIndex = this.view.getUint32(this._offsetNodes + parentIndex * NODE_LEN + NODE_OFFSET_PARENT, true);
            }
            const parent = parentIndex === index ? this : this.getOrCreateNodeAtIndex(parentIndex);
            node = new RemoteNode(this.view, index, parent, this, this._offsetNodes);
            this.nodes[index] = node;
            this._timing?.recordMaterialization();
        }
        return node;
    }
    // ═══ SourceFile-specific extended data getters ═══
    get extendedDataOffset() {
        return this._offsetExtendedData + (this.data & NODE_EXTENDED_DATA_MASK);
    }
    get fileName() {
        const stringIndex = this.view.getUint32(this.extendedDataOffset + 4, true);
        return this.getString(stringIndex);
    }
    get path() {
        const stringIndex = this.view.getUint32(this.extendedDataOffset + 8, true);
        return this.getString(stringIndex);
    }
    get languageVariant() {
        return this.view.getUint32(this.extendedDataOffset + 12, true);
    }
    get scriptKind() {
        return this.view.getUint32(this.extendedDataOffset + 16, true);
    }
    get referencedFiles() {
        if (this._cachedReferencedFiles !== undefined)
            return this._cachedReferencedFiles;
        const offset = this.view.getUint32(this.extendedDataOffset + 20, true);
        const files = this.readFileReferences(offset);
        this._cachedReferencedFiles = files;
        return files;
    }
    get typeReferenceDirectives() {
        if (this._cachedTypeReferenceDirectives !== undefined)
            return this._cachedTypeReferenceDirectives;
        const offset = this.view.getUint32(this.extendedDataOffset + 24, true);
        const directives = this.readFileReferences(offset);
        this._cachedTypeReferenceDirectives = directives;
        return directives;
    }
    get libReferenceDirectives() {
        if (this._cachedLibReferenceDirectives !== undefined)
            return this._cachedLibReferenceDirectives;
        const offset = this.view.getUint32(this.extendedDataOffset + 28, true);
        const directives = this.readFileReferences(offset);
        this._cachedLibReferenceDirectives = directives;
        return directives;
    }
    get imports() {
        if (this._cachedImports !== undefined)
            return this._cachedImports;
        const offset = this.view.getUint32(this.extendedDataOffset + 32, true);
        const imports = this.readNodeIndexArray(offset);
        this._cachedImports = imports;
        return imports;
    }
    get moduleAugmentations() {
        if (this._cachedModuleAugmentations !== undefined)
            return this._cachedModuleAugmentations;
        const offset = this.view.getUint32(this.extendedDataOffset + 36, true);
        const moduleAugmentations = this.readNodeIndexArray(offset);
        this._cachedModuleAugmentations = moduleAugmentations;
        return moduleAugmentations;
    }
    get ambientModuleNames() {
        if (this._cachedAmbientModuleNames !== undefined)
            return this._cachedAmbientModuleNames;
        const offset = this.view.getUint32(this.extendedDataOffset + 40, true);
        const names = this.readStringArray(offset);
        this._cachedAmbientModuleNames = names;
        return names;
    }
    get externalModuleIndicator() {
        const nodeIndex = this.view.getUint32(this.extendedDataOffset + 44, true);
        if (nodeIndex === 0)
            return undefined;
        if (nodeIndex === this.index)
            return true;
        return this.getOrCreateNodeAtIndex(nodeIndex);
    }
    get isDeclarationFile() {
        return (this.flags & NodeFlags.Ambient) !== 0;
    }
    get text() {
        if (this._cachedText !== undefined)
            return this._cachedText;
        const text = super.text;
        this._cachedText = text;
        return text;
    }
    // ═══ Line/character position mapping ═══
    getLineStarts() {
        return this._lineStarts ??= computeLineStarts(this.text ?? "");
    }
    getLineAndCharacterOfPosition(position) {
        const lineStarts = this.getLineStarts();
        const line = computeLineOfPosition(lineStarts, position);
        return { line, character: position - lineStarts[line] };
    }
    getPositionOfLineAndCharacter(line, character) {
        const lineStarts = this.getLineStarts();
        if (line < 0 || line >= lineStarts.length) {
            throw new Error(`Bad line number. Line: ${line}, lineStarts.length: ${lineStarts.length}`);
        }
        return lineStarts[line] + character;
    }
}
/**
 * Find the 0-based line number containing the given position via binary search.
 * Assumes the first line starts at position 0 and `position` is non-negative.
 */
function computeLineOfPosition(lineStarts, position) {
    let low = 0;
    let high = lineStarts.length - 1;
    while (low <= high) {
        const middle = low + ((high - low) >> 1);
        const value = lineStarts[middle];
        if (value < position) {
            low = middle + 1;
        }
        else if (value > position) {
            high = middle - 1;
        }
        else {
            return middle;
        }
    }
    return low - 1;
}
/**
 * Find a descendant node at a specific position with matching kind and end position.
 */
export function findDescendant(root, pos, end, kind) {
    if (root.pos === pos && root.end === end && root.kind === kind) {
        return root;
    }
    // Search children
    let result;
    root.forEachChild(child => {
        if (result)
            return result; // Already found
        // Only search in children that could contain our target
        if (child.pos <= pos && child.end >= end) {
            result = findDescendant(child, pos, end, kind);
        }
        return undefined;
    });
    return result;
}
/**
 * Parse a node handle string into its components.
 * Handle format: "index.kind.path" where path may contain dots.
 */
export function parseNodeHandle(handle) {
    const firstDot = handle.indexOf(".");
    if (firstDot === -1) {
        throw new Error(`Invalid node handle: ${handle}`);
    }
    const secondDot = handle.indexOf(".", firstDot + 1);
    if (secondDot === -1) {
        throw new Error(`Invalid node handle: ${handle}`);
    }
    return {
        index: parseInt(handle.slice(0, firstDot), 10),
        kind: parseInt(handle.slice(firstDot + 1, secondDot), 10),
        path: handle.slice(secondDot + 1),
    };
}
/**
 * Decode binary-encoded AST data into a Node.
 * Works for any binary-encoded node, including synthetic nodes
 * (e.g. from typeToTypeNode) that don't have a source file.
 */
export function decodeNode(data) {
    const sf = new RemoteSourceFile(data, new Wtf8Decoder());
    return sf;
}
/**
 * Get the unique ID string for a remote node.
 * Throws if the node is not a RemoteNode (i.e. not decoded from binary data).
 */
export function getNodeId(node) {
    if (!(node instanceof RemoteNode)) {
        throw new Error("getNodeId requires a RemoteNode");
    }
    return node.id;
}
//# sourceMappingURL=node.js.map