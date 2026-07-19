---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# iOS Safari/Chrome 한글 IME 조합 문제 해결

> 2026-04-01 | Task #22 모바일 대응 중 발견

---

## 증상

iOS Safari/Chrome에서 한글 입력 시 자모가 분리되어 입력됨.
- 기대: "가나다"
- 실제: "ㄱㅏㄴㅏㄷㅏ" 또는 "ㄱ가간가나낟나다"

데스크톱(Windows/Mac Chrome)에서는 정상 동작.

## 원인

### 1. iOS WebKit은 hidden textarea에서 composition 이벤트를 발생시키지 않음

```
Desktop: compositionstart → compositionupdate → compositionend (정상)
iOS:     composition 이벤트 없음, input(insertText)만 발생
```

- `<textarea>`가 `opacity:0`, `width:1px`, `position:fixed;left:-9999px` 등으로 숨겨져 있으면
  iOS WebKit이 해당 요소를 "가상 키보드 불필요"로 판단하여 IME composition을 비활성화
- `compositionstart`, `compositionupdate`, `compositionend` 이벤트가 아예 발생하지 않음
- 한글 자모가 개별 `insertText`로 전달되어 조합 불가

### 2. iOS의 실제 한글 조합 패턴

iOS는 `<div contentEditable>`에서 한글 조합을 **deleteContentBackward + insertText 쌍**으로 처리:

```
"가나다" 입력 시 iOS가 발생시키는 이벤트:
  insertText "ㄱ"      → value="ㄱ"
  deleteBackward       → value=""
  insertText "가"      → value="가"
  deleteBackward       → value=""
  insertText "간"      → value="간"
  deleteBackward       → value=""
  insertText "가나"    → value="가나"    ← "간"→"가"+"나" 분리
  deleteBackward       → value="가"
  insertText "낟"      → value="가낟"
  deleteBackward       → value="가"
  insertText "나다"    → value="가나다"  ← "낟"→"나"+"다" 분리
```

핵심: iOS가 div의 textContent를 **자체적으로 완벽하게 관리**한다.
우리는 매 input마다 div의 value를 문서에 반영하기만 하면 된다.

## 해결 방법

### 1단계: iOS에서 contentEditable div 사용

`<textarea>` 대신 `<div contentEditable>`을 사용해야 iOS에서 IME가 동작한다.

```typescript
const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) ||
  (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1);

if (isIOS) {
  const div = document.createElement('div');
  div.contentEditable = 'true';
  div.style.cssText =
    'position:absolute;left:0;top:0;width:2em;height:1.5em;' +
    'color:transparent;background:transparent;caret-color:transparent;' +
    'border:none;outline:none;overflow:hidden;white-space:nowrap;' +
    'z-index:10;font-size:16px;padding:0;margin:0;';
  // value 프록시로 textarea 인터페이스 호환
  Object.defineProperty(div, 'value', {
    get() { return div.textContent || ''; },
    set(v: string) { div.textContent = v; },
  });
  this.textarea = div as unknown as HTMLTextAreaElement;
} else {
  // 데스크톱: 기존 hidden textarea 유지
  this.textarea = document.createElement('textarea');
  this.textarea.style.cssText = 'position:fixed;left:-9999px;top:0;width:1px;height:1px;opacity:0;';
}
```

### 2단계: iOS 폴백 조합 처리 (핵심)

iOS에서는 `isComposing`이 항상 `false`이므로, **앵커 + 길이** 방식으로 이전 삽입을 교체한다:

```typescript
if (this._isIOS && !this.isComposing) {
  // 앵커 설정 (첫 입력 시)
  if (!this._iosAnchor) {
    this._iosAnchor = this.cursor.getPosition();
    this._iosLength = 0;
  }

  // 이전 삽입 삭제
  if (this._iosLength > 0) {
    this.deleteTextAt(this._iosAnchor, this._iosLength);
  }

  // 현재 value 전체를 재삽입
  const text = this.textarea.value;
  if (text) {
    this.insertTextAtRaw(this._iosAnchor, text);
    this._iosLength = text.length;
  } else {
    this._iosLength = 0;
  }

  // 렌더링 디바운스 (중요!)
  clearTimeout(this._iosInputTimer);
  this._iosInputTimer = setTimeout(() => {
    this.afterEdit();
    this.textarea.focus();
  }, 100);
  return;
}
```

### 3단계: afterEdit() 디바운스 (가장 중요)

**`afterEdit()`를 매 input마다 호출하면 안 된다.** `afterEdit()`가 `document-changed` 이벤트를 발생시켜 Canvas를 재렌더링하는데, 이 과정에서 contentEditable div의 focus/textContent가 교란된다.

iOS는 `deleteBackward + insertText`를 10ms 이내 연속 발생시키므로, 중간에 렌더링이 끼면 div 상태가 깨진다.

```
잘못된 방식:
  input("ㄱ") → afterEdit() → Canvas 재렌더링 → div 교란
  input("가") → afterEdit() → Canvas 재렌더링 → div 교란
  → 결과: "ㄱ가"

올바른 방식 (100ms 디바운스):
  input("ㄱ") → 문서만 갱신 (렌더링 없음)
  input("가") → 문서만 갱신 (렌더링 없음)
  ... 100ms 후 ...
  afterEdit() → Canvas 렌더링 1회
  → 결과: "가"
```

## 시행착오 기록

| 시도 | 결과 | 원인 |
|------|------|------|
| textarea left:-9999px → left:0 | 실패 | iOS가 여전히 composition 미발생 |
| textarea opacity:0 → color:transparent | 실패 | textarea 자체가 iOS IME 미지원 |
| contentEditable div 사용 | 부분 성공 | composition 발생하지만 value 교란 |
| div value를 매번 초기화 | 실패 | iOS가 리셋 후 새로운 조합으로 인식 |
| div value를 건드리지 않음 | 부분 성공 | afterEdit()가 div를 교란 |
| **afterEdit() 디바운스 100ms** | **성공** | iOS의 연속 이벤트를 방해하지 않음 |

## 참고

- Google Docs도 동일한 "Shadow Input + 디바운스" 패턴 사용
- iOS Safari는 `<textarea>`보다 `<div contentEditable>`에서 IME가 안정적
- `font-size:16px` 필수 — 미만이면 iOS가 화면을 자동 확대
- `inputmode="text"` 설정으로 한글 키보드 보장
- 1초 무입력 후 div 초기화 + 앵커 리셋 (다음 입력은 새 위치에서 시작)
