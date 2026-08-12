// is.ts — Hand-written type guard functions
// Generated guards are in is.generated.ts
import { NodeFlags } from "#enums/nodeFlags";
import { OuterExpressionKinds } from "#enums/outerExpressionKinds";
import { ScriptKind } from "#enums/scriptKind";
import { SyntaxKind } from "#enums/syntaxKind";
import { isLiteralExpression } from "./is.generated.js";
export * from "./is.generated.js";
export function isTypeNode(node) {
    return isTypeNodeKind(node.kind);
}
function isTypeNodeKind(kind) {
    return kind >= SyntaxKind.FirstTypeNode && kind <= SyntaxKind.LastTypeNode
        || kind === SyntaxKind.AnyKeyword
        || kind === SyntaxKind.UnknownKeyword
        || kind === SyntaxKind.NumberKeyword
        || kind === SyntaxKind.BigIntKeyword
        || kind === SyntaxKind.ObjectKeyword
        || kind === SyntaxKind.BooleanKeyword
        || kind === SyntaxKind.StringKeyword
        || kind === SyntaxKind.SymbolKeyword
        || kind === SyntaxKind.VoidKeyword
        || kind === SyntaxKind.UndefinedKeyword
        || kind === SyntaxKind.NeverKeyword
        || kind === SyntaxKind.IntrinsicKeyword
        || kind === SyntaxKind.ExpressionWithTypeArguments
        || kind === SyntaxKind.JSDocAllType
        || kind === SyntaxKind.JSDocNullableType
        || kind === SyntaxKind.JSDocNonNullableType
        || kind === SyntaxKind.JSDocOptionalType
        || kind === SyntaxKind.JSDocVariadicType
        || kind === SyntaxKind.JSDocTypeExpression
        || kind === SyntaxKind.JSDocTypeLiteral
        || kind === SyntaxKind.JSDocSignature;
}
export function isStatement(node) {
    const kind = node.kind;
    return kind === SyntaxKind.VariableStatement || kind === SyntaxKind.EmptyStatement
        || kind === SyntaxKind.ExpressionStatement || kind === SyntaxKind.IfStatement
        || kind === SyntaxKind.DoStatement || kind === SyntaxKind.WhileStatement
        || kind === SyntaxKind.ForStatement || kind === SyntaxKind.ForInStatement
        || kind === SyntaxKind.ForOfStatement || kind === SyntaxKind.ContinueStatement
        || kind === SyntaxKind.BreakStatement || kind === SyntaxKind.ReturnStatement
        || kind === SyntaxKind.WithStatement || kind === SyntaxKind.SwitchStatement
        || kind === SyntaxKind.LabeledStatement || kind === SyntaxKind.ThrowStatement
        || kind === SyntaxKind.TryStatement || kind === SyntaxKind.DebuggerStatement
        || kind === SyntaxKind.InterfaceDeclaration || kind === SyntaxKind.TypeAliasDeclaration
        || kind === SyntaxKind.EnumDeclaration || kind === SyntaxKind.ModuleDeclaration
        || kind === SyntaxKind.ImportDeclaration || kind === SyntaxKind.ImportEqualsDeclaration
        || kind === SyntaxKind.ExportDeclaration || kind === SyntaxKind.ExportAssignment
        || kind === SyntaxKind.NamespaceExportDeclaration || kind === SyntaxKind.FunctionDeclaration
        || kind === SyntaxKind.ClassDeclaration || kind === SyntaxKind.MissingDeclaration
        || kind === SyntaxKind.NotEmittedStatement || kind === SyntaxKind.Block;
}
export function isExpression(node) {
    const kind = node.kind;
    return kind === SyntaxKind.ConditionalExpression || kind === SyntaxKind.YieldExpression
        || kind === SyntaxKind.ArrowFunction || kind === SyntaxKind.BinaryExpression
        || kind === SyntaxKind.SpreadElement || kind === SyntaxKind.AsExpression
        || kind === SyntaxKind.OmittedExpression
        || kind === SyntaxKind.SatisfiesExpression
        || kind === SyntaxKind.PrefixUnaryExpression || kind === SyntaxKind.PostfixUnaryExpression
        || kind === SyntaxKind.DeleteExpression || kind === SyntaxKind.TypeOfExpression
        || kind === SyntaxKind.VoidExpression || kind === SyntaxKind.AwaitExpression
        || kind === SyntaxKind.TypeAssertionExpression
        || kind === SyntaxKind.CallExpression || kind === SyntaxKind.NewExpression
        || kind === SyntaxKind.TaggedTemplateExpression || kind === SyntaxKind.NonNullExpression
        || kind === SyntaxKind.MetaProperty || kind === SyntaxKind.JsxExpression
        || kind === SyntaxKind.PropertyAccessExpression || kind === SyntaxKind.ElementAccessExpression
        || kind === SyntaxKind.FunctionExpression || kind === SyntaxKind.ClassExpression
        || kind === SyntaxKind.ParenthesizedExpression || kind === SyntaxKind.ArrayLiteralExpression
        || kind === SyntaxKind.ObjectLiteralExpression || kind === SyntaxKind.TemplateExpression
        || kind === SyntaxKind.Identifier
        || kind === SyntaxKind.PrivateIdentifier
        || kind === SyntaxKind.NumericLiteral || kind === SyntaxKind.BigIntLiteral
        || kind === SyntaxKind.StringLiteral || kind === SyntaxKind.RegularExpressionLiteral
        || kind === SyntaxKind.NoSubstitutionTemplateLiteral || kind === SyntaxKind.JsxElement
        || kind === SyntaxKind.JsxSelfClosingElement || kind === SyntaxKind.JsxFragment
        || kind === SyntaxKind.NullKeyword || kind === SyntaxKind.TrueKeyword
        || kind === SyntaxKind.FalseKeyword || kind === SyntaxKind.ThisKeyword
        || kind === SyntaxKind.SuperKeyword || kind === SyntaxKind.ImportKeyword
        || kind === SyntaxKind.ExpressionWithTypeArguments;
}
export function isBlockOrExpression(node) {
    return node.kind === SyntaxKind.Block || isExpression(node);
}
export function isLeftHandSideExpression(node) {
    return isLeftHandSideExpressionKind(skipPartiallyEmittedExpressions(node).kind);
}
export function skipPartiallyEmittedExpressions(node) {
    return skipOuterExpressions(node, OuterExpressionKinds.PartiallyEmittedExpressions);
}
function isLeftHandSideExpressionKind(kind) {
    switch (kind) {
        case SyntaxKind.PropertyAccessExpression:
        case SyntaxKind.ElementAccessExpression:
        case SyntaxKind.NewExpression:
        case SyntaxKind.CallExpression:
        case SyntaxKind.JsxElement:
        case SyntaxKind.JsxSelfClosingElement:
        case SyntaxKind.JsxFragment:
        case SyntaxKind.TaggedTemplateExpression:
        case SyntaxKind.ArrayLiteralExpression:
        case SyntaxKind.ParenthesizedExpression:
        case SyntaxKind.ObjectLiteralExpression:
        case SyntaxKind.ClassExpression:
        case SyntaxKind.FunctionExpression:
        case SyntaxKind.Identifier:
        case SyntaxKind.PrivateIdentifier: // technically this is only an Expression if it's in a `#field in expr` BinaryExpression
        case SyntaxKind.RegularExpressionLiteral:
        case SyntaxKind.NumericLiteral:
        case SyntaxKind.BigIntLiteral:
        case SyntaxKind.StringLiteral:
        case SyntaxKind.NoSubstitutionTemplateLiteral:
        case SyntaxKind.TemplateExpression:
        case SyntaxKind.FalseKeyword:
        case SyntaxKind.NullKeyword:
        case SyntaxKind.ThisKeyword:
        case SyntaxKind.TrueKeyword:
        case SyntaxKind.SuperKeyword:
        case SyntaxKind.NonNullExpression:
        case SyntaxKind.ExpressionWithTypeArguments:
        case SyntaxKind.MetaProperty:
        case SyntaxKind.ImportKeyword: // technically this is only an Expression if it's in a CallExpression
        case SyntaxKind.MissingDeclaration:
            return true;
        default:
            return false;
    }
}
export function isUnaryExpression(node) {
    return isUnaryExpressionKind(skipPartiallyEmittedExpressions(node).kind);
}
function isUnaryExpressionKind(kind) {
    switch (kind) {
        case SyntaxKind.PrefixUnaryExpression:
        case SyntaxKind.PostfixUnaryExpression:
        case SyntaxKind.DeleteExpression:
        case SyntaxKind.TypeOfExpression:
        case SyntaxKind.VoidExpression:
        case SyntaxKind.AwaitExpression:
        case SyntaxKind.TypeAssertionExpression:
            return true;
        default:
            return isLeftHandSideExpressionKind(kind);
    }
}
/** @internal */
export function isOuterExpression(node, kinds = OuterExpressionKinds.All) {
    switch (node.kind) {
        case SyntaxKind.ParenthesizedExpression:
            if (kinds & OuterExpressionKinds.ExcludeJSDocTypeAssertion && isJSDocTypeAssertion(node)) {
                return false;
            }
            return (kinds & OuterExpressionKinds.Parentheses) !== 0;
        case SyntaxKind.TypeAssertionExpression:
        case SyntaxKind.AsExpression:
            return (kinds & OuterExpressionKinds.TypeAssertions) !== 0;
        case SyntaxKind.SatisfiesExpression:
            return (kinds & (OuterExpressionKinds.ExpressionsWithTypeArguments | OuterExpressionKinds.Satisfies)) !== 0;
        case SyntaxKind.ExpressionWithTypeArguments:
            return (kinds & OuterExpressionKinds.ExpressionsWithTypeArguments) !== 0;
        case SyntaxKind.NonNullExpression:
            return (kinds & OuterExpressionKinds.NonNullAssertions) !== 0;
        case SyntaxKind.PartiallyEmittedExpression:
            return (kinds & OuterExpressionKinds.PartiallyEmittedExpressions) !== 0;
    }
    return false;
}
/** @internal */
export function skipOuterExpressions(node, kinds = OuterExpressionKinds.All) {
    while (isOuterExpression(node, kinds)) {
        node = node.expression;
    }
    return node;
}
function isJSDocTypeAssertion(node) {
    const sourceFile = node.getSourceFile();
    if (sourceFile.scriptKind !== ScriptKind.JS && sourceFile.scriptKind !== ScriptKind.JSX) {
        return false;
    }
    const expression = node.expression;
    if (expression.kind !== SyntaxKind.AsExpression) {
        return false;
    }
    const asExpression = expression;
    return !!asExpression.type
        && (asExpression.type.flags & NodeFlags.Reparsed) !== 0;
}
export function isBindingPattern(node) {
    return node.kind === SyntaxKind.ObjectBindingPattern || node.kind === SyntaxKind.ArrayBindingPattern;
}
export function isConciseBody(node) {
    return node.kind === SyntaxKind.Block || isExpression(node);
}
export function isForInitializer(node) {
    return node.kind === SyntaxKind.VariableDeclarationList || isExpression(node);
}
export function isQuestionOrExclamationToken(node) {
    return node.kind === SyntaxKind.QuestionToken || node.kind === SyntaxKind.ExclamationToken;
}
export function isIdentifierOrThisTypeNode(node) {
    return node.kind === SyntaxKind.Identifier || node.kind === SyntaxKind.ThisType;
}
export function isReadonlyKeywordOrPlusOrMinusToken(node) {
    return node.kind === SyntaxKind.ReadonlyKeyword || node.kind === SyntaxKind.PlusToken || node.kind === SyntaxKind.MinusToken;
}
export function isQuestionOrPlusOrMinusToken(node) {
    return node.kind === SyntaxKind.QuestionToken || node.kind === SyntaxKind.PlusToken || node.kind === SyntaxKind.MinusToken;
}
export function isTemplateMiddleOrTemplateTail(node) {
    return node.kind === SyntaxKind.TemplateMiddle || node.kind === SyntaxKind.TemplateTail;
}
export function isLiteralTypeLiteral(node) {
    const kind = node.kind;
    return kind === SyntaxKind.NullKeyword || kind === SyntaxKind.TrueKeyword
        || kind === SyntaxKind.FalseKeyword || kind === SyntaxKind.PrefixUnaryExpression
        || isLiteralExpression(node);
}
export function isIdentifierOrJSDocNamespaceDeclaration(node) {
    return node.kind === SyntaxKind.Identifier || node.kind === SyntaxKind.ModuleDeclaration;
}
export function isJSDocTypeExpressionOrJSDocTypeLiteral(node) {
    return node.kind === SyntaxKind.JSDocTypeExpression || node.kind === SyntaxKind.JSDocTypeLiteral;
}
export function isJsxTagNameExpression(node) {
    const kind = node.kind;
    return kind === SyntaxKind.ThisKeyword
        || kind === SyntaxKind.Identifier
        || kind === SyntaxKind.PropertyAccessExpression
        || kind === SyntaxKind.JsxNamespacedName;
}
//# sourceMappingURL=is.js.map