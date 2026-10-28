/** Finite, data-only JScript compatibility. No eval, host objects, I/O or prototypes. */
export interface FormAddress { section: number; para: number; control: number; cellPath: number[][] }
export interface ActionForm {
  address: FormAddress; name: string; formType: string; text: string; caption: string;
  value: number; enabled: boolean; items: string[];
}
export interface FormActionContext {
  schemaVersion: number; button: FormAddress; trigger: string; header: string; source: string; forms: ActionForm[];
}
export interface FormActionPlan {
  updates: Omit<ActionForm, 'name' | 'formType'>[];
  commands: { name: 'MoveDocBegin' | 'MoveDocEnd' | 'InsertText' | 'Print' | 'FileSave'; text?: string }[];
  steps: number;
}
type Node = { k: string; [key: string]: any };
type Ref = { tag: 'host'; path: string[] } | { tag: 'form'; index: number };
type Value = string | number | boolean | null | Ref;
type Token = { v: string; literal?: Value };
const bad = (message = '지원하지 않는 script 문법 또는 기능입니다.'): never => { throw new Error(`FORM_ACTION_SCRIPT: ${message}`); };
const MAX_STEPS = 20_000, MAX_TEXT = 65_536;
const forbidden = new Set(['__proto__', 'prototype', 'constructor', 'caller', 'callee', 'arguments']);

