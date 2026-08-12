import { NodeFlags } from "#enums/nodeFlags";
import { SyntaxKind } from "#enums/syntaxKind";
import { createToken } from "./factory.generated.js";
import { isJSDocNodeKind, isKeywordKind, isPrivateIdentifier, isPropertyNameLiteral, isTokenKind, } from "./is.js";
import { createScanner, skipTrivia, } from "./scanner.js";
export function getTokenAtPosition(sourceFile, position) {
    return getTokenAtPositionImpl(sourceFile, position, /*allowPositionInLeadingTrivia*/ true, /*includePrecedingTokenAtEndPosition*/ undefined);
}
export function getTouchingPropertyName(sourceFile, position) {
    return getTokenAtPositionImpl(sourceFile, position, /*allowPositionInLeadingTrivia*/ false, node => isPropertyNameLiteral(node) || isKeywordKind(node.kind) || isPrivateIdentifier(node));
}
export function getTouchingToken(sourceFile, position) {
    return getTokenAtPositionImpl(sourceFile, position, /*allowPositionInLeadingTrivia*/ false, /*includePrecedingTokenAtEndPosition*/ undefined);
}
/**
 * Finds the token that starts immediately after `previousToken` ends, searching
 * within `parent`.  Returns `undefined` if no such token exists.
 */
export function findNextToken(previousToken, parent, sourceFile) {
    return find(parent);
    function find(n) {
        if (isTokenKind(n.kind) && n.pos === previousToken.end) {
            // This is the token that starts at the end of previousToken – return it.
            return n;
        }
        // Find the child node that contains `previousToken` or starts immediately after it.
        let foundNode;
        const visitChild = (node) => {
            if (node.flags & NodeFlags.Reparsed) {
                return undefined;
            }
            if (node.pos <= previousToken.end && node.end > previousToken.end) {
                foundNode = node;
            }
            return undefined;
        };
        // Visit JSDoc children first (mirrors Go's VisitEachChildAndJSDoc).
        if (n.jsDoc) {
            for (const jsdoc of n.jsDoc) {
                visitChild(jsdoc);
            }
        }
        n.forEachChild(visitChild, nodes => {
            if (nodes.length > 0 && foundNode === undefined) {
                for (const node of nodes) {
                    if (node.flags & NodeFlags.Reparsed)
                        continue;
                    if (node.pos > previousToken.end)
                        break;
                    if (node.end > previousToken.end) {
                        foundNode = node;
                        break;
                    }
                }
            }
            return undefined;
        });
        // Recurse into the found child.
        if (foundNode !== undefined) {
            return find(foundNode);
        }
        // No AST child covers the position; use the scanner to find the syntactic token.
        // The scanner is initialized at `previousToken.end`, so tokenFullStart === previousToken.end.
        const startPos = previousToken.end;
        if (startPos >= n.pos && startPos < n.end) {
            const scanner = getScannerForSourceFile(sourceFile, startPos);
            const token = scanner.getToken();
            const tokenFullStart = scanner.getTokenFullStart();
            const tokenEnd = scanner.getTokenEnd();
            const flags = scanner.getTokenFlags();
            return getOrCreateToken(sourceFile, token, tokenFullStart, tokenEnd, n, flags);
        }
        return undefined;
    }
}
/**
 * Finds the leftmost token satisfying `position < token.end`.
 * If the position is in the trivia of that leftmost token, or the token is invalid,
 * returns the rightmost valid token with `token.end <= position`.
 * Excludes `JsxText` tokens containing only whitespace.
 */
