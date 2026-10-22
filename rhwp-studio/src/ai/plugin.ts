/**
 * AI 도우미 플러그인 — studio 번들에 들어 있는 첫 내장 플러그인.
 *
 * 경계: 문서는 `PluginHost` 로만 읽는다(`read`·`getSelectionText`). WASM·`InputHandler`·
 * `WasmBridge` 를 import 하지 않는다. 화면 부품(i18n, 대화상자 틀)은 studio 의 것을 쓴다 —
 * 같은 번들 안의 플러그인이라 따로 만들면 테마·키보드 계약이 갈리기 때문이다.
 *
 * `main.ts` 는 `__RHWP_AI__` 상수 분기 안에서만 이 모듈을 동적 import 한다.
 * `RHWP_WITHOUT_AI=1` 빌드에는 이 청크가 남지 않는다.
 *
 * 계획: `mydocs/plans/rhwp_studio_ai_chat_p0_impl.md`
 */
import type { PluginHost, StudioPlugin } from '@/plugin/types';
import { PLUGIN_API_VERSION } from '@/plugin/types';
import { t } from '@/i18n/index.ts';
import { AiPanel } from './ui/ai-panel';

export const AI_PLUGIN_ID = 'ai-assistant';
export const AI_PANEL_ID = 'chat';
export const AI_TOGGLE_COMMAND = 'ext:ai.toggle-panel';

export const aiAssistantPlugin: StudioPlugin = {
  id: AI_PLUGIN_ID,
  apiVersion: PLUGIN_API_VERSION,

  activate(host: PluginHost) {
    const panel = new AiPanel();

    host.ui.registerSidePanel({
      id: AI_PANEL_ID,
      title: t('ai.panel.title'),
      defaultWidth: 380,
      mount: (container) => panel.mount(container),
      unmount: () => panel.dispose(),
      onVisibilityChange: (open) => panel.setVisible(open),
    });

    host.automation.registerCommand({
      id: AI_TOGGLE_COMMAND,
      label: t('ai.command.togglePanel'),
      execute: () => { host.ui.toggleSidePanel(AI_PANEL_ID); },
    });
    host.automation.addMenuItem({ menuId: 'tool', commandId: AI_TOGGLE_COMMAND, position: 'top' });

    return {
      /** 패널 열림 여부 — e2e 가 화면 대신 상태로 확인하는 자리 */
      isPanelOpen: () => host.ui.isSidePanelOpen(AI_PANEL_ID),
      togglePanel: (open?: boolean) => host.ui.toggleSidePanel(AI_PANEL_ID, open),
      /** 도구가 읽을 선택 평문 — e2e 가 호스트 계약을 실제 문서로 확인하는 자리 */
      selectionText: () => host.getSelectionText(),
    };
  },
};
