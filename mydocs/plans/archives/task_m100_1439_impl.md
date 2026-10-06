# Task M100 #1439 구현계획서 — 드래그&드롭 보안 게이트 + 모달 확인 (확장/웹 공통)

- 이슈: #1439, 마일스톤 M100, 브랜치 `local/task1439`
- 작성일: 2026-06-19
- 수행계획서: `mydocs/plans/task_m100_1439.md`

## 0. 크롬 확장 모드 동작 보장 (작업지시자 요구)

rhwp-studio 는 **PWA + 크롬 확장 양쪽 배포**다. 드롭 보안 게이트가 양쪽에서 동작해야 함.

- 드롭 핸들러·대화상자는 모두 `main.ts` 의 같은 페이지 컨텍스트에서 동작. 확장 모드도
  `/rhwp/` 를 **전체 탭(standalone) 페이지**로 열므로(`manifest` display=standalone,
  file_handlers), popup 제약 없이 `ModalDialog`(DOM 오버레이) 정상 렌더.
- 드롭 보안 게이트는 확장/웹 **공통 경로** — 별도 분기 불필요. `chrome` API 의존 없음
  (대화상자는 순수 DOM). 기존 `typeof chrome !== 'undefined'` 옵셔널 패턴과 무충돌.
- 확장 모드 file:// 권한(#1131, `isAllowedFileSchemeAccess`)은 **드롭과 무관** — 드롭은
  `dataTransfer.files`(File 객체)라 file scheme 권한이 필요 없다. 게이트는 그 위 계층.
- **검증**: e2e(headless)로 드롭 게이트 + 확장 빌드(`npm run build`) 산출물에 동일 코드
  포함 확인. 확장 로드 수동 점검은 작업지시자 환경(선택).

## 1. 신규 — `drop-confirm-dialog.ts`

`unsaved-changes-dialog.ts` 동형 (`ModalDialog` 상속 + `showAsync(): Promise<boolean>`):

```ts
import { ModalDialog } from './dialog';

class DropConfirmDialog extends ModalDialog {
  private resolve!: (v: boolean) => void;
  constructor(private readonly fileName: string) {
    super('로컬 파일 열기 확인', 420);
  }
  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.cssText = 'padding:16px 20px;line-height:1.6;white-space:pre-line;';
    body.textContent =
      `드래그한 로컬 파일을 엽니다.\n\n"${this.fileName || '선택한 파일'}"\n\n` +
      `이 동작은 로컬 파일의 내용을 읽습니다. 계속하시겠습니까?`;
    return body;
  }
  protected onConfirm(): void { this.resolve(true); }      // [열기]
  override hide(): void { this.resolve(false); super.hide(); } // 취소/밖클릭/esc
  showAsync(): Promise<boolean> {
    return new Promise((resolve) => {
      let done = false;
      this.resolve = (v) => { if (!done) { done = true; resolve(v); } };
      super.show();
      const footer = this.dialog.querySelector('.dialog-footer');
      const okBtn = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
      const cancelBtn = footer?.querySelector('.dialog-btn:not(.dialog-btn-primary)') as HTMLButtonElement | null;
      if (okBtn) okBtn.textContent = '열기';
      if (cancelBtn) cancelBtn.textContent = '취소';
    });
  }
}

export function showDropConfirmDialog(fileName: string): Promise<boolean> {
  return new DropConfirmDialog(fileName).showAsync();
}
```

기본값 안전: 미동의(취소/밖클릭/esc) → `false` → 미로딩.

## 2. drop 핸들러 게이트 연결 (`main.ts:364~393`)

HWP/HWPX 분기에서 `loadFile` **호출 전** 확인:

```ts
if (!dropName.endsWith('.hwp') && !dropName.endsWith('.hwpx')) {
  alert('HWP/HWPX 파일 또는 이미지 파일만 지원합니다.');
  return;
}
// [#1439] 보안: 드롭 로컬 파일 로딩은 명시적 동의(opt-in) 후에만.
const confirmed = await showDropConfirmDialog(file.name);
if (!confirmed) return;          // 미동의 → 미로딩
await loadFile(file);            // 동의 → 이후 loadFile 내부 unsaved 가드
```

순서: **드롭 보안 확인 → loadFile 내부 unsaved 가드**. 사용자가 "이 파일 열기" 먼저
동의 → 그 다음 저장 안 된 변경 경고.

## 3. 범위 결정 — 이미지/파일 메뉴

- **HWP/HWPX 드롭**: 보안 게이트 적용 (본 이슈 핵심).
- **이미지 드롭**(`imageExts`): 이미지 "삽입"(문서 편집)으로 로컬 문서를 교체·읽기 로딩이
  아니라 현재 문서에 배치. 보안 결은 다르나, **일관성을 위해 동일 확인 게이트 적용**
  (드래그한 로컬 이미지를 삽입한다는 고지). 단순화 위해 같은 `showDropConfirmDialog` 재사용.
- **파일 메뉴/열기 버튼**: 사용자가 이미 명시적으로 트리거한 경로 → 게이트 **미적용**
  (loadFile 시그니처·기존 경로 불변).

## 4. 단계별 구현

### 1단계 — drop-confirm-dialog + drop 핸들러 게이트
- `drop-confirm-dialog.ts` 신규.
- `main.ts` drop 핸들러 HWP/HWPX(+이미지) 분기에 확인 게이트 연결.
- `npx tsc --noEmit` 통과.

### 2단계 — 빌드 + 확장/웹 공통 동작 + 기존 회귀
- `npm run build`(확장/PWA 산출물) 성공, 산출물에 게이트 포함 확인.
- 기존 e2e(파일 열기/unsaved-guard) 회귀 0.

### 3단계 — e2e + 보안 문서 + 보고서
- e2e 신규: 드롭 시뮬 → 대화상자 표시 / [열기] 로딩 / [취소] 미로딩.
- 보안 가이드/감사 문서 반영(`browser_extension_dev_guide.md`, `browser_extension_security_audit.md`).
- 단계별/최종 보고서.

## 5. 검증

- `npx tsc --noEmit`, `npm run build`.
- e2e(puppeteer headless): 드롭 게이트 3케이스.
- 기존 e2e 회귀 0 (특히 `unsaved-changes-guard.test.mjs`).
- 확장 빌드 산출물에 게이트 코드 포함 확인 (확장 수동 로드는 작업지시자 선택).

## 6. 위험과 대응

| 위험 | 대응 |
|------|------|
| 확장 popup 컨텍스트 모달 제약 | 확장도 standalone 탭(`/rhwp/`)이라 모달 정상. popup 미사용 |
| 드롭 확인 ↔ unsaved 순서 | 드롭 확인 먼저 → loadFile 내부 unsaved (2절) |
| 이미지 드롭 UX 변화 | 동일 게이트로 일관. 필요 시 메시지 분기 |
| e2e 드롭 시뮬 난도 | DataTransfer 합성 + drop 이벤트 디스패치 (기존 e2e 패턴) |
