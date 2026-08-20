/**
 * 첫 실행 스킨 선택 대화상자.
 *
 * 스킨을 한 번도 직접 고르지 않은 사용자(theme.skinChosen=false)에게 첫 실행 시
 * 클래식/모던/올드스쿨 카드를 보여준다. 카드를 고르면 즉시 적용·저장되어 라이브
 * 미리보기가 되고, [시작하기]로 닫는다. 닫기(×/Escape)만 해도 다시 묻지 않도록
 * 선택 확정(markSkinChosen)으로 처리한다 — 이후 변경은 보기 > 테마 메뉴에서 한다.
 */
import { ModalDialog } from './dialog';
import { setThemeSkin, syncThemeMenu } from '../core/theme';
import { shouldShowSkinOnboarding, userSettings, type ThemeSkin } from '../core/user-settings';

interface SkinChoice {
  skin: ThemeSkin;
  name: string;
  description: string;
  /** 카드 미니 프리뷰 색상 [크롬, 액센트, 작업영역] */
  swatch: [string, string, string];
}

const SKIN_CHOICES: readonly SkinChoice[] = [
  {
    skin: 'default',
    name: '클래식',
    description: '기존 rhwp-studio 기본 모습',
    swatch: ['#f0f0f0', '#6182d6', '#e0e0e0'],
  },
  {
    skin: 'flat',
    name: '모던',
    description: '화이트 크롬과 블루 액센트의 플랫 스킨',
    swatch: ['#ffffff', '#2b7de9', '#eef0f4'],
  },
  {
    skin: 'oldschool',
    name: '올드스쿨',
    description: '한글 97 시절의 고전 데스크톱 감성',
    swatch: ['#c0c0c0', '#000080', '#808080'],
  },
];

class SkinOnboardingDialog extends ModalDialog {
  constructor() {
    super('화면 스킨 선택', 460);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'skin-onboarding-body';

    const intro = document.createElement('p');
    intro.className = 'skin-onboarding-intro';
    intro.textContent =
      '마음에 드는 화면 스킨을 골라 시작하세요. 언제든 보기 > 테마에서 바꿀 수 있습니다.';
    body.appendChild(intro);

    const cards = document.createElement('div');
    cards.className = 'skin-onboarding-cards';
    cards.setAttribute('role', 'radiogroup');
    cards.setAttribute('aria-label', '화면 스킨');

    const currentSkin = userSettings.getThemeSettings().skin;
    for (const choice of SKIN_CHOICES) {
      cards.appendChild(this.createCard(choice, choice.skin === currentSkin));
    }
    body.appendChild(cards);
    return body;
  }

  private createCard(choice: SkinChoice, selected: boolean): HTMLElement {
    const card = document.createElement('button');
    card.type = 'button';
    card.className = 'skin-onboarding-card';
    card.dataset.skinChoice = choice.skin;
    card.setAttribute('role', 'radio');
    card.setAttribute('aria-checked', String(selected));
    if (selected) card.classList.add('selected');

    const preview = document.createElement('span');
    preview.className = 'skin-onboarding-preview';
    preview.setAttribute('aria-hidden', 'true');
    const [chrome, accent, workspace] = choice.swatch;
    preview.style.background = workspace;
    const previewChrome = document.createElement('span');
    previewChrome.className = 'skin-onboarding-preview-chrome';
    previewChrome.style.background = chrome;
    const previewAccent = document.createElement('span');
    previewAccent.className = 'skin-onboarding-preview-accent';
    previewAccent.style.background = accent;
    previewChrome.appendChild(previewAccent);
    preview.appendChild(previewChrome);

    const name = document.createElement('span');
    name.className = 'skin-onboarding-name';
    name.textContent = choice.name;

    const description = document.createElement('span');
    description.className = 'skin-onboarding-description';
    description.textContent = choice.description;

    card.appendChild(preview);
    card.appendChild(name);
    card.appendChild(description);

    card.addEventListener('click', () => {
      // 즉시 적용 + 저장 = 라이브 미리보기이자 최종 선택.
      setThemeSkin(choice.skin);
      syncThemeMenu();
      for (const other of this.dialog.querySelectorAll<HTMLElement>('.skin-onboarding-card')) {
        const active = other === card;
        other.classList.toggle('selected', active);
        other.setAttribute('aria-checked', String(active));
      }
    });

    return card;
  }

  protected onConfirm(): void {
    userSettings.markSkinChosen();
  }

  override hide(): void {
    // ×/Escape 로 닫아도 다시 묻지 않는다 — 현재 스킨을 그대로 확정.
    userSettings.markSkinChosen();
    super.hide();
  }

  override show(): void {
    super.show();
    const okBtn = this.dialog.querySelector<HTMLButtonElement>('.dialog-btn-primary');
    if (okBtn) okBtn.textContent = '시작하기';
    // 선택은 카드가 담당하므로 취소 버튼은 숨긴다 (닫기 = 현재 스킨 유지).
    const cancelBtn = this.dialog.querySelector<HTMLButtonElement>(
      '.dialog-footer .dialog-btn:not(.dialog-btn-primary)',
    );
    if (cancelBtn) cancelBtn.style.display = 'none';
  }
}

/** 필요할 때(첫 실행) 스킨 선택 안내를 띄운다. 표시했으면 true. */
export function maybeShowSkinOnboarding(): boolean {
  if (!shouldShowSkinOnboarding(userSettings.getThemeSettings())) return false;
  // 임베드/브리지 모드(크롬 숨김, iframe 호스팅)에서는 호스트 UX 를 방해하지 않는다.
  const root = document.getElementById('studio-root');
  const chromeSuppressed = !!root && [...root.classList].some((c) => c.startsWith('rhwp-chrome-no-'));
  const embedded = window.parent !== window;
  if (chromeSuppressed || embedded) return false;
  new SkinOnboardingDialog().show();
  return true;
}
