export type DocumentErrorType = 'line-break' | 'page-count' | 'paint';

/** Render one CLI-stable document error as `type: [flat document attributes]`. */
export function formatDocumentError(
  type: DocumentErrorType,
  attributes: readonly (readonly [name: string, value: string | number])[],
): string {
  const body = attributes.map(([name, value]) => {
    if (!/^[a-z][a-zA-Z]*$/.test(name)) throw new Error(`invalid document error attribute: ${name}`);
    const rendered = String(value);
    if (!/^[\x21-\x5a\x5c\x5e-\x7e]+$/.test(rendered)) {
      throw new Error(`invalid document error value: ${name}`);
    }
    return `${name}=${rendered}`;
  }).join(' ');
  return `${type}: [${body}]`;
}
