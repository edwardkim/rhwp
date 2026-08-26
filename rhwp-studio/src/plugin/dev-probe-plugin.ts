/**
 * 개발용 시험 플러그인 — 플러그인 계약 자체를 e2e 로 재기 위한 것.
 *
 * **개발 빌드에서만 등록된다**(`main.ts` 의 `import.meta.env.DEV` 게이트). 프로덕션 번들에는
 * 들어가지 않으므로, 이 파일이 있다고 해서 studio 의 배포 표면이 넓어지지 않는다.
 *
 * 하는 일은 계약을 밟아 보는 것뿐이다 — 트랜잭션 1건이 undo 1스텝인가, 중첩이 거절되는가,
 * unload 가 등록물을 걷어가는가.
 */
import type { PluginHost, StudioPlugin } from './types';
import { PLUGIN_API_VERSION } from './types';

export const devProbePlugin: StudioPlugin = {
  id: 'dev-probe',
  apiVersion: PLUGIN_API_VERSION,

  activate(host: PluginHost) {
    let swapCount = 0;
    host.onDocumentSwap(() => { swapCount += 1; });

    return {
      /** 살아 있는지 확인 */
      ping: () => 'pong',

      /** studio 소유 경로로 빈 문서로 교체한다. swap 알림 단일 전달 회귀용이다. */
      replaceWithBlank: () => {
        host.createBlankDocument();
        return true;
      },

      /** 읽기 — 히스토리를 건드리지 않는다 */
      pageCount: () => host.read((doc) => JSON.parse(doc.getDocumentInfo()).pageCount as number),

      /**
       * 트랜잭션 1건으로 문단 한 곳에 여러 번 쓴다.
       * undo 1회로 전부 되돌아가야 한다 — 그것이 "배치 = undo 1스텝" 의 판정이다.
       */
      appendRuns: (text: string, times: number) => host.transaction('appendRuns', (tx) => {
        const doc = tx.doc();
        tx.deferPagination();
        let written = 0;
        for (let i = 0; i < times; i += 1) {
          doc.insertText(0, 0, 0, text);
          written += text.length;
        }
        return written;
      }),

      /** 중첩 트랜잭션 — `NESTED_TX` 로 거절되어야 한다 */
      nestedTransaction: () => host.transaction('outer', () =>
        host.transaction('inner', () => 'should not reach')),

      /** 본문이 던지면 롤백된다 — 문서는 진입 시점 그대로여야 한다 */
      throwingTransaction: (text: string) => host.transaction('throwing', (tx) => {
        tx.doc().insertText(0, 0, 0, text);
        throw new Error('의도된 실패');
      }),

      /** 자동화 표면 사용 — unload 시 원장이 걷어가는지 본다 */
      addProbeCommand: () => {
        host.automation.registerCommand({
          id: 'ext:dev-probe',
          label: '개발 시험 커맨드',
          execute: () => { /* noop */ },
        });
        host.automation.addMenuItem({ menuId: 'tool', commandId: 'ext:dev-probe' });
        return true;
      },

      /** 이벤트 구독 — 역시 원장 대상 */
      subscribeProbe: () => {
        host.events.on('command-state-changed', () => { /* noop */ });
        return true;
      },

      documentSwapCount: () => swapCount,
    };
  },
};
