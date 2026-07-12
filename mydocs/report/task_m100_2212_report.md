# Task M100 #2212 최종 보고 — 중첩 표 셀 경로 bbox 해석 실패 (편집)

- 이슈: #2212 (#2211 진단 중 작업지시자 콘솔 로그로 발견) / 브랜치: `local/task2212`
- 기간: 2026-07-12 / 판정: **통과** (studio 실사용 — 셀 선택 동작·콘솔 청정)
- 재현: `samples/basic/issue1994_behindtext_table_20200830.hwp` p1 좌측 단
  표 pi=5 셀[10] 안 인라인 TAC 내부 표(18×9)

## 결론

셀 안 인라인 TAC 표를 렌더하는 run_tacs 경로가 cell_context를 전파하지 않아
내부 표 TextRun에 2단 경로가 기록되지 않던 결함을 정정. studio의 경로 기반
셀 bbox 조회(updateCellSelection/renderTableObjectSelection)가 복원되고
입력마다 반복되던 예외가 소멸 — #2222(성능)의 예외 반복 축도 함께 제거.

## 원인 (가설 A 확정)

- 렌더 트리 실덤프: ppi=5 컨텍스트가 외곽 1단 11종뿐 — 내부 표 2단 경로 전무.
- run_tacs 인라인 TAC 표 렌더([paragraph_layout.rs:4907](../../src/renderer/layout/paragraph_layout.rs#L4907))가
  `cell_context=None, depth=0` 호출 — table_layout 중첩 분기(:3475)는 올바른
  확장을 하고 있었고 이 경로만 누락.
- 부수 기록: 조회 API의 Err→JsValue 변환이 네이티브 테스트에서 abort — 향후
  네이티브 재현 시 렌더 트리 검사를 선행할 것.

## 정정 (`f0a70883`)

외곽 셀 경로를 확장한 2단 cell_context 전달(+depth 1) — table_layout 중첩
분기와 동일 규칙, +13줄 단일 지점.

## 게이트 + 검증

| 항목 | 결과 |
|------|------|
| 2단 경로 기록 | 11종 → **101종** / 실패 경로 조회 **Err→Ok** (내부 셀 48 bbox) |
| 렌더 픽셀 불변 | 주보 4페이지 SVG **바이트 동일** (patch 왕복 실측) |
| fmt / clippy / 전수 `--no-fail-fast` | 통과 / 0 / **3,050/0** (표적 테스트 신설, 수정 전 FAILED 실증) |
| OVR 3샘플 | 회귀 0건 |
| studio 실사용 (WASM 빌드) | **통과** — 셀 선택 동작, 예외 소멸 |

## 산출물

- 커밋: `9d3c0472`(계획), `f0a70883`(정정+테스트+2·3단계 보고)
- 문서: `working/task_m100_2212_stage2.md`, 본 보고서
- 연계: #2211(발견 경위), #2222(예외 반복 축 제거 — 캐시 과제는 유지)
