# PR #821 검토

## 메타

| 항목 | 값 |
|------|---|
| PR | [#821](https://github.com/edwardkim/rhwp/pull/821) |
| 제목 | feat: Ctrl+Delete 단어 단위 전방 삭제 (#260) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) |
| 규모 | +74/-1, 1 파일 (`input-handler-keyboard.ts`) |
| 커밋 | 2개 (feat + Copilot 리뷰 반영) |
| mergeable | **CONFLICTING** |
| CI | no checks |
| Ref | Issue #260 |

## 현재 devel 상태 vs PR #821

| 기능 | PR #821 | 현재 devel | 상태 |
|------|---------|-----------|------|
| Alt+Delete (macOS 단어 전방 삭제) | 미포함 (PR #819 영역) | `cursor.moveToWordBoundary(1)` (라인 816) | **이미 devel에 있음** |
| Ctrl+Delete (Win/Linux 단어 전방 삭제) | `deleteWordForward()` 자체 헬퍼 | 미구현 | **신규 가치** |
| 필드(누름틀) 경계 보호 | `getFieldInfoAt` + 경계 클램프 | 미구현 | **신규 가치** |
| 단어 경계 판정 | `findWordEndInText()` + `isWordSep()` 자체 구현 | `cursor.moveToWordBoundary()` 커서 API | devel 방식이 더 깔끔 |

PR #819와 동일 패턴 — 충돌 + 중복 + 신규 가치 혼재.

## 신규 가치

1. **Ctrl+Delete → 단어 전방 삭제**: `handleCtrlKey()` switch에 추가
2. **필드 경계 보호**: 누름틀 끝에서 삭제 차단 + wordEnd를 필드 끝으로 클램프

## 옵션 분류

### 옵션 A: 컨트리뷰터에게 devel rebase + 중복 제거 요청

충돌 해소 + Alt+Delete 중복 영역 제거 + 신규 가치(Ctrl+Delete + 필드 경계 보호)만 남기도록 PR 갱신 요청. devel의 `moveToWordBoundary` API 활용 권유.

**장점**: 컨트리뷰터가 직접 정합, 필드 경계 보호 로직도 컨트리뷰터 의도대로 유지
**단점**: 컨트리뷰터 부담 + devel API 파악 필요 + 응답 대기 시간

### 옵션 B: 메인테이너가 devel API로 직접 구현 (신규 가치만)

PR #819 처리와 동일 패턴. Ctrl+Delete를 `handleCtrlKey()` switch에 추가, devel `moveToWordBoundary(1)` API 재사용. 필드 경계 보호는 별도 후속 이슈로 분리.

**장점**: 즉시 처리, devel API 일관성 유지
**단점**: 필드 경계 보호 로직 누락, 컨트리뷰터 코드와 다른 구현

### 옵션 C: PR close + 별도 PR 재제출 권유

Alt+Delete가 이미 devel에 흡수된 사실 안내 + 신규 가치(Ctrl+Delete + 필드 경계 보호)를 별도 PR로 재제출 권유.

**장점**: 깔끔한 분리, 컨트리뷰터가 필드 경계 보호도 포함한 완성된 PR 제출 가능
**단점**: 컨트리뷰터에게 재작업 요청

### 옵션 D: 옵션 B + 필드 경계 보호 포함

메인테이너가 Ctrl+Delete + 필드 경계 보호 모두 직접 구현. PR #821의 `getFieldInfoAt` 로직을 devel API와 결합.

**장점**: 신규 가치 전부 반영
**단점**: 필드 경계 보호의 동작 검증이 필요 (시각 판정 영역 확대)

## 작업지시자 결정 요청

옵션 A/B/C/D 중 어느 방향으로 진행?
