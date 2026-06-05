# PR #1212 처리 보고서 — rhwp-studio 입력 편집 재렌더 비용 축소 (narrow invalidation)

- **작성일**: 2026-06-01
- **PR**: #1212 → **MERGED** (devel, 로컬 `--no-ff` 머지 + orders 충돌 해소)
- **컨트리뷰터**: @postmelee (핵심 컨트리뷰터)
- **연결 이슈**: #1211 → **CLOSED** (`Related:` — 수동 클로즈)
- **판단**: **머지** ✅ (작업지시자 승인)

## 결정 사유

표 셀 텍스트 입력이 매 키마다 full refresh(visible page 전체 재렌더 + image retry reset)를 타던
비용을, 같은 cellPath 단일 insert/delete 만 page-local repaint 로 보내는 보수적 narrow invalidation
으로 축소. Rust 무변경, npm test 52 passed, CI green. 잘못 판정 시 full refresh fallback(안전).

## 변경 요약 (14 파일, +731/−9 — rhwp-studio TS 전용, Rust 0)

| 파일 | 변경 |
|------|------|
| `input-edit-invalidation.ts` | `isPageLocalTextEditCommand` 판정 헬퍼(신규) |
| `canvas-view.ts` | `document-page-invalidated` 처리 + page-local renderCanvas() / count·invalid index fallback |
| `input-handler.ts` | command 전/후 위치 비교 → 같은 cellPath 단일 insert/delete page-local 라우팅 |
| `input-handler-text.ts` | IME 조합 raw / iOS fallback 도 같은 라우터 |
| `tests/input-edit-invalidation.test.ts` | 판정 경계 단위 테스트 |
| mydocs ×8 | 컨트리뷰터 작업 문서 |

선택안 A(이벤트 의미 분리) + C(image retry reset 생략). B/D 제외(근거: 이벤트 계약 모호 / API 표면 증가→후속).

## 판정 로직 — 보수적 화이트리스트

`insertText`/`deleteText` + section/parentPara/control/cell/cellParaIndex 동일 + sameCellPath 일치
시에만 true. 본문 입력(parentParaIndex 미정의)·구조 변경·셀 이동은 full refresh. 좁게 허용,
나머지 안전 경로 — `feedback_hancom_compat_specific_over_general` 취지.

## 충돌 해소 (CONFLICTING)

`orders/20260601.md` 단일 충돌 — #1207 머지로 추가한 우리 기록과 #1212 의 #1211 작업일지가 파일
상단에 함께 추가된 텍스트 충돌(코드 충돌 0). 양쪽 작업일지 + M100 테이블 전부 보존하여 해소.

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge `--no-ff` | ✅ orders 해소(양쪽 보존), 코드 충돌 0 |
| Rust 소스 변경 | ✅ 0 (rhwp-studio 전용) → WASM 재빌드 불요 |
| rhwp-studio `npm test` | ✅ **52 passed, 0 fail** (input-edit-invalidation 신규 포함) |
| rhwp-studio `npm run build` | ✅ tsc + vite |
| Rust 전체 `cargo test --tests` | ✅ **1924 passed, 0 failed** (무변경, 회귀 0) |
| CI(PR) | ✅ 전부 PASS |

## 위험 평가

- 낮음. Rust 코어 무변경, narrow 경로는 page-local repaint(콘텐츠 동일). 잘못 판정 시 위험은
  "최적화 미적용=기존 full refresh" 방향(안전). page count/invalid index fallback.

## 처리 절차

1. PR 정보 — CONFLICTING(orders)/DIRTY, CI green, rhwp-studio TS 5파일. @postmelee(#1207/#1212 연계).
2. 본문/이슈/판정 로직 검토. 트러블슈팅 검색(render/invalidation).
3. 로컬 `pr1212-verify`: orders 충돌 해소(양쪽 보존) / Rust 무변경 / npm test 52 / build / Rust test 1924.
4. 작업지시자 머지 승인.
5. devel `--no-ff` 머지(orders 해소) + push. 이슈 #1211 클로즈. (WASM 불요.)

## 비고

- 후보 D(flow image 전용 WASM API)는 A+C 후 잔여 병목 시 후속 이슈(PR 본문 명시).
- #1207(커서를 셀에 남김)이 입력 비용 노출 → #1212 가 그 비용 축소. 직접 연계.
