# PR #819 검토

## 메타

| 항목 | 값 |
|------|---|
| PR | [#819](https://github.com/edwardkim/rhwp/pull/819) |
| 제목 | feat: Cmd+Backspace / Alt+Backspace 삭제 단축키 (#260) |
| 컨트리뷰터 | @oksure (Hyunwoo Park, 활발한 컨트리뷰터) |
| Fork | oksure/rhwp (`contrib/macos-delete-shortcuts`) |
| Base | devel |
| 규모 | +82/-7, 1 파일 (`rhwp-studio/src/engine/input-handler-keyboard.ts`) |
| 커밋 | 2개 (feat + Copilot 리뷰 반영) |
| mergeable | **CONFLICTING** |
| CI | no checks |
| Ref | Issue #260 (macOS에서의 커서 이동 관련 UX, OPEN) |
| 생성 | 2026-05-11 06:20 |

## 핵심 판정: devel 이미 흡수 + 충돌 상태

PR #819가 제출된 5/11 이후 다른 PR 처리 과정에서 **동일 영역의 변경이 이미 devel에 흡수**되어 충돌 발생.

### 현재 devel 상태 vs PR #819 인도물 비교

| 기능 | PR #819 | 현재 devel | 상태 |
|------|---------|-----------|------|
| Alt+Backspace (macOS 단어 삭제) | `deleteWordBackward()` 자체 헬퍼 | `cursor.moveToWordBoundary(-1)` API 사용 (라인 813~817) | **devel이 이미 구현**, 방식은 다름 |
| Ctrl+Backspace (Win/Linux 단어 삭제) | `handleCtrlKey` 내 backspace case | 미구현 | **PR #819에만 있음 (신규 가치)** |
| Cmd+Backspace (macOS 줄 시작까지 삭제) | `deleteToLineStart()` 함수 | 미구현 | **PR #819에만 있음 (신규 가치)** |
| 단어 경계 판정 | `findWordBoundaryInText()` + `isWordSeparator()` 자체 구현 | `cursor.moveToWordBoundary()` 커서 API | devel 방식이 더 깔끔 |
| Alt 조합 분기 구조 | `if (e.altKey)` 조건 재배치 | `isAltWordKey` 가드 + dispatcher 분리 | devel 구조가 정합성 우수 |

### 충돌 원인

PR #821 (Ctrl+Delete 단어 단위 전방 삭제)이 이미 devel에 머지되면서:
- Alt+Backspace/Delete 단어 삭제가 Backspace/Delete case 안에서 `cursor.moveToWordBoundary()` API로 구현됨
- Alt 조합 분기에 `isAltWordKey` 가드 도입
- PR #819가 수정하는 동일 영역과 양방향 충돌

## PR #819의 신규 가치 (2건)

### 1. Ctrl+Backspace → 단어 삭제 (Windows/Linux)

`handleCtrlKey()` 함수 내에 `case 'backspace'`를 추가:
```typescript
case 'backspace': {
  e.preventDefault();
  if (e.metaKey) {
    deleteToLineStart.call(this);
  } else {
    deleteWordBackward.call(this);
  }
  break;
}
```

본 영역은 현재 devel에 없으므로 신규 가치. 다만 `deleteWordBackward()` 대신 devel의 `cursor.moveToWordBoundary(-1)` API를 사용하는 것이 정합.

### 2. Cmd+Backspace → 줄 시작까지 삭제 (macOS)

```typescript
function deleteToLineStart(this: any): void {
  if (this.cursor.hasSelection()) {
    this.deleteSelection();
    return;
  }
  this.cursor.setAnchor();
  this.cursor.moveToLineStart();
  if (this.cursor.hasSelection()) {
    this.deleteSelection();
  }
}
```

macOS 표준 편집 동작. 현재 devel에 없으므로 신규 가치.

## 옵션 분류

### 옵션 A: 컨트리뷰터에게 devel rebase + 중복 제거 요청

충돌 해소 + Alt+Backspace 중복 제거 + 신규 가치(Ctrl+Backspace, Cmd+Backspace)만 남기도록 PR 갱신 요청.

**장점**: 컨트리뷰터가 정합 구현
**단점**: 컨트리뷰터 부담 + devel의 `moveToWordBoundary` API 파악 필요

### 옵션 B: 메인테이너가 cherry-pick 대신 직접 구현 (신규 가치 2건만)

PR #819의 신규 가치(Ctrl+Backspace, Cmd+Backspace)를 메인테이너가 직접 `handleCtrlKey()` 안에 추가. 기존 devel의 `moveToWordBoundary` API 재사용.

**장점**: 간결, 정합
**단점**: 컨트리뷰터 코드와 다른 구현

### 옵션 C: PR close + 컨트리뷰터에게 devel 흡수 사실 안내

Alt+Backspace는 이미 devel에 흡수. 신규 가치(Ctrl+Backspace, Cmd+Backspace)는 별도 PR로 재제출 권유.

**장점**: 깔끔한 분리
**단점**: 컨트리뷰터에게 재작업 요청

### 옵션 D: PR close + 메인테이너가 후속 이슈로 Ctrl/Cmd+Backspace 등록

PR #819 close 후 Issue #260 영역에 Ctrl+Backspace, Cmd+Backspace를 후속 이슈로 등록하여 누구나 처리 가능하게.

## 메인테이너 권고

**옵션 C** — 충돌 해소 자체가 단순하지 않고(Alt 분기 구조 전면 변경), 신규 가치 2건도 devel의 `moveToWordBoundary` API를 활용한 재구현이 더 깔끔. 컨트리뷰터에게 devel 흡수 사실 + 신규 가치 영역 안내 후 별도 PR 재제출 권유가 가장 효율적.

## 작업지시자 결정 요청

1. 옵션 A/B/C/D 중 어느 방향으로 진행?
2. Issue #260 (macOS 커서 이동 UX)에 Ctrl+Backspace, Cmd+Backspace 후속 영역 코멘트 추가?
