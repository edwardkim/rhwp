// A lexical inventory, not a Rust compiler. Unsupported mutable receiver syntax
// fails closed so the routing audit cannot silently lose exported methods.
function tokens(source: string): string[] {
  const out: string[] = [];
  let i = 0;
  while (i < source.length) {
    const rest = source.slice(i);
    if (/^\s/.test(rest)) { i++; continue; }
    if (rest.startsWith('//')) {
      const end = source.indexOf('\n', i);
      i = end < 0 ? source.length : end + 1;
      continue;
    }
    if (rest.startsWith('/*')) {
      let depth = 1;
      i += 2;
      while (i < source.length && depth) {
        if (source.startsWith('/*', i)) { depth++; i += 2; }
        else if (source.startsWith('*/', i)) { depth--; i += 2; }
        else i++;
      }
      if (depth) throw new Error('Unterminated Rust comment');
      continue;
    }
    const raw = /^(?:br|cr|r)(#*)"/.exec(rest);
    if (raw) {
      const end = source.indexOf('"' + raw[1], i + raw[0].length);
      if (end < 0) throw new Error('Unterminated Rust raw string');
      out.push(JSON.stringify(source.slice(i + raw[0].length, end)));
      i = end + 1 + raw[1].length;
      continue;
    }
    const literal = /^(?:b|c)?"(?:\\[\s\S]|[^"\\])*"|^(?:b)?'(?:\\(?:u\{[\da-fA-F]+\}|x[\da-fA-F]{2}|[\s\S])|[^'\\])'/.exec(rest);
    if (literal) { out.push(literal[0]); i += literal[0].length; continue; }
    if (rest[0] === '"') throw new Error('Unterminated Rust string');
    const word = /^(?:r#)?[a-zA-Z_][\w]*|^'[a-zA-Z_]\w*/.exec(rest);
    if (word) { out.push(word[0].replace(/^r#/, '')); i += word[0].length; }
    else { out.push(source[i]); i++; }
  }
  return out;
}

export function rustMutatingExports(source: string): string[] {
  const t = tokens(source);
  const pairs = new Map<number, number>();
  const stack: number[] = [];
  for (let i = 0; i < t.length; i++) {
    if (['(', '[', '{'].includes(t[i])) stack.push(i);
    else if ([')', ']', '}'].includes(t[i])) {
      const start = stack.pop();
      if (start === undefined || '([{'.indexOf(t[start]) !== ')]}'.indexOf(t[i])) {
        throw new Error('Unbalanced Rust delimiters');
      }
      pairs.set(start, i);
    }
  }
  if (stack.length) throw new Error('Unbalanced Rust delimiters');
  const out: string[] = [];
  function scan(start: number, end: number, exportedImpl = false): void {
    let attrs: string[][] = [];
    for (let i = start; i < end;) {
      if (t[i] === '#' && t[i + 1] === '[') {
        const close = pairs.get(i + 1)!;
        attrs.push(t.slice(i + 2, close));
        i = close + 1;
        continue;
      }
      const bindgen = attrs.find(a => a[0] === 'wasm_bindgen');
      if (attrs.some(a => a[0] !== 'wasm_bindgen' && a.includes('wasm_bindgen'))) {
        throw new Error('Unsupported wrapped wasm_bindgen attribute');
      }
      if (t[i] === 'impl') {
        let body = i + 1;
        while (body < end && t[body] !== '{') body++;
        if (body === end) throw new Error('Missing Rust impl body');
        scan(body + 1, pairs.get(body)!, Boolean(bindgen));
        i = pairs.get(body)! + 1;
        attrs = [];
        continue;
      }
      if (t[i] === 'mod' && t[i + 2] === '{') {
        scan(i + 3, pairs.get(i + 2)!);
        i = pairs.get(i + 2)! + 1;
        attrs = [];
        continue;
      }
      let fn = i;
      const publicMethod = t[fn] === 'pub';
      if (publicMethod) fn++;
      while (['async', 'unsafe', 'const'].includes(t[fn])) fn++;
      if (exportedImpl && publicMethod && t[fn] !== 'fn') {
        throw new Error('Unsupported public wasm-bindgen impl item');
      }
      if (t[fn] === 'fn') {
        const name = t[fn + 1];
        let args = fn + 2;
        while (args < end && t[args] !== '(' && t[args] !== '{') args++;
        if (t[args] !== '(') throw new Error(`Missing Rust arguments: ${name}`);
        const close = pairs.get(args)!;
        const receiver = t.slice(args + 1, close).join(' ').split(',')[0].trim();
        if (exportedImpl && publicMethod && !bindgen?.includes('skip')) {
          if (/\bself\b/.test(receiver) && /\bmut\b/.test(receiver)) {
            if (!/^& (?:'\w+ )?mut self$/.test(receiver)) {
              throw new Error(`Unsupported mutable receiver: ${name}: ${receiver}`);
            }
            const rename = bindgen?.indexOf('js_name') ?? -1;
            let exportedName = name;
            if (rename >= 0) {
              if (bindgen![rename + 1] !== '=') throw new Error(`Invalid js_name: ${name}`);
              const value = bindgen![rename + 2];
              exportedName = value?.startsWith('"') ? JSON.parse(value) : value;
              if (!exportedName || !/^[A-Za-z_$][\w$]*$/.test(exportedName)) {
                throw new Error(`Unsupported js_name: ${name}`);
              }
            }
            out.push(exportedName);
          }
        }
        i = close + 1;
        while (i < end && t[i] !== '{' && t[i] !== ';') i++;
        i = t[i] === '{' ? pairs.get(i)! + 1 : i + 1;
        attrs = [];
        continue;
      }
      if (pairs.has(i)) i = pairs.get(i)! + 1;
      else i++;
      attrs = [];
    }
  }
  scan(0, t.length);
  if (!out.length) throw new Error('No mutable wasm-bindgen exports found');
  return out;
}
