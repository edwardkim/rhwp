// 별도 PR에서 검사를 다시 추가할 문서를 회귀 fixture 수집에서 제외한다.
import {readFileSync} from 'node:fs';

const names = new Set(readFileSync(new URL('../../tests/fixtures/deferred_regression_documents.txt', import.meta.url), 'utf8')
  .split(/\r?\n/).filter(line => line && !line.startsWith('#')));

export function isDeferredRegressionInput(path) {
  const name = String(path).replaceAll('\\', '/').split('/').at(-1);
  return [...names].some(value => value.endsWith('*') ? name.startsWith(value.slice(0, -1)) : name === value);
}
