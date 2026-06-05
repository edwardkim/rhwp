# PR #1212 검토 — rhwp-studio 입력 편집 재렌더 비용 축소 (narrow invalidation)

- **작성일**: 2026-06-01
- **PR**: #1212 (OPEN)
- **컨트리뷰터**: @postmelee (핵심 컨트리뷰터 — 방금 #1207 머지)
- **연결 이슈**: #1211 (`Related:` — 자동 종료 안 함, OPEN)
- **base/head**: `devel` ← `task-1211` (cross-repo)
- **규모**: 14 파일, +731/−9 (**rhwp-studio TS 5 + 테스트 1 + 작업문서 8**, **Rust 0**)
- **mergeable**: **CONFLICTING / DIRTY** → orders 충돌 해소(아래 4절)
- **CI**: **전부 PASS** (Build&Test/Canvas visual diff/Analyze×3/CodeQL)
- **라벨**: enhancement / 마일스톤 v1.0.0

## 1. 문제 (#1211 + #1207 후속)

rhwp-studio 에서 표 셀 텍스트 입력이 매 키마다 `document-changed → CanvasView.refreshPages()`
full refresh(visible page 전체 재렌더 + `resetImageRetryState()`)를 타 입력 지연. #1207 로 커서가
중첩 셀에 정확히 남으면서 기존 입력 렌더 비용이 체감됨.

## 2. 수정 내용 검토 (Rust 무변경, rhwp-studio 전용)

| 파일 | 변경 |
|------|------|
| `input-edit-invalidation.ts` | `isPageLocalTextEditCommand` 판정 헬퍼 (신규) |
| `canvas-view.ts` | `document-page-invalidated` 처리 + page-local `renderCanvas()` 분리(layout 재계산/전체 release/image retry reset 생략), page count 변경/invalid index 는 full refresh fallback |
| `input-handler.ts` | command 전/후 DocumentPosition 비교 → 같은 cellPath 단일 insert/delete 만 page-local 라우팅 |
| `input-handler-text.ts` | IME 조합 raw 편집 / iOS fallback 도 같은 라우터 |
| `tests/input-edit-invalidation.test.ts` | 판정 경계 단위 테스트 |

선택안 A(이벤트 의미 분리 `document-page-invalidated`) + C(image retry reset 생략). B/D 는
근거와 함께 제외(B: 이벤트 계약 모호, D: API 표면 증가 — 후속).

### 판정 로직 — 보수적 화이트리스트 (핵심 안전성)

`isPageLocalTextEditCommand(commandType, before, after)`:
- `insertText`/`deleteText` 만 (Set 화이트리스트)
- `parentParaIndex` 미정의 → false (본문 문단 입력 = full refresh)
- section/parentPara/control/**cell/cellParaIndex 전부 동일** + `sameCellPath` 일치해야 true
- → 셀 위치가 조금이라도 바뀌면(문단 분할·병합, 구조 변경, 셀 이동) **full refresh 로 안전하게 fallback**

`feedback_hancom_compat_specific_over_general`(구조 가드) 취지 부합 — 좁게 허용, 나머지는 안전 경로.

## 3. 위험 평가

- **낮음.** Rust 코어 무변경. narrow 경로는 page-local repaint 만 (콘텐츠 동일). 잘못 판정 시
  위험은 "최적화 미적용(=기존 full refresh)" 방향이라 안전. page count/invalid index fallback.
- 본문 입력/붙여넣기/문단 분할·병합/표·객체·페이지 구조/header·footer·footnote 는 기존 경로 유지.

## 4. 충돌 해소 (CONFLICTING)

`orders/20260601.md` 단일 충돌 — 방금 #1207 머지로 우리가 추가한 기록과 #1212 의 #1211 작업일지가
파일 상단에 함께 추가된 텍스트 충돌(코드 충돌 0). 양쪽 작업일지 + M100 테이블 전부 보존하여 해소.

## 5. 검증 결과 (로컬 머지 시뮬 `pr1212-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ orders 충돌 해소(양쪽 보존), 코드 충돌 0 |
| Rust 소스 변경 | ✅ 0 (rhwp-studio 전용) |
| **rhwp-studio `npm test`** | ✅ **52 passed, 0 fail** (input-edit-invalidation 신규 포함) |
| rhwp-studio `npm run build` | (진행) |
| Rust 전체 `cargo test --tests` | (진행 — 무변경이라 회귀 0 예상) |
| CI(PR) | ✅ 전부 PASS |

## 6. 판단 — 머지 권고 (편집기 입력 체감 판정 게이트)

- 설계 신중(보수적 화이트리스트, 안전 fallback), Rust 무변경, npm test green, CI green.
- 성능 최적화라 정확성 회귀 위험은 "최적화 미적용" 방향(안전). 다만 narrow 경로의 실제 화면
  정합(입력 후 page-local repaint 가 콘텐츠를 올바로 갱신하는지)은 **작업지시자 편집기 체감 판정**
  게이트 권고(성명 칸 연속 입력 → 지연 감소 + 화면 정합).
- 승인 + 체감 확인 시 메인테이너 로컬 `--no-ff` 머지(orders 해소 포함) + push. 이슈 #1211 수동 클로즈.
  (Rust 무변경 → WASM 재빌드 불요.)

## 7. 비고

- 후보 D(flow image 전용 WASM API)는 A+C 후에도 병목 남으면 후속 이슈(PR 본문 명시).
- @postmelee #1207(방금 머지)과 직접 연계 — #1207 이 커서를 셀에 남겨 입력 비용 노출, #1212 가 그 비용 축소.