export function findPrecedingToken(sourceFile, position) {
    return findPrecedingTokenImpl(sourceFile, position, sourceFile);
}
function getTokenAtPositionImpl(sourceFile, position, allowPositionInLeadingTrivia, includePrecedingTokenAtEndPosition) {
    let current = sourceFile;
    let nodeAfterLeft;
    const state = {
        next: undefined,
        prevSubtree: undefined,
        left: 0,
    };
    const testNode = (node) => {
        if (node.kind !== SyntaxKind.EndOfFile && node.end === position && includePrecedingTokenAtEndPosition !== undefined) {
            state.prevSubtree = node;
        }
        // A node "contains" the position if position < end, except nodes at the file end
        // treat end as inclusive (there's nowhere else to look). This applies to the EOF
        // token itself, and to JSDoc nodes reaching EOF (e.g. unterminated JSDoc comments).
        if (node.end < position || node.end === position &&
            node.kind !== SyntaxKind.EndOfFile &&
            (!isJSDocNodeKind(node.kind) || node.end !== sourceFile.endOfFileToken.end)) {
            return -1;
        }
        const nodePos = getPosition(node, sourceFile, allowPositionInLeadingTrivia);
        if (nodePos > position) {
            return 1;
        }
        return 0;
    };
    while (true) {
        // Visit each child of current to find the one containing the position.
        state.next = undefined;
        nodeAfterLeft = undefined;
        // In Strada, JSDoc nodes with a single comment represent that comment as a string
        // property (not a child node), so forEachChild does not visit it. We replicate this
        // by detecting single-comment NodeLists in visitList and skipping their elements in visitNode.
        let skipSingleCommentChildren = false;
        const visitNode = (node) => {
            if (node.flags & NodeFlags.Reparsed) {
                return undefined;
            }
            if (skipSingleCommentChildren && isJSDocCommentChildKind(node.kind)) {
                return undefined;
            }
            if (nodeAfterLeft === undefined) {
                nodeAfterLeft = node;
            }
            if (state.next === undefined) {
                const result = testNode(node);
                switch (result) {
                    case -1:
                        if (!isJSDocNodeKind(node.kind)) {
                            state.left = node.end;
                        }
                        nodeAfterLeft = undefined;
                        break;
                    case 0:
                        state.next = node;
                        break;
                }
            }
            return undefined;
        };
        // Visit JSDoc children first, then regular children (mirrors Go's VisitEachChildAndJSDoc).
        if (current.jsDoc) {
            for (const jsdoc of current.jsDoc) {
                visitNode(jsdoc);
            }
        }
        current.forEachChild(visitNode, nodes => {
            // Track whether this NodeList is a single-comment list that should be skipped.
            // The flag affects the subsequent forEachNode(visitNode) calls for this NodeList.
            skipSingleCommentChildren = isJSDocSingleCommentNodeList(nodes);
            if (nodes.length === 0 || skipSingleCommentChildren) {
                return undefined;
            }
            if (nodeAfterLeft === undefined) {
                for (const node of nodes) {
                    if (!(node.flags & NodeFlags.Reparsed)) {
                        nodeAfterLeft = node;
                        break;
                    }
                }
            }
            if (state.next === undefined) {
                if (nodes.end === position && includePrecedingTokenAtEndPosition !== undefined) {
                    state.left = nodes.end;
                    nodeAfterLeft = undefined;
                    state.prevSubtree = nodes[nodes.length - 1];
                }
                else if (nodes.end <= position) {
                    state.left = nodes.end;
                    nodeAfterLeft = undefined;
                }
                else if (nodes.pos <= position) {
                    binarySearchNodeList(nodes, testNode, (node, middle, arr) => {
                        state.left = node.end;
                        nodeAfterLeft = undefined;
                        for (let i = middle + 1; i < arr.length; i++) {
                            if (!(arr[i].flags & NodeFlags.Reparsed)) {
                                nodeAfterLeft = arr[i];
                                break;
                            }
                        }
                    }, found => {
                        state.next = found;
                    });
                }
            }
            return undefined;
        });
        // If prevSubtree was set, check if the rightmost token of prevSubtree should be returned.
        if (state.prevSubtree !== undefined) {
            const child = findPrecedingTokenImpl(sourceFile, position, state.prevSubtree);
            if (child !== undefined && child.end === position && includePrecedingTokenAtEndPosition(child)) {
                return child;
            }
            state.prevSubtree = undefined;
        }
        if (state.next === undefined) {
            if (isTokenKind(current.kind) || shouldSkipChild(current)) {
                return current;
            }
            // Use the scanner to find a token not stored in the AST.
            const scanner = getScannerForSourceFile(sourceFile, state.left);
            let end = current.end;
            const afterLeft = nodeAfterLeft;
            if (afterLeft !== undefined) {
                end = afterLeft.pos;
            }
            while (state.left < end) {
                const token = scanner.getToken();
                const tokenFullStart = scanner.getTokenFullStart();
                const tokenStart = allowPositionInLeadingTrivia ? tokenFullStart : scanner.getTokenStart();
                const tokenEnd = scanner.getTokenEnd();
                const flags = scanner.getTokenFlags();
                if (tokenEnd > end) {
                    break;
                }
                if (tokenStart <= position && position < tokenEnd) {
                    if (token === SyntaxKind.Identifier || !isTokenKind(token)) {
                        if (isJSDocNodeKind(current.kind)) {
                            return current;
                        }
                        throw new Error(`did not expect ${SyntaxKind[current.kind]} to have ${SyntaxKind[token]} in its trivia`);
                    }
                    return getOrCreateToken(sourceFile, token, tokenFullStart, tokenEnd, current, flags);
                }
                if (includePrecedingTokenAtEndPosition !== undefined && tokenEnd === position) {
                    const prevToken = getOrCreateToken(sourceFile, token, tokenFullStart, tokenEnd, current, flags);
                    if (includePrecedingTokenAtEndPosition(prevToken)) {
                        return prevToken;
                    }
                }
                state.left = tokenEnd;
                scanner.scan();
            }
            return current;
        }
        current = state.next;
        state.left = current.pos;
        nodeAfterLeft = undefined;
    }
}
function getPosition(node, sourceFile, allowPositionInLeadingTrivia) {
    if (allowPositionInLeadingTrivia) {
        return node.pos;
    }
    return getTokenPosOfNode(node, sourceFile, /*includeJSDoc*/ true);
}
/** @internal */
export function getTokenPosOfNode(node, sourceFile, includeJSDoc) {
    // With nodes that have no width (i.e. 'Missing' nodes), we actually *don't*
    // want to skip trivia because this will launch us forward to the next token.
    if (nodeIsMissing(node)) {
        return node.pos;
    }
    if (isJSDocNodeKind(node.kind) || node.kind === SyntaxKind.JsxText) {
        return skipTrivia(sourceFile.text, node.pos, /*stopAfterLineBreak*/ false, /*stopAtComments*/ true);
    }
    if (includeJSDoc && node.jsDoc && node.jsDoc.length > 0) {
        return getTokenPosOfNode(node.jsDoc[0], sourceFile, /*includeJSDoc*/ false);
    }
    return skipTrivia(sourceFile.text, node.pos, /*stopAfterLineBreak*/ false, /*stopAtComments*/ false, /*inJSDoc*/ !!(node.flags & NodeFlags.JSDoc));
}
function nodeIsMissing(node) {
    return node.pos === node.end && node.pos >= 0 && node.kind !== SyntaxKind.EndOfFile;
}
function findPrecedingTokenImpl(sourceFile, position, startNode) {
    const find = (n) => {
        if (isTokenKind(n.kind) && n.kind !== SyntaxKind.EndOfFile) {
            return n;
        }
        let foundChild;
        let prevChild;
        // Visit JSDoc nodes first (mirrors Go's VisitEachChildAndJSDoc).
        if (n.jsDoc) {
            for (const jsdoc of n.jsDoc) {
                if (jsdoc.flags & NodeFlags.Reparsed)
                    continue;
                if (foundChild !== undefined)
                    break;
                if (position < jsdoc.end && (prevChild === undefined || prevChild.end <= position)) {
                    foundChild = jsdoc;
                }
                else {
                    prevChild = jsdoc;
                }
            }
        }
        let skipSingleCommentChildrenImpl = false;
        n.forEachChild(node => {
            if (node.flags & NodeFlags.Reparsed) {
                return undefined;
            }
            if (skipSingleCommentChildrenImpl && isJSDocCommentChildKind(node.kind)) {
                return undefined;
            }
            if (foundChild !== undefined) {
                return undefined;
            }
            if (position < node.end && (prevChild === undefined || prevChild.end <= position)) {
                foundChild = node;
            }
            else {
                prevChild = node;
            }
            return undefined;
        }, nodes => {
            skipSingleCommentChildrenImpl = isJSDocSingleCommentNodeList(nodes);
            if (foundChild !== undefined) {
                return undefined;
            }
            if (nodes.length > 0 && !skipSingleCommentChildrenImpl) {
                const index = binarySearchForPrecedingToken(nodes, position);
                if (index >= 0 && !(nodes[index].flags & NodeFlags.Reparsed)) {
                    foundChild = nodes[index];
                }
                const lookupIndex = index >= 0 ? index - 1 : nodes.length - 1;
                for (let i = lookupIndex; i >= 0; i--) {
                    if (!(nodes[i].flags & NodeFlags.Reparsed)) {
                        if (prevChild === undefined) {
                            prevChild = nodes[i];
                        }
                        break;
                    }
                }
            }
            return undefined;
        });
        if (foundChild !== undefined) {
            const start = getTokenPosOfNode(foundChild, sourceFile, /*includeJSDoc*/ true);
            if (start >= position) {
                if (position >= foundChild.pos) {
                    // We are in the leading trivia of foundChild. Check for JSDoc nodes of n
                    // preceding foundChild, mirroring Go's findPrecedingToken logic.
                    let jsDoc;
                    if (n.jsDoc) {
                        for (let i = n.jsDoc.length - 1; i >= 0; i--) {
                            if (n.jsDoc[i].pos >= foundChild.pos) {
                                jsDoc = n.jsDoc[i];
                                break;
                            }
                        }
                    }
                    if (jsDoc !== undefined) {
                        if (position < jsDoc.end) {
                            return find(jsDoc);
                        }
                        return findRightmostValidToken(sourceFile, jsDoc.end, n, position);
                    }
                    return findRightmostValidToken(sourceFile, foundChild.pos, n, -1);
                }
                // Answer is in tokens between two visited children.
                return findRightmostValidToken(sourceFile, foundChild.pos, n, position);
            }
            return find(foundChild);
        }
        if (position >= n.end) {
            return findRightmostValidToken(sourceFile, n.end, n, -1);
        }
        return findRightmostValidToken(sourceFile, n.end, n, position);
    };
    return find(startNode);
}
function findRightmostValidToken(sourceFile, endPos, containingNode, position) {
    if (position === -1) {
        position = containingNode.end;
    }
    const find = (n, endPos) => {
        if (isTokenKind(n.kind) && n.kind !== SyntaxKind.EndOfFile) {
            return n;
        }
        let rightmostValidNode;
        let hasChildren = false;
        // Visit JSDoc nodes first (mirrors Go's VisitEachChildAndJSDoc).
        if (n.jsDoc) {
            hasChildren = true;
            for (const jsdoc of n.jsDoc) {
                if (jsdoc.flags & NodeFlags.Reparsed)
                    continue;
                if (jsdoc.end > endPos || getTokenPosOfNode(jsdoc, sourceFile) >= position)
                    continue;
                if (isValidPrecedingNode(jsdoc, sourceFile)) {
                    rightmostValidNode = jsdoc;
                }
            }
        }
        let skipSingleCommentChildren = false;
        n.forEachChild(node => {
            if (node.flags & NodeFlags.Reparsed) {
                return undefined;
            }
            if (skipSingleCommentChildren && isJSDocCommentChildKind(node.kind)) {
                return undefined;
            }
            hasChildren = true;
            if (node.end > endPos || getTokenPosOfNode(node, sourceFile) >= position) {
                return undefined;
            }
            if (isValidPrecedingNode(node, sourceFile)) {
                rightmostValidNode = node;
            }
            return undefined;
        }, nodes => {
            // Skip single-comment JSDoc NodeLists (e.g. JSDocText children of a JSDoc node):
            // In Go, these are stored as string properties and are never visited as children.
            skipSingleCommentChildren = isJSDocSingleCommentNodeList(nodes);
            if (nodes.length > 0 && !skipSingleCommentChildren) {
                hasChildren = true;
                for (let i = nodes.length - 1; i >= 0; i--) {
                    const node = nodes[i];
                    if (node.flags & NodeFlags.Reparsed)
                        continue;
                    if (node.end > endPos || getTokenPosOfNode(node, sourceFile) >= position)
                        continue;
                    if (isValidPrecedingNode(node, sourceFile)) {
                        rightmostValidNode = node;
                        break;
                    }
                }
            }
            return undefined;
        });
        // Scan for syntactic tokens (e.g. `{`, `,`) between AST nodes, matching Go's
        // findRightmostValidToken scanner step.
        if (!shouldSkipChild(n)) {
            const startPos = rightmostValidNode !== undefined ? rightmostValidNode.end : n.pos;
            const targetEnd = Math.min(endPos, position);
            if (startPos < targetEnd) {
                const scanner = getScannerForSourceFile(sourceFile, startPos);
                let pos = startPos;
                let lastScannedToken;
                while (pos < targetEnd) {
                    const tokenStart = scanner.getTokenStart();
                    if (tokenStart >= position)
                        break;
                    const tokenFullStart = scanner.getTokenFullStart();
                    const tokenEnd = scanner.getTokenEnd();
                    const token = scanner.getToken();
                    const flags = scanner.getTokenFlags();
                    lastScannedToken = getOrCreateToken(sourceFile, token, tokenFullStart, tokenEnd, n, flags);
                    pos = tokenEnd;
                    scanner.scan();
                }
                if (lastScannedToken !== undefined) {
                    return lastScannedToken;
                }
            }
        }
        if (!hasChildren) {
            if (n !== containingNode) {
                return n;
            }
            return undefined;
        }
        if (rightmostValidNode !== undefined) {
            return find(rightmostValidNode, rightmostValidNode.end);
        }
        return undefined;
    };
    return find(containingNode, endPos);
}
function isValidPrecedingNode(node, sourceFile) {
    if (node.kind === SyntaxKind.EndOfFile) {
        return false;
    }
    const start = getTokenPosOfNode(node, sourceFile);
    const width = node.end - start;
    return width > 0;
}
function shouldSkipChild(node) {
    return node.kind === SyntaxKind.JSDoc ||
        node.kind === SyntaxKind.JSDocText ||
        node.kind === SyntaxKind.JSDocTypeLiteral ||
        node.kind === SyntaxKind.JSDocSignature ||
        node.kind === SyntaxKind.JSDocLink ||
        node.kind === SyntaxKind.JSDocLinkCode ||
        node.kind === SyntaxKind.JSDocLinkPlain ||
        isJSDocTag(node);
}
function isJSDocTag(node) {
    return node.kind >= SyntaxKind.FirstJSDocTagNode && node.kind <= SyntaxKind.LastJSDocTagNode;
}
// In Strada, if a JSDoc node has a single comment, that comment is represented as a string
// property as a simplification, and therefore that comment is not visited by forEachChild.
// To match, we skip single-element comment NodeLists within JSDoc/JSDocTag nodes.
function isJSDocCommentChildKind(kind) {
    switch (kind) {
        case SyntaxKind.JSDocText:
        case SyntaxKind.JSDocLink:
        case SyntaxKind.JSDocLinkCode:
        case SyntaxKind.JSDocLinkPlain:
            return true;
        default:
            return false;
    }
}
function isJSDocSingleCommentNodeList(nodes) {
    return nodes.length === 1 && isJSDocCommentChildKind(nodes[0].kind);
}
function getScannerForSourceFile(sourceFile, pos) {
    const scanner = createScanner(/*skipTrivia*/ true, sourceFile.languageVariant, sourceFile.text);
    scanner.resetTokenState(pos);
    scanner.scan();
    return scanner;
}
function getOrCreateToken(sourceFile, kind, pos, end, parent, _flags) {
    const key = `${pos}_${end}`;
    if (!sourceFile.tokenCache) {
        sourceFile.tokenCache = new Map();
    }
    const existing = sourceFile.tokenCache.get(key);
    if (existing !== undefined) {
        return existing;
    }
    const token = createToken(kind);
    token.pos = pos;
    token.end = end;
    token.parent = parent;
    sourceFile.tokenCache.set(key, token);
    return token;
}
/** Binary search a node list for the node containing position. */
function binarySearchNodeList(nodes, testNode, onLeft, onMatch) {
    let lo = 0;
    let hi = nodes.length - 1;
    while (lo <= hi) {
        const mid = (lo + hi) >>> 1;
        const node = nodes[mid];
        if (node.flags & NodeFlags.Reparsed) {
            // Skip reparsed nodes: try to find a non-reparsed node nearby
            let found = false;
            for (let i = mid + 1; i <= hi; i++) {
                if (!(nodes[i].flags & NodeFlags.Reparsed)) {
                    const cmp = testNode(nodes[i]);
                    if (cmp < 0) {
                        onLeft(nodes[i], i, nodes);
                        lo = i + 1;
                    }
                    else if (cmp > 0) {
                        hi = i - 1;
                    }
                    else {
                        onMatch(nodes[i]);
                        return;
                    }
                    found = true;
                    break;
                }
            }
            if (!found) {
                hi = mid - 1;
            }
            continue;
        }
        const cmp = testNode(node);
        if (cmp < 0) {
            onLeft(node, mid, nodes);
            lo = mid + 1;
        }
        else if (cmp > 0) {
            hi = mid - 1;
        }
        else {
            onMatch(node);
            return;
        }
    }
}
function binarySearchForPrecedingToken(nodes, position) {
    let lo = 0;
    let hi = nodes.length - 1;
    let result = -1;
    while (lo <= hi) {
        const mid = (lo + hi) >>> 1;
        const node = nodes[mid];
        if (node.flags & NodeFlags.Reparsed) {
            lo = mid + 1;
            continue;
        }
        if (position < node.end) {
            if (mid === 0 || position >= nodes[mid - 1].end) {
                result = mid;
                break;
            }
            hi = mid - 1;
        }
        else {
            lo = mid + 1;
        }
    }
    return result;
}
//# sourceMappingURL=astnav.js.map