# 최종 결과 보고서 — Task #130

**이슈**: [#130](https://github.com/edwardkim/rhwp/issues/130)
**타이틀**: 투명 선 토글 단축키 추가 및 초기값 버그 수정 — Alt+V, T (Chord)
**마일스톤**: M100
**완료일**: 2026-04-13
**브랜치**: `local/task130`

---

## 완료 내용

### 1. Alt+V → T Chord 단축키

한컴오피스 `Alt+V, T` 조합을 계승하여 `view:border-transparent` 토글 단축키 구현.

- `chordMapV` 테이블: `t/ㅅ → view:border-transparent` (한글 IME 대응)
- Alt+V 입력 → `_pendingChordV = true` 대기 → T 입력 → 커맨드 디스패치
- 기존 충돌 단축키 없음: Ctrl+F5(브라우저 새로고침), Alt+T(문단모양), F5(셀 선택) 모두 회피

### 2. 초기값 불일치 버그 수정

**원인**: 셀 진입 시 `checkTransparentBordersTransition()`이 자동으로 `setShowTransparentBorders(true)` 호출. 이 상태에서 커맨드의 클로저 변수 `showBorders`는 여전히 `false`라 첫 토글이 `false→true`(변화 없음)로 무효화.

**수정**: 클로저 변수 제거 → `getShowTransparentBorders()` WASM getter로 실제 상태를 읽어 토글. 상태 불일치 근본 해결.

---

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/wasm_api.rs` | `getShowTransparentBorders` getter 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `getShowTransparentBorders()` 브릿지 추가 |
| `rhwp-studio/src/command/commands/view.ts` | 초기값 버그 수정 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | `chordMapV` + Chord 처리 추가 |

---

## 검증

- `cargo check` 통과
- `npx tsc --noEmit` 통과
- 기존 단축키 회귀 없음

---

## 커밋

- `eabca79` Task #130: 투명 선 토글 단축키 추가 및 초기값 버그 수정
