/**
 * Hand-written visitor implementations for nodes with runtime-dependent
 * child ordering. Generated code in visitor.generated.ts and factory.generated.ts
 * delegates to these functions.
 */
import { SyntaxKind } from "#enums/syntaxKind";
import { updateJSDocParameterTag, updateJSDocPropertyTag, } from "./factory.generated.js";
import { isEntityName, isIdentifier, isTypeNode, } from "./is.js";
import { visitNode, visitNodes, } from "./visitor.generated.js";
export { visitEachChild, visitNode, visitNodes, visitNodesArray } from "./visitor.generated.js";
// ── forEachChild helpers (same signature as forEachChildTable entries) ──
function visitNodeForEachChild(cbNode, node) {
    return node ? cbNode(node) : undefined;
}
function visitNodesForEachChild(cbNode, cbNodes, nodes) {
    if (!nodes)
        return undefined;
    if (cbNodes)
        return cbNodes(nodes);
    for (const node of nodes) {
        const result = cbNode(node);
        if (result)
            return result;
    }
    return undefined;
}
// ── forEachChild implementations ──
function forEachChildOfJSDocParameterOrPropertyTag(data, cbNode, cbNodes) {
    return visitNodeForEachChild(cbNode, data.tagName) ||
        (data.isNameFirst
            ? visitNodeForEachChild(cbNode, data.name) || visitNodeForEachChild(cbNode, data.typeExpression)
            : visitNodeForEachChild(cbNode, data.typeExpression) || visitNodeForEachChild(cbNode, data.name)) ||
        visitNodesForEachChild(cbNode, cbNodes, data.comment);
}
export { forEachChildOfJSDocParameterOrPropertyTag as forEachChildOfJSDocParameterTag, forEachChildOfJSDocParameterOrPropertyTag as forEachChildOfJSDocPropertyTag };
// ── visitEachChild implementations ──
function visitEachChildOfJSDocParameterOrPropertyTag(node, visitor) {
    const _tagName = visitNode(node.tagName, visitor, isIdentifier);
    const _name = visitNode(node.name, visitor, isEntityName);
    const _typeExpression = visitNode(node.typeExpression, visitor, isTypeNode);
    const _comment = visitNodes(node.comment, visitor);
    return node.kind === SyntaxKind.JSDocParameterTag
        ? updateJSDocParameterTag(node, _tagName, _name, _typeExpression, _comment)
        : updateJSDocPropertyTag(node, _tagName, _name, _typeExpression, _comment);
}
export { visitEachChildOfJSDocParameterOrPropertyTag as visitEachChildOfJSDocParameterTag, visitEachChildOfJSDocParameterOrPropertyTag as visitEachChildOfJSDocPropertyTag };
//# sourceMappingURL=visitor.js.map