import type { CommandServices } from '@/command/types';
import type { DocumentPosition } from '@/core/types';
import type { InputHandler } from '@/engine/input-handler';

/**
 * [Task #2370 클러스터 C] 다이얼로그 [확인]의 문서 변경을 편집 라우터로 보내는 공용 경로.
 *
 * Track 1(#2362·#2364·#2368)에서 다이얼로그 여섯을 `executeOperation({kind:'snapshot'})`
 * 으로 이관할 때 실패 처리가 제각각으로 남았다 — 어떤 것은 `try/catch` 로 삼키고
 * 다이얼로그를 닫았고(다단·새 번호), 어떤 것은 가드 없이 throw 를 전파했다(미주 모양·
 * 편집 용지·구역·쪽 테두리). 같은 조작이 실패했을 때 사용자가 보는 것이 달라진다.
 *
 * 여기서 하나로 정한다: **실패하면 경고를 남기고 다이얼로그를 열어 둔다.** 사용자가
 * 입력한 값을 잃지 않고 다시 시도할 수 있다(`ModalDialog.onConfirm` 이 `false` 를
 * 받으면 닫지 않는다 — `dialog.ts` 의 `if (shouldClose !== false) this.hide()`).
 * 종전 throw 전파도 결과적으로 열린 채였지만 처리되지 않은 예외를 남겼다.
 *
 * `operation` 은 문서를 바꾸지 않았으면 `null` 을 반환해 기록을 취소한다([Task #2370]
 * 클러스터 A 의 공용 no-op 신호). `services` 미주입 환경(구 호출부)에서는 `fallback` 을
 * 그대로 쓴다.
 *
 * @returns 적용에 성공했으면 `true`. 그대로 `onConfirm` 의 반환값으로 쓰면 된다.
 */
export function applyThroughRouter(options: {
  services: CommandServices | undefined;
  /** 로그 접두어 — 실패 원인을 찾을 수 있게 커맨드/다이얼로그를 식별한다. */
  label: string;
  operationType: string;
  operation: (ih: InputHandler) => DocumentPosition | null;
  /** 라우터에 도달하지 못하는 환경의 직접 적용 경로. */
  fallback: () => void;
}): boolean {
  const { services, label, operationType, operation, fallback } = options;
  try {
    const ih = services?.getInputHandler();
    if (ih) {
      ih.executeOperation({ kind: 'snapshot', operationType, operation: () => operation(ih) });
    } else {
      fallback();
    }
    return true;
  } catch (err) {
    console.warn(`[${label}] 적용 실패:`, err);
    return false;
  }
}
