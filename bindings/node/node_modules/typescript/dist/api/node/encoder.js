import { SyntaxKind } from "../../ast/index.js";
import { getNodeCommonData, getNodeDataType, } from "./encoder.generated.js";
import { MsgpackWriter } from "./msgpack.js";
import { childProperties, HEADER_OFFSET_EXTENDED_DATA, HEADER_OFFSET_METADATA, HEADER_OFFSET_NODES, HEADER_OFFSET_STRING_TABLE, HEADER_OFFSET_STRING_TABLE_OFFSETS, HEADER_OFFSET_STRUCTURED_DATA, HEADER_SIZE, KIND_NODE_LIST, NODE_DATA_TYPE_CHILDREN, NODE_DATA_TYPE_EXTENDED, NODE_DATA_TYPE_STRING, NODE_LEN, PROTOCOL_VERSION, } from "./protocol.js";
const NODE_FIELDS = NODE_LEN / 4;
const NODE_FIELD_NEXT = 3;
const NO_STRUCTURED_DATA = 0xFFFFFFFF;
// String table that accumulates strings into a flat byte pool.
class StringTable {
    parts;
    byteLen;
    offsets;
    constructor() {
        this.parts = [];
        this.byteLen = 0;
        this.offsets = [];
    }
    add(text) {
        const index = this.offsets.length;
        const encoder = cachedEncoder();
        const encodedLength = encoder.encode(text).length;
        const offset = this.byteLen;
        this.parts.push(text);
        this.byteLen += encodedLength;
        this.offsets.push(offset, offset + encodedLength);
        return index;
    }
    encode() {
        const encoder = cachedEncoder();
        const dataBytes = encoder.encode(this.parts.join(""));
        const offsetBytes = new Uint8Array(this.offsets.length * 4);
        const view = new DataView(offsetBytes.buffer);
        for (let i = 0; i < this.offsets.length; i++) {
            view.setUint32(i * 4, this.offsets[i], true);
        }
        const result = new Uint8Array(offsetBytes.length + dataBytes.length);
        result.set(offsetBytes, 0);
        result.set(dataBytes, offsetBytes.length);
        return result;
    }
    stringByteLength() {
        return this.byteLen;
    }
    offsetsCount() {
        return this.offsets.length;
    }
}
let _encoder;
function cachedEncoder() {
    return _encoder ??= new TextEncoder();
}
function getChildrenPropertyMask(node) {
    const kind = node.kind;
    const props = childProperties[kind];
    if (!props) {
        return 0;
    }
    const n = node;
    let mask = 0;
    for (let i = 0; i < props.length; i++) {
        const prop = props[i];
        if (prop !== undefined && isChildPresent(n[prop])) {
            mask |= 1 << i;
        }
    }
    return mask;
}
// A child is "present" if it's non-null/non-undefined.
// This matches the Go encoder's behavior where non-nil NodeLists (even empty)
// are treated as present, and only nil NodeLists are absent.
function isChildPresent(v) {
    if (v === undefined || v === null)
        return false;
    return true;
}
function recordNodeStrings(node, strs) {
    return strs.add(node.text ?? "");
}
function encodeFileReferences(refs, writer) {
    if (!refs || refs.length === 0)
        return NO_STRUCTURED_DATA;
    const offset = writer.finish().length;
    writer.writeArrayHeader(refs.length);
    for (const ref of refs) {
        writer.writeArrayHeader(5);
        writer.writeUint(ref.pos);
        writer.writeUint(ref.end);
        writer.writeString(ref.fileName);
        writer.writeUint(ref.resolutionMode ?? 0);
        writer.writeBool(ref.preserve ?? false);
    }
    return offset;
}
function recordExtendedData(node, strs, extendedData, structuredWriter) {
    const offset = extendedData.length * 4;
    if (node.kind === SyntaxKind.SourceFile) {
        const sf = node;
        const textIndex = strs.add(sf.text);
        const fileNameIndex = strs.add(sf.fileName);
        const pathIndex = strs.add(sf.path);
        const referencedFilesOffset = encodeFileReferences(sf.referencedFiles, structuredWriter);
        const typeRefDirectivesOffset = encodeFileReferences(sf.typeReferenceDirectives, structuredWriter);
        const libRefDirectivesOffset = encodeFileReferences(sf.libReferenceDirectives, structuredWriter);
        extendedData.push(textIndex, fileNameIndex, pathIndex, sf.languageVariant, sf.scriptKind, referencedFilesOffset, typeRefDirectivesOffset, libRefDirectivesOffset, NO_STRUCTURED_DATA, NO_STRUCTURED_DATA, NO_STRUCTURED_DATA, 0);
    }
    else if (node.kind === SyntaxKind.TemplateHead ||
        node.kind === SyntaxKind.TemplateMiddle ||
        node.kind === SyntaxKind.TemplateTail) {
        const tmpl = node;
        const text = tmpl.text ?? "";
        const rawText = tmpl.rawText ?? "";
        const templateFlags = tmpl.templateFlags ?? 0;
        const textIndex = strs.add(text);
        const rawTextIndex = strs.add(rawText);
        extendedData.push(textIndex, rawTextIndex, templateFlags);
    }
    else {
        // StringLiteral, NumericLiteral, BigIntLiteral, RegularExpressionLiteral,
        // NoSubstitutionTemplateLiteral — format: [textIndex, tokenFlags]
        const n = node;
        const text = n.text ?? "";
        const tokenFlags = n.tokenFlags ?? 0;
        const textIndex = strs.add(text);
        extendedData.push(textIndex, tokenFlags);
    }
    return offset;
}
function getNodeData(node, strs, extendedData, structuredWriter) {
    const t = getNodeDataType(node.kind);
    const common = getNodeCommonData(node);
    switch (t) {
        case NODE_DATA_TYPE_CHILDREN:
            return t | common | getChildrenPropertyMask(node);
        case NODE_DATA_TYPE_STRING:
            return t | common | recordNodeStrings(node, strs);
        case NODE_DATA_TYPE_EXTENDED:
            return t | common | recordExtendedData(node, strs, extendedData, structuredWriter);
        default:
            throw new Error("unreachable");
    }
}
function getChildPropertiesForNode(node) {
    return childProperties[node.kind];
}
// Returns whether a value is a NodeArray (array-like with pos and end).
function isNodeArray(value) {
    return Array.isArray(value) && typeof value.pos === "number" && typeof value.end === "number";
}
/**
 * Encode a SourceFile AST node into the binary format.
 */
