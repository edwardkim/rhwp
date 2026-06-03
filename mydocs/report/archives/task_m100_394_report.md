# 최종 결과 보고서 — Task M100 #394

## 이슈

[#394](https://github.com/edwardkim/edwardkim/rhwp/issues/394) — rhwp-studio 표 셀 진입 시 투명선 자동 ON 로직 제거 (기본값 OFF 유지)

## 결과 요약

rhwp-studio 의 자동 투명선 토글 로직을 **삭제하지 않고 주석으로 보존** 하여 향후 되돌리기 가능한 형태로 비활성화. 표 셀 진입 / 탈출 시 자동 ON/OFF 안 일어나며, 사용자 명시 토글 (`Alt+V → T`, 메뉴) 만으로 동작.

## 변경 파일

`rhwp-studio/src/engine/input-handler.ts` + `input-handler-mouse.ts` + `input-handler-keyboard.ts`, 총 13 영역 주석 처리:

### `input-handler.ts` (5 영역)

| 위치 | 영역 | 주석 형식 |
|------|------|----------|
| line 219-224 | 상태 변수 3 개 (`wasInCell`, `manualTransparentBorders`, `autoTransparentBorders`) | `//` 라인 |
| line 392-397 | `transparent-borders-changed` 이벤트 핸들러 | `//` 라인 |
| line 1507-1510 | 호출 지점 (`updateCaretAndScroll`) | `//` 라인 |
| line 1527-1528 | 호출 지점 (`updateCaretNoScroll`) | `//` 라인 |
| line 1777-1808 | 메서드 본체 `checkTransparentBordersTransition()` | `/* ... */` 블록 |

### `input-handler-mouse.ts` (4 호출 지점, Stage 3 hotfix)

| 위치 | 컨텍스트 |
|------|----------|
| line 588 | 표 객체 선택 진입 (셀 안 클릭) |
| line 606 | 표 외곽 클릭 → 표 객체 선택 |
| line 623 | 글상자 텍스트 직접 히트 |
| line 739 | 일반 mousedown 종료 |

### `input-handler-keyboard.ts` (4 호출 지점, Stage 3 hotfix)

| 위치 | 컨텍스트 |
|------|----------|
| line 509 | 표 객체 선택 → ESC (표 밖 이동) |
| line 531 | 중첩 표 Delete (선택만 해제) |
| line 540 | 일반 표 Delete (표 삭제 후) |
| line 579 | 표 Ctrl+X (잘라내기 후) |

모든 영역에 `[Task #394]` 마커 + 사유 + 되돌리기 방법 명시.

## 변경 없음

- WASM 코어 (`src/document_core/`, `src/renderer/layout.rs`, `src/wasm_api.rs`) — 기본값 OFF 그대로
- 사용자 토글 명령어 (`rhwp-studio/src/command/commands/view.ts`)
- 단축키 / 메뉴 (`Alt+V → T`)
- emit 측 (`view.ts:121` 의 `transparent-borders-changed` emit) — 향후 다른 구독자 사용 가능
- 다른 확장 (`rhwp-chrome`, `rhwp-firefox`, `rhwp-vscode`) — 자체 자동 토글 로직 없음

## 단계별 진행

| 단계 | 작업 | 커밋 | 보고서 |
|------|------|------|--------|
| Stage 1 | 호출 지점 (input-handler.ts line 1503, 1520) 주석 처리 | `d4f9ba6` | `mydocs/working/task_m100_394_stage1.md` |
| Stage 2 | 메서드 본체 + 상태 변수 + 이벤트 핸들러 주석 처리 | `f2317fb` | `mydocs/working/task_m100_394_stage2.md` |
| Stage 3 | 빌드 / 회귀 테스트 / 시각 검증 | `a4e5a41` | `mydocs/working/task_m100_394_stage3.md` |
| Stage 3 hotfix | mouse / keyboard 핸들러의 누락된 8 호출 지점 주석 처리 (TypeError 회귀 정정) | (pending) | `mydocs/working/task_m100_394_stage3_hotfix.md` |

## 검증 결과

| 검증 항목 | 결과 |
|----------|------|
| TS 빌드 (`npx tsc --noEmit`) | ✅ 통과 |
| Vite 빌드 (`npx vite build`) | ✅ 통과 (567ms, 85 modules, dist 산출) |
| `cargo test --lib` | ✅ **1016 passed**, 0 failed (PR #392 시점과 동일 — 회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| `test_task79_transparent_border_lines` | ✅ ok (코어 투명선 기능 영향 없음) |
| E2E 식별자 영향 | ✅ 0건 (회귀 위험 없음) |
| Dev server HMR 반영 | ✅ port 7700, 5 영역 모두 응답 |
| 작업지시자 시각 판정 | ✅ 통과 |

## 동작 변화

| 상태 | 변경 전 | 변경 후 |
|------|--------|--------|
| 문서 로드 직후 | 투명선 OFF | 동일 (OFF) |
| 표 셀 안 커서 진입 | **자동 ON** | **자동 토글 안 함** (OFF 유지) |
| 표 셀 밖 커서 탈출 | 자동 OFF (auto 였던 경우) | 변화 없음 |
| 사용자 토글 (`Alt+V → T` / 메뉴) | manual ON/OFF | 동일 |
| manual ON 후 셀 진입/탈출 | 자동 OFF 안 일어남 | 동일 (manual 유지) |

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 한컴 출력 정합성 우선 | ✅ 한컴 뷰어와 시각 일치 (투명선 없는 표 = 기본 화면) |
| 작업지시자 시각 판정 게이트 | ✅ 시각 판정 후 머지 |
| 타스크 프로세스 준수 | ✅ 이슈 → 브랜치 → 계획서 → 단계별 보고서 → 최종 보고서 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/task394` 에서 커밋 |

## 되돌리기 방법

향후 정책 복귀 시 다음 13 영역의 주석을 동시에 해제:

```bash
grep -rn "Task #394" rhwp-studio/src/engine/
```

`input-handler.ts` (5) + `input-handler-mouse.ts` (4) + `input-handler-keyboard.ts` (4) = **총 13 영역**. 모두 마커가 있어 일괄 추적 / 정리 가능. 별도 task 로 깔끔히 정리 예정.

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 커밋
2. `local/task394` → `local/devel` 머지
3. `local/devel` → `devel` 머지 + push
4. 이슈 #394 close

## 산출물

- 변경 파일: `rhwp-studio/src/engine/input-handler.ts`
- 수행 계획서: `mydocs/plans/task_m100_394.md`
- 구현 계획서: `mydocs/plans/task_m100_394_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_394_stage{1,2,3}.md`
- 최종 보고서: `mydocs/report/task_m100_394_report.md` (본 문서)

## 참고

- 이슈: [#394](https://github.com/edwardkim/rhwp/issues/394)
- 브랜치: `local/task394`
- 작업 일자: 2026-04-28
