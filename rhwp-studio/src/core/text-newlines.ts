/** 코어와 같은 LF 입력을 커서·문자 수·Undo가 함께 소비한다. */
export function normalizeTextNewlines(text: string): string {
  return text.includes('\r') ? text.replace(/\r\n?/g, '\n') : text;
}