export function encodeSourceFile(sourceFile) {
    return encodeNode(sourceFile);
}
/**
 * Encode an arbitrary AST node into the binary format.
 * When encoding a non-SourceFile node, the header hash and parse options fields will be zero.
 */
export function encodeNode(node) {
    const strs = new StringTable();
    const extendedDataValues = [];
    const structuredWriter = new MsgpackWriter();
    // We'll build an array of uint32 values for the nodes section, 7 per node
    const nodeValues = [];
    // Nil node (index 0)
    nodeValues.push(0, 0, 0, 0, 0, 0, 0);
    let nodeCount = 0;
    let parentIndex = 0;
    let prevIndex = 0;
    function visitNode(node) {
        nodeCount++;
        const currentIndex = nodeCount;
        if (prevIndex !== 0) {
            // Set next pointer on previous sibling
            nodeValues[prevIndex * NODE_FIELDS + NODE_FIELD_NEXT] = currentIndex;
        }
        const data = getNodeData(node, strs, extendedDataValues, structuredWriter);
        nodeValues.push(node.kind, node.pos >= 0 ? node.pos : 0, node.end >= 0 ? node.end : 0, 0, // next (filled in later)
        parentIndex, data, node.flags);
        const saveParentIndex = parentIndex;
        const savePrevIndex = prevIndex;
        parentIndex = currentIndex;
        prevIndex = 0;
        visitChildren(node);
        prevIndex = currentIndex;
        parentIndex = saveParentIndex;
    }
    function visitNodeList(list) {
        if (!list) {
            return;
        }
        nodeCount++;
        const currentIndex = nodeCount;
        if (prevIndex !== 0) {
            nodeValues[prevIndex * NODE_FIELDS + NODE_FIELD_NEXT] = currentIndex;
        }
        nodeValues.push(KIND_NODE_LIST, list.pos >= 0 ? list.pos : 0, list.end >= 0 ? list.end : 0, 0, // next
        parentIndex, list.length, // data for NodeList is its length
        0);
        const saveParentIndex = parentIndex;
        parentIndex = currentIndex;
        prevIndex = 0;
        for (const child of list) {
            visitNode(child);
        }
        prevIndex = currentIndex;
        parentIndex = saveParentIndex;
    }
    function visitChildren(node) {
        const props = getChildPropertiesForNode(node);
        const n = node;
        if (props) {
            for (const propName of props) {
                if (propName === undefined)
                    continue;
                const child = n[propName];
                if (child === undefined || child === null)
                    continue;
                if (isNodeArray(child)) {
                    visitNodeList(child);
                }
                else {
                    visitNode(child);
                }
            }
        }
    }
    // Encode root node
    nodeCount++;
    parentIndex++;
    const rootData = getNodeData(node, strs, extendedDataValues, structuredWriter);
    nodeValues.push(node.kind, node.pos >= 0 ? node.pos : 0, node.end >= 0 ? node.end : 0, 0, 0, rootData, node.flags);
    const saveParent = parentIndex;
    prevIndex = 0;
    parentIndex = 1; // root is at index 1
    visitChildren(node);
    parentIndex = saveParent;
    // Encode extended data section
    const extendedDataBytes = new Uint8Array(extendedDataValues.length * 4);
    const extView = new DataView(extendedDataBytes.buffer);
    for (let i = 0; i < extendedDataValues.length; i++) {
        extView.setUint32(i * 4, extendedDataValues[i], true);
    }
    // Encode structured data section
    const structuredDataBytes = structuredWriter.finish();
    // Encode string table
    const strsBytes = strs.encode();
    // Encode nodes section
    const nodesBytes = new Uint8Array(nodeValues.length * 4);
    const nodesView = new DataView(nodesBytes.buffer);
    for (let i = 0; i < nodeValues.length; i++) {
        nodesView.setUint32(i * 4, nodeValues[i] >>> 0, true);
    }
    // Calculate section offsets
    const offsetStringTableOffsets = HEADER_SIZE;
    const offsetStringTableData = HEADER_SIZE + strs.offsetsCount() * 4;
    const offsetExtendedData = offsetStringTableData + strs.stringByteLength();
    const offsetStructuredData = offsetExtendedData + extendedDataBytes.length;
    const offsetNodes = offsetStructuredData + structuredDataBytes.length;
    // Build header
    const header = new Uint8Array(HEADER_SIZE);
    const headerView = new DataView(header.buffer);
    const metadata = PROTOCOL_VERSION << 24;
    headerView.setUint32(HEADER_OFFSET_METADATA, metadata, true);
    // bytes 4-19: hash (zero for non-SourceFile, we don't have access to xxh3 here)
    // byte 20-23: parse options (zero for non-SourceFile)
    headerView.setUint32(HEADER_OFFSET_STRING_TABLE_OFFSETS, offsetStringTableOffsets, true);
    headerView.setUint32(HEADER_OFFSET_STRING_TABLE, offsetStringTableData, true);
    headerView.setUint32(HEADER_OFFSET_EXTENDED_DATA, offsetExtendedData, true);
    headerView.setUint32(HEADER_OFFSET_STRUCTURED_DATA, offsetStructuredData, true);
    headerView.setUint32(HEADER_OFFSET_NODES, offsetNodes, true);
    // Concatenate all sections
    const result = new Uint8Array(header.length + strsBytes.length + extendedDataBytes.length + structuredDataBytes.length + nodesBytes.length);
    result.set(header, 0);
    result.set(strsBytes, HEADER_SIZE);
    result.set(extendedDataBytes, offsetExtendedData);
    result.set(structuredDataBytes, offsetStructuredData);
    result.set(nodesBytes, offsetNodes);
    return result;
}
/**
 * Encode a Uint8Array to a base64 string.
 */
export function uint8ArrayToBase64(data) {
    return Buffer.from(data).toString("base64");
}
//# sourceMappingURL=encoder.js.map