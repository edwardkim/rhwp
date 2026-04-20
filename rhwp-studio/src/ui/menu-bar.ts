import type { EventBus } from '@/core/event-bus';
import type { CommandDispatcher } from '@/command/dispatcher';

/**
 * 메뉴바 드롭다운 컨트롤러
 *
 * - 메뉴 타이틀 클릭 → 드롭다운 토글
 * - 열린 상태에서 다른 메뉴 hover → 자동 전환
 * - 바깥 클릭 / Escape → 닫기
 * - 항목 클릭 → CommandDispatcher 경유 실행
 * - 드롭다운 열릴 때 컨텍스트 감응 활성/비활성 갱신
 */
export class MenuBar {
  private menuItems: HTMLElement[];
  private openMenu: HTMLElement | null = null;

  constructor(
    private container: HTMLElement,
    private eventBus: EventBus,
    private dispatcher: CommandDispatcher,
  ) {
    this.menuItems = Array.from(container.querySelectorAll('.menu-item'));
    this.setupTitleClicks();
    this.setupTitleHover();
    this.setupItemClicks();
    this.setupOutsideClose();
    this.setupKeyboardClose();
  }

  /** 메뉴 타이틀 클릭 → 드롭다운 토글 */
  private setupTitleClicks(): void {
    for (const item of this.menuItems) {
      const title = item.querySelector('.menu-title') as HTMLElement;
      if (!title) continue;
      title.addEventListener('mousedown', (e) => {
        e.preventDefault();
        if (this.openMenu === item) {
          this.closeAll();
        } else {
          this.openMenu?.classList.remove('open');
          item.classList.add('open');
          this.openMenu = item;
          this.updateMenuStates(item);
        }
      });
    }
  }

  /** 열린 상태에서 다른 메뉴 hover → 자동 전환 */
  private setupTitleHover(): void {
    for (const item of this.menuItems) {
      const title = item.querySelector('.menu-title') as HTMLElement;
      if (!title) continue;
      title.addEventListener('mouseenter', () => {
        if (this.openMenu && this.openMenu !== item) {
          this.openMenu.classList.remove('open');
          item.classList.add('open');
          this.openMenu = item;
          this.updateMenuStates(item);
        }
      });
    }
  }

  /** 드롭다운 항목 클릭 → 커맨드 디스패치 + 닫기 */
  private setupItemClicks(): void {
    this.container.addEventListener('click', (e) => {
      const target = (e.target as HTMLElement).closest('.md-item') as HTMLElement;
      if (!target) return;
      if (target.classList.contains('disabled')) return;

      const cmd = target.dataset.cmd;
      if (cmd) {
        // data-* 속성을 params로 변환 (data-cmd 제외)
        const params: Record<string, unknown> = { anchorEl: target };
        for (const [key, val] of Object.entries(target.dataset)) {
          if (key !== 'cmd') params[key] = val;
        }
        this.dispatcher.dispatch(cmd, params);
      }
      this.closeAll();
    });
  }

  /** 바깥 클릭 → 닫기 */
  private setupOutsideClose(): void {
    document.addEventListener('mousedown', (e) => {
      if (!this.openMenu) return;
      if (!this.container.contains(e.target as Node)) {
        this.closeAll();
      }
    });
  }

  /** Escape 키 → 닫기 */
  private setupKeyboardClose(): void {
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && this.openMenu) {
        this.closeAll();
      }
    });
  }

  /** 드롭다운 열릴 때 항목별 활성/비활성 상태를 컨텍스트 기반으로 갱신 */
  private updateMenuStates(menuElement: HTMLElement): void {
    // 일반 항목
    const items = menuElement.querySelectorAll('.md-item[data-cmd]');
    for (const item of items) {
      const el = item as HTMLElement;
      const cmdId = el.dataset.cmd!;
      const enabled = this.dispatcher.isEnabled(cmdId);
      el.classList.toggle('disabled', !enabled);
      // #196: file:save 가 비활성이면 베타 안내 툴팁 표시
      if (cmdId === 'file:save' && !enabled) {
        el.title = 'HWPX 직접 저장은 현재 베타 단계로 비활성화되어 있습니다. 다음 업데이트에서 지원 예정입니다.';
      } else if (cmdId === 'file:save') {
        el.removeAttribute('title');
      }
    }
    // 서브메뉴 컨테이너: 하위 항목 중 활성이 하나라도 있으면 서브메뉴도 활성
    const subs = menuElement.querySelectorAll('.md-sub');
    for (const sub of subs) {
      const subItems = sub.querySelectorAll('.md-item[data-cmd]');
      let anyEnabled = false;
      for (const si of subItems) {
        if (!si.classList.contains('disabled')) {
          anyEnabled = true;
          break;
        }
      }
      sub.classList.toggle('disabled', !anyEnabled);
    }
  }

  private closeAll(): void {
    this.openMenu?.classList.remove('open');
    this.openMenu = null;
  }
}
