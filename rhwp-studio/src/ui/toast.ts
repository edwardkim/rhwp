/**
 * 우상단 슬라이드 토스트 알림 (#196).
 *
 * - 슬라이드 인 + 자동 페이드 (기본 8초)
 * - 사용자 닫기 버튼 (×)
 * - 선택적 액션 버튼 (텍스트 링크 스타일)
 * - 일반 재사용 가능 — 다른 안내에도 활용 가능
 */

const CONTAINER_ID = 'rhwp-toast-container';
const DEFAULT_DURATION_MS = 8000;
const SLIDE_DURATION_MS = 200;
const FADE_DURATION_MS = 350;

export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface ToastOptions {
  /** 메시지 본문 (필수). \n 줄바꿈 지원. */
  message: string;
  /** 자동 페이드 시간 (ms). 기본 8000ms. 0 이면 자동 페이드 없음 (사용자 닫기만). */
  durationMs?: number;
  /** 액션 버튼 (선택, 텍스트 링크 스타일). */
  action?: ToastAction;
  /**
   * 확인 버튼 라벨 (선택). 지정 시 우측에 명시적 확인 버튼 추가.
   * 자동 페이드를 끄고 사용자 닫기를 강제하는 용도 (durationMs: 0 와 함께 사용).
   */
  confirmLabel?: string;
}

function ensureContainer(): HTMLElement {
  let container = document.getElementById(CONTAINER_ID);
  if (container) return container;

  container = document.createElement('div');
  container.id = CONTAINER_ID;
  container.style.position = 'fixed';
  container.style.top = '16px';
  container.style.right = '16px';
  container.style.zIndex = '21000';  // 모달 (10000~20000) 보다 위
  container.style.display = 'flex';
  container.style.flexDirection = 'column';
  container.style.gap = '8px';
  container.style.pointerEvents = 'none';  // 토스트 외 영역은 통과
  document.body.appendChild(container);
  return container;
}

/**
 * 토스트 알림을 표시한다.
 *
 * @param options 메시지·지속시간·액션
 */
export function showToast(options: ToastOptions): void {
  const container = ensureContainer();
  const duration = options.durationMs ?? DEFAULT_DURATION_MS;

  const toast = document.createElement('div');
  toast.style.background = '#1e293b';
  toast.style.color = '#f1f5f9';
  toast.style.padding = '12px 14px';
  toast.style.borderRadius = '6px';
  toast.style.boxShadow = '0 4px 12px rgba(0, 0, 0, 0.2)';
  toast.style.font = '13px/1.5 sans-serif';
  toast.style.maxWidth = '400px';
  toast.style.minWidth = '280px';
  toast.style.display = 'flex';
  toast.style.alignItems = 'flex-start';
  toast.style.gap = '12px';
  toast.style.transform = 'translateX(120%)';
  toast.style.transition = `transform ${SLIDE_DURATION_MS}ms ease-out, opacity ${FADE_DURATION_MS}ms ease-out`;
  toast.style.opacity = '1';
  toast.style.pointerEvents = 'auto';
  toast.setAttribute('role', 'status');
  toast.setAttribute('aria-live', 'polite');

  // 본문
  const body = document.createElement('div');
  body.style.flex = '1';
  body.style.whiteSpace = 'pre-line';
  body.textContent = options.message;
  toast.appendChild(body);

  // 액션 버튼 (선택, 텍스트 링크 스타일)
  if (options.action) {
    const actionBtn = document.createElement('button');
    actionBtn.type = 'button';
    actionBtn.textContent = options.action.label;
    actionBtn.style.background = 'transparent';
    actionBtn.style.color = '#60a5fa';
    actionBtn.style.border = 'none';
    actionBtn.style.cursor = 'pointer';
    actionBtn.style.padding = '0';
    actionBtn.style.font = 'inherit';
    actionBtn.style.textDecoration = 'underline';
    actionBtn.style.flexShrink = '0';
    actionBtn.addEventListener('click', () => {
      options.action!.onClick();
      // 액션 클릭 시 토스트 자동 닫지 않음 — confirmLabel 가 있으면 사용자가 명시적으로 닫음
      if (!options.confirmLabel) removeToast();
    });
    toast.appendChild(actionBtn);
  }

  // 확인 버튼 (선택, 강조 스타일) 또는 닫기 버튼 (×)
  if (options.confirmLabel) {
    const confirmBtn = document.createElement('button');
    confirmBtn.type = 'button';
    confirmBtn.textContent = options.confirmLabel;
    confirmBtn.style.background = '#2563eb';
    confirmBtn.style.color = '#ffffff';
    confirmBtn.style.border = 'none';
    confirmBtn.style.borderRadius = '4px';
    confirmBtn.style.cursor = 'pointer';
    confirmBtn.style.padding = '4px 12px';
    confirmBtn.style.font = 'inherit';
    confirmBtn.style.flexShrink = '0';
    confirmBtn.addEventListener('click', () => removeToast());
    toast.appendChild(confirmBtn);
  } else {
    const closeBtn = document.createElement('button');
    closeBtn.type = 'button';
    closeBtn.setAttribute('aria-label', '닫기');
    closeBtn.textContent = '×';
    closeBtn.style.background = 'transparent';
    closeBtn.style.color = '#94a3b8';
    closeBtn.style.border = 'none';
    closeBtn.style.cursor = 'pointer';
    closeBtn.style.padding = '0';
    closeBtn.style.font = '20px/1 sans-serif';
    closeBtn.style.flexShrink = '0';
    closeBtn.style.lineHeight = '1';
    closeBtn.addEventListener('click', () => removeToast());
    toast.appendChild(closeBtn);
  }

  container.appendChild(toast);

  // 슬라이드 인 (다음 프레임에서 transform 변경)
  requestAnimationFrame(() => {
    toast.style.transform = 'translateX(0)';
  });

  let removed = false;
  let timer: ReturnType<typeof setTimeout> | null = null;

  function removeToast(): void {
    if (removed) return;
    removed = true;
    if (timer) clearTimeout(timer);
    toast.style.opacity = '0';
    toast.style.transform = 'translateX(120%)';
    setTimeout(() => {
      toast.remove();
    }, FADE_DURATION_MS);
  }

  // 자동 페이드
  if (duration > 0) {
    timer = setTimeout(removeToast, duration);
  }
}
