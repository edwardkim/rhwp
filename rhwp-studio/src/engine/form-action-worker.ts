import { evaluateFormAction } from '../../../rhwp-shared/forms/action-runtime.ts';

self.onmessage = ({ data }) => {
  try { self.postMessage({ result: evaluateFormAction(data) }); }
  catch { self.postMessage({ error: 'FORM_ACTION_SCRIPT: 처리할 수 없는 명령이거나 실행 한도를 초과했습니다.' }); }
};
