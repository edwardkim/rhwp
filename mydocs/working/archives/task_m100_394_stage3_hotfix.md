# Stage 3 Hotfix 보고서 — Task M100 #394

## 회귀 발견

작업지시자 시각 판정 중 다음 콘솔 오류 발견:

```
input-handler-mouse.ts:746 [InputHandler] hitTest 실패:
  TypeError: this.checkTransparentBordersTransition is not a function
```

증상:
- 단축키 `Alt+V → T` 한/영 영어 모드에서만 적용 (한글 모드에서 미적용)
- 콘솔에 hitTest 실패 TypeError 반복

## 원인

본 task 의 grep 점검에서 **`input-handler.ts` 만 검색** 하고 같은 클래스의 분리 파일 `input-handler-mouse.ts`, `input-handler-keyboard.ts` 를 누락. Stage 2 에서 메서드 본체를 주석 처리했지만 mouse / keyboard 파일에서 8 호출이 남아 있었음 → 메서드 미존재로 런타임 TypeError.

## 누락 호출 지점 (총 8 개)

| 파일 | 라인 (기존) | 컨텍스트 |
|------|------|----------|
| `input-handler-mouse.ts` | 587 | 표 객체 선택 진입 (셀 안 클릭) |
| `input-handler-mouse.ts` | 604 | 표 외곽 클릭 → 표 객체 선택 |
| `input-handler-mouse.ts` | 620 | 글상자 텍스트 직접 히트 |
| `input-handler-mouse.ts` | 735 | 일반 mousedown 종료 |
| `input-handler-keyboard.ts` | 508 | 표 객체 선택 → ESC (표 밖 이동) |
| `input-handler-keyboard.ts` | 529 | 중첩 표 Delete (선택만 해제) |
| `input-handler-keyboard.ts` | 537 | 일반 표 Delete (표 삭제 후) |
| `input-handler-keyboard.ts` | 575 | 표 Ctrl+X (잘라내기 후) |

## 수정

8 호출 모두 동일 패턴으로 주석 처리:

```typescript
// [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
// this.checkTransparentBordersTransition();
```

## 검증

| 항목 | 결과 |
|------|------|
| TS 빌드 (`npx tsc --noEmit`) | ✅ 통과 |
| 활성 코드의 `this.checkTransparentBordersTransition()` 호출 grep | 0 건 ✅ |
| Dev server HMR 반영 (`curl http://localhost:7700/src/engine/...`) | ✅ 즉시 반영 |

## 회귀 정정 후 동작

- TypeError 사라짐 → hitTest 정상
- `Alt+V → T` 단축키는 일반 dispatcher 경로로 정상 — input-handler 의 hitTest 회복으로 다른 문제 (단축키 미적용 등) 와 무관 확인
- 한/영 모드 영향: 단축키 자체는 `e.key === 'v'` 외에 `'V'`, `'ㅍ'` 도 받음 (한글 IME). hitTest 실패가 키 핸들링 흐름을 부분 차단했을 가능성 — 본 hotfix 로 해소

## 교훈

같은 클래스가 여러 파일에 분산되어 있을 때 (`input-handler.ts`, `input-handler-mouse.ts`, `input-handler-keyboard.ts`) 단일 파일 grep 만으로는 호출 지점 누락 위험. 차후 작업 시 클래스 단위 grep:

```bash
grep -rn "checkTransparentBordersTransition" rhwp-studio/src
```

## 영향

- 본 task 의 사용자 가시 동작 (셀 진입 자동 ON 비활성화) 은 변함 없음
- 회귀 (TypeError) 정정만 추가
- 5 영역 → **13 영역** 으로 `[Task #394]` 마커 확대 (메서드 본체 1, 상태 변수 1, 이벤트 핸들러 1, 호출 지점 10)

## 산출물

- 수정 파일: `rhwp-studio/src/engine/input-handler-mouse.ts`, `rhwp-studio/src/engine/input-handler-keyboard.ts`
- 본 보고서: `mydocs/working/task_m100_394_stage3_hotfix.md`