function tokens(source: string): Token[] {
  if (source.length > 1_048_576) bad('script 크기 한도를 초과했습니다.');
  const out: Token[] = [];
  const pattern = /\s+|\/\/[^\r\n]*|\/\*[\s\S]*?\*\/|"(?:\\.|[^"\\\r\n])*"|'(?:\\.|[^'\\\r\n])*'|(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?|[$_\p{ID_Start}][$_\u200c\u200d\p{ID_Continue}]*|===|!==|==|!=|<=|>=|&&|\|\||\+\+|--|\+=|-=|[{}()[\].,;:+*\/<>!=?%-]/uy;
  let offset = 0;
  while (offset < source.length) {
    pattern.lastIndex = offset;
    const match = pattern.exec(source);
    if (!match) bad();
    const text = match![0]; offset = pattern.lastIndex;
    if (/^\s|^\/\//.test(text) || text.startsWith('/*')) continue;
    let literal: Value | undefined;
    if (/^["']/.test(text)) {
      const quote = text[0]; let result = '';
      for (let i = 1; i < text.length - 1; i++) {
        const c = text[i];
        if (c !== '\\') { result += c; continue; }
        const next = text[++i];
        if (next === 'u') {
          const hex = text.slice(i + 1, i + 5); if (!/^[\da-f]{4}$/i.test(hex)) bad();
          result += String.fromCharCode(parseInt(hex, 16)); i += 4;
        } else if (next === quote || next === '\\' || next === '/') result += next;
        else if (next === 'n') result += '\n'; else if (next === 'r') result += '\r';
        else if (next === 't') result += '\t'; else bad();
      }
      literal = result;
    } else if (/^[\d.]/.test(text) && text !== '.') literal = Number(text);
    else if (text === 'true' || text === 'false') literal = text === 'true';
    else if (text === 'null' || text === 'undefined') literal = null;
    if (forbidden.has(text)) bad();
    out.push({ v: text, ...(literal !== undefined ? { literal } : {}) });
    if (out.length > 20_000) bad('token 한도를 초과했습니다.');
  }
  out.push({ v: '<end>' }); return out;
}

class Parser {
  private at = 0;
  private depth = 0;
  readonly functions = new Map<string, { params: string[]; body: Node }>();
  private readonly list: Token[];
  constructor(list: Token[]) { this.list = list; }
  private peek(v?: string): boolean | string { return v === undefined ? this.list[this.at]?.v ?? '<end>' : this.list[this.at]?.v === v; }
  private take(v?: string): Token {
    const token = this.list[this.at++]; if (!token || (v !== undefined && token.v !== v)) bad(); return token;
  }
  private name(): string { const name = this.take().v; if (!/^[$_\p{ID_Start}][$_\u200c\u200d\p{ID_Continue}]*$/u.test(name)) bad(); return name; }
  program(): Node { const list: Node[] = []; while (!this.peek('<end>')) list.push(this.statement()); return { k: 'block', list }; }
  private statement(): Node {
    if (++this.depth > 64) bad('중첩 한도를 초과했습니다.');
    try {
      if (this.peek('{')) { this.take(); const list: Node[] = []; while (!this.peek('}')) { if (this.peek('<end>')) bad(); list.push(this.statement()); } this.take('}'); return { k: 'block', list }; }
      if (this.peek(';')) { this.take(); return { k: 'block', list: [] }; }
      if (this.peek('function')) {
        if (this.depth !== 1) bad('중첩 함수 선언은 지원하지 않습니다.');
        this.take(); const name = this.name(); this.take('('); const params: string[] = [];
        while (!this.peek(')')) { params.push(this.name()); if (params.length > 16) bad(); if (!this.peek(',')) break; this.take(); }
        this.take(')'); const body = this.statement();
        if (this.functions.has(name) || this.functions.size >= 256) bad(); this.functions.set(name, { params, body });
        return { k: 'block', list: [] };
      }
      if (['var', 'let', 'const'].includes(this.peek() as string)) {
        this.take(); const declarations: Node[] = [];
        do { if (this.peek(',')) this.take(); const name = this.name();
          const value = this.peek('=') ? (this.take(), this.expression(1)) : { k: 'literal', value: null };
          declarations.push({ k: 'declare', name, value });
        } while (this.peek(','));
        if (this.peek(';')) this.take(); return { k: 'block', list: declarations };
      }
      if (this.peek('if')) {
        this.take(); this.take('('); const condition = this.expression(); this.take(')'); const yes = this.statement();
        const no = this.peek('else') ? (this.take(), this.statement()) : null; return { k: 'if', condition, yes, no };
      }
      if (this.peek('while')) {
        this.take(); this.take('('); const condition = this.expression(); this.take(')'); return { k: 'loop', condition, body: this.statement() };
      }
      if (this.peek('for')) {
        this.take(); this.take('('); const init = this.statement();
        const condition = this.peek(';') ? { k: 'literal', value: true } : this.expression(); this.take(';');
        const after = this.peek(')') ? null : this.expression(); this.take(')'); return { k: 'loop', init, condition, after, body: this.statement() };
      }
      if (this.peek('return')) { this.take(); const value = this.peek(';') || this.peek('}') ? null : this.expression(); if (this.peek(';')) this.take(); return { k: 'return', value }; }
      const value = this.expression(); if (this.peek(';')) this.take(); return { k: 'expression', value };
    } finally { this.depth--; }
  }
  private expression(min = 0): Node {
    if (++this.depth > 64) bad('중첩 한도를 초과했습니다.');
    try {
      const first = this.take(); let left: Node;
      if (first.v === '(') { left = this.expression(); this.take(')'); }
      else if (['!', '-', '+'].includes(first.v)) left = { k: 'unary', op: first.v, value: this.expression(8) };
      else if ('literal' in first) left = { k: 'literal', value: first.literal };
      else if (/^[$_\p{ID_Start}]/u.test(first.v)) left = { k: 'identifier', name: first.v };
      else return bad();
      const precedence: Record<string, number> = { '=': 1, '+=': 1, '-=': 1, '||': 2, '&&': 3,
        '==': 4, '!=': 4, '===': 4, '!==': 4, '<': 5, '>': 5, '<=': 5, '>=': 5, '+': 6, '-': 6, '*': 7, '/': 7, '%': 7 };
      while (true) {
        const next = this.peek() as string;
        if (next === '.') { this.take(); left = { k: 'member', object: left, name: { k: 'literal', value: this.name() } }; continue; }
        if (next === '[') { this.take(); const name = this.expression(); this.take(']'); left = { k: 'member', object: left, name }; continue; }
        if (next === '(') { this.take(); const args: Node[] = []; while (!this.peek(')')) { args.push(this.expression()); if (args.length > 16) bad(); if (!this.peek(',')) break; this.take(); } this.take(')'); left = { k: 'call', callee: left, args }; continue; }
        if (next === '++' || next === '--') { this.take(); left = { k: 'increment', value: left, delta: next === '++' ? 1 : -1 }; continue; }
        const rank = precedence[next]; if (!rank || rank < min) break;
        this.take(); const right = this.expression(rank === 1 ? rank : rank + 1); left = { k: rank === 1 ? 'assign' : 'binary', op: next, left, right };
      }
      return left;
    } finally { this.depth--; }
  }
}

export function evaluateFormAction(context: FormActionContext): FormActionPlan {
  if (context.schemaVersion !== 1 || context.forms.length > 4096) bad();
  const parser = new Parser(tokens(`${context.header}\n${context.source}`)); const program = parser.program();
  const environment = new Map<string, Value>();
  for (const name of ['XHwpDocuments', 'HAction', 'HParameterSet', 'Math', 'String', 'Number', 'parseInt']) environment.set(name, { tag: 'host', path: [name] });
  const forms = structuredClone(context.forms); const dirty = new Set<number>();
  const commands: FormActionPlan['commands'] = []; const parameters = new Map<string, Value>();
  let steps = 0, calls = 0, outputBytes = 0, valueWork = 0;
  const parents = new WeakMap<Map<string, Value>, Map<string, Value>>();
  const owner = (name: string, current: Map<string, Value>): Map<string, Value> | null => {
    for (let env: Map<string, Value> | undefined = current; env; env = parents.get(env)) if (env.has(name)) return env;
    return null;
  };
  const tick = () => { if (++steps > MAX_STEPS) bad('실행 단계 한도를 초과했습니다.'); };
  const checked = (value: Value): Value => {
    if (typeof value === 'string' && (valueWork += value.length * 2) > 4_194_304) bad('문자열 작업 예산을 초과했습니다.');
    if ((typeof value === 'string' && value.length > MAX_TEXT) || (typeof value === 'number' && !Number.isFinite(value))) bad('값의 크기 한도를 초과했습니다.'); return value;
  };
  const member = (node: Node): { object: Value; name: string } => {
    const object = expression(node.object), name = String(expression(node.name)); if (forbidden.has(name) || name.length > 256) bad(); return { object, name };
  };
  const fieldNames: Record<string, keyof ActionForm> = { Text: 'text', Caption: 'caption', Value: 'value', Enabled: 'enabled' };
  const get = (object: Value, name: string): Value => {
    if (typeof object === 'string' && name === 'length') return object.length;
    if (!object || typeof object !== 'object') return bad();
    if (object.tag === 'form') { const field = fieldNames[name]; return field ? forms[object.index][field] as Value : { tag: 'host', path: ['formMethod', String(object.index), name] }; }
    const path = [...object.path, name];
    if (path[0] === 'HParameterSet' && name !== 'HSet' && path.length >= 3) return parameters.get(path.join('.')) ?? null;
    return { tag: 'host', path };
  };
  const assign = (node: Node, value: Value, scope: Map<string, Value>): Value => {
    checked(value);
    if (node.k === 'identifier') { if (!owner(node.name, scope) || typeof value === 'object' && value !== null && value.tag === 'host' && value.path.length > 16) bad(); owner(node.name, scope)!.set(node.name, value); return value; }
    if (node.k !== 'member') return bad();
    const { object, name } = member(node);
    if (!object || typeof object !== 'object') return bad();
    if (object.tag === 'form') {
      const form = forms[object.index];
      if (name === 'Text' || name === 'Caption') form[fieldNames[name] as 'text' | 'caption'] = String(value);
      else if (name === 'Value') { const number = Number(value); if (!Number.isInteger(number) || number < -2147483648 || number > 2147483647) bad(); form.value = number; }
      else if (name === 'Enabled') form.enabled = Boolean(value);
      else return bad(); dirty.add(object.index); return value;
    }
    if (object.path[0] === 'HParameterSet' && object.path.length <= 3) { parameters.set([...object.path, name].join('.'), value); return value; }
    return bad();
  };
  const invoke = (ref: Value, args: Value[], scope: Map<string, Value>): Value => {
    if (!ref || typeof ref !== 'object' || ref.tag !== 'host') return bad();
    const path = ref.path, method = path.at(-1)!;
    if (method === 'ItemFromName' && typeof args[0] === 'string') {
      const matches = forms.flatMap((form, index) => form.name === args[0] ? [index] : []);
      if (matches.length !== 1) return bad('양식 이름을 유일하게 찾을 수 없습니다.'); return { tag: 'form', index: matches[0] };
    }
    if (path[0] === 'formMethod') {
      const index = Number(path[1]), form = forms[index];
      if (!form) return bad();
      if (method === 'ResetContent') form.items = [];
      else if (method === 'InsertString') {
        const at = Number(args[1]); if (!Number.isInteger(at) || at < 0 || at > form.items.length || form.items.length >= 512) return bad();
        const text = String(args[0]); checked(text); form.items.splice(at, 0, text);
      } else return bad();
      dirty.add(index); return true;
    }
    if (path[0] === 'HAction') {
      const name = String(args[0]);
      if (method === 'GetDefault' && name === 'InsertText') { parameters.set('HParameterSet.HInsertText.Text', ''); return true; }
      if (!['Run', 'Execute'].includes(method)) return bad();
      if (!['MoveDocBegin', 'MoveDocEnd', 'InsertText', 'Print', 'FileSave'].includes(name)) return bad(`지원하지 않는 편집 명령: ${name}`);
      const text = name === 'InsertText' ? String(parameters.get('HParameterSet.HInsertText.Text') ?? '') : undefined;
      outputBytes += text?.length ?? 0;
      if (commands.length >= 512 || outputBytes > 262144) return bad('편집 계획 한도를 초과했습니다.');
      commands.push({ name: name as FormActionPlan['commands'][number]['name'], ...(text !== undefined ? { text } : {}) }); return true;
    }
    if (path.length === 1 && ['String', 'Number', 'parseInt'].includes(method)) {
      if (method === 'String') return checked(String(args[0]));
      return checked(method === 'Number' ? Number(args[0]) : parseInt(String(args[0]), args[1] === undefined ? 10 : Number(args[1])));
    }
    if (path[0] === 'Math' && path.length === 2 && ['round', 'floor', 'ceil', 'abs', 'min', 'max'].includes(method)) {
      const numbers = args.map(Number); return checked((Math[method as 'min'] as (...v: number[]) => number)(...numbers));
    }
    return bad();
  };
  let scope = environment;
  const expression = (node: Node): Value => {
    tick();
    if (node.k === 'literal') return checked(node.value);
    if (node.k === 'identifier') {
      if (parser.functions.has(node.name)) return { tag: 'host', path: ['function', node.name] };
      const env = owner(node.name, scope); if (!env) return bad(`이름을 찾을 수 없습니다: ${node.name}`); return env.get(node.name)!;
    }
    if (node.k === 'member') { const value = member(node); return get(value.object, value.name); }
    if (node.k === 'unary') { const v = expression(node.value); return checked(node.op === '!' ? !v : node.op === '-' ? -Number(v) : Number(v)); }
    if (node.k === 'increment') { const before = expression(node.value); assign(node.value, checked(Number(before) + node.delta), scope); return before; }
    if (node.k === 'assign') {
      let value = expression(node.right);
      if (node.op !== '=') { const before = expression(node.left); value = node.op === '+=' ? (typeof before === 'string' || typeof value === 'string' ? String(before) + String(value) : Number(before) + Number(value)) : Number(before) - Number(value); }
      return assign(node.left, checked(value), scope);
    }
    if (node.k === 'call') {
      const ref = expression(node.callee), args = node.args.map(expression);
      if (ref && typeof ref === 'object' && ref.tag === 'host' && ref.path[0] === 'function') {
        const fn = parser.functions.get(ref.path[1])!; if (++calls > 32) return bad('호출 깊이 한도를 초과했습니다.');
        const previous = scope; scope = new Map(); parents.set(scope, environment); fn.params.forEach((name, index) => scope.set(name, args[index] ?? null));
        try { const returned = statement(fn.body); return returned?.value ?? null; } finally { scope = previous; calls--; }
      }
      return invoke(ref, args, scope);
    }
    if (node.k === 'binary') {
      const a = expression(node.left);
      if (node.op === '&&' && !a || node.op === '||' && a) return a;
      const b = expression(node.right);
      switch (node.op) {
        case '&&': case '||': return b;
        case '+': return checked(typeof a === 'string' || typeof b === 'string' ? String(a) + String(b) : Number(a) + Number(b));
        case '-': return checked(Number(a) - Number(b)); case '*': return checked(Number(a) * Number(b));
        case '/': return checked(Number(a) / Number(b)); case '%': return checked(Number(a) % Number(b));
        case '==': return a == b; case '!=': return a != b; case '===': return a === b; case '!==': return a !== b;
        case '<': return Number(a) < Number(b); case '>': return Number(a) > Number(b);
        case '<=': return Number(a) <= Number(b); case '>=': return Number(a) >= Number(b);
      }
    }
    return bad();
  };
  const statement = (node: Node): { value: Value } | null => {
    tick();
    switch (node.k) {
      case 'block': for (const child of node.list) { const result = statement(child); if (result) return result; } break;
      case 'declare': if (scope.size >= 256) return bad('변수 한도를 초과했습니다.'); scope.set(node.name, expression(node.value)); break;
      case 'expression': expression(node.value); break;
      case 'if': if (expression(node.condition)) return statement(node.yes); else if (node.no) return statement(node.no); break;
      case 'loop':
        if (node.init) statement(node.init);
        while (expression(node.condition)) { const result = statement(node.body); if (result) return result; if (node.after) expression(node.after); } break;
      case 'return': return { value: node.value ? expression(node.value) : null };
      default: return bad();
    }
    return null;
  };
  const button = forms.findIndex(form => JSON.stringify(form.address) === JSON.stringify(context.button));
  if (button < 0 || !forms[button].enabled) bad(); environment.set('this', { tag: 'form', index: button });
  statement(program);
  const trigger = parser.functions.get(context.trigger); if (!trigger) bad('이 단추의 Click 처리기가 없습니다.');
  statement(trigger!.body);
  const updates = [...dirty].map(index => { const { name: _name, formType: _type, ...update } = forms[index]; return update; });
  if (JSON.stringify(updates).length > 1_048_576) bad('편집 계획 한도를 초과했습니다.');
  return { updates, commands, steps };
}
