import { SyntaxKind } from "#enums/syntaxKind";
import { cloneNode, createNodeArray, createNumericLiteral, createStringLiteral, } from "./factory.generated.js";
import { visitEachChild } from "./visitor.js";
function isArray(value) {
    // See: https://github.com/microsoft/TypeScript/issues/17002
    return Array.isArray(value);
}
function forEachChildRecursively(rootNode, cbNode, cbNodes) {
    const queue = gatherPossibleChildren(rootNode);
    const parents = []; // tracks parent references for elements in queue
    while (parents.length < queue.length) {
        parents.push(rootNode);
    }
    while (queue.length !== 0) {
        const current = queue.pop();
        const parent = parents.pop();
        if (isArray(current)) {
            if (cbNodes) {
                const res = cbNodes(current, parent);
                if (res) {
                    if (res === "skip")
                        continue;
                    return res;
                }
            }
            for (let i = current.length - 1; i >= 0; --i) {
                queue.push(current[i]);
                parents.push(parent);
            }
        }
        else {
            const res = cbNode(current, parent);
            if (res) {
                if (res === "skip")
                    continue;
                return res;
            }
            if (current.kind >= SyntaxKind.FirstNode) {
                // add children in reverse order to the queue, so popping gives the first child
                for (const child of gatherPossibleChildren(current)) {
                    queue.push(child);
                    parents.push(current);
                }
            }
        }
    }
}
function gatherPossibleChildren(node) {
    const children = [];
    node.forEachChild(addWorkItem, addWorkItem); // By using a stack above and `unshift` here, we emulate a depth-first preorder traversal
    return children;
    function addWorkItem(n) {
        children.unshift(n);
    }
}
function setParentRecursive(rootNode) {
    if (rootNode === undefined)
        return rootNode;
    forEachChildRecursively(rootNode, (child, parent) => {
        child.parent = parent;
        return undefined;
    });
    return rootNode;
}
function setTextRange(node, range) {
    if (range) {
        node.pos = range.pos;
        node.end = range.end;
    }
    return node;
}
export function getSynthesizedDeepClone(node, includeTrivia = true) {
    const clone = node && getSynthesizedDeepCloneWorker(node);
    if (clone && !includeTrivia) {
        clone.pos = -1;
        clone.end = -1;
    }
    return setParentRecursive(clone);
}
export function getSynthesizedDeepClones(nodes, includeTrivia = true) {
    if (nodes) {
        const cloned = createNodeArray(nodes.map(n => getSynthesizedDeepClone(n, includeTrivia)), nodes.pos, nodes.end);
        return cloned;
    }
    return nodes;
}
function getSynthesizedDeepCloneWorker(node) {
    const visited = visitEachChild(node, n => getSynthesizedDeepCloneWorker(n));
    if (visited === node) {
        // Leaf node — visitEachChild returned the same node since there are no children.
        // We need to explicitly clone it.
        const clone = node.kind === SyntaxKind.StringLiteral
            ? createStringLiteral(node.text, node.tokenFlags)
            : node.kind === SyntaxKind.NumericLiteral
                ? createNumericLiteral(node.text, node.tokenFlags)
                : cloneNode(node);
        return setTextRange(clone, node);
    }
    // visitEachChild already created a new node with visited children.
    // Clear the parent since setParentRecursive will set it later.
    visited.parent = undefined;
    return visited;
}
//# sourceMappingURL=clone.js.map