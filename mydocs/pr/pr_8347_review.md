# PR #8347 검토 — fix(layout): 엇갈린 병합 칸의 행 높이를 칸 내용으로 푼다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8347, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `19aed211d178996cf83aae89388ed91d6ec93bcf`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:52.364267+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: Build & Test: failure, test-archive-b-shard-1 / Default-feature tests (Archive B): cancelled, Lint (fmt, clippy, WASM check): cancelled.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 공통 측정·배치 경로와 경계

- 선행 해결: #7685의 선언 초과분 처리·미지 행 풀이를 공통화한 `3eb7ff36287973f7569440fe5581dbc0b7f7bb9a`를 추적했다. 최신 plain-cell 흐름은 #8174/#8360의 `f125c735c7`도 포함한다. 기존 선언 초과분을 먼저 반영하는 순서와 Storage 소비 계약을 보존한다.
- 생산: `height_measurer::solve_rowspan_row_heights`가 균등 fallback 이전에 제약 행을 모두 풀었는지 반환한다. `LayoutEngine::resolve_reflow_span_requirements`는 선언으로 풀 수 없지만 실제 가로쓰기 내용으로 모두 풀 수 있는 경우만 공통 높이를 채택한다. 기존 plain reflow overflow 경로는 그대로 둔다.
- 측정 소비: `HeightMeasurer`의 span content 요구(글줄·개체·padding) → `resolve_reflow_span_requirements` → measured `row_heights`. 실제 배치 소비: `LayoutEngine::resolve_row_heights`의 같은 요구와 같은 helper → 실제 row/cell 원점. 이후의 미결 행 fallback은 이 사례의 완전히 풀린 양의 행 높이를 덮어쓰지 않는다. 선언보다 작은 값으로 예약하고 paint에서 늘리는 보정은 추가하지 않았다.
- 실제 사례는 2쪽 안에서 끝나는 27행 표다. 표 끝 → 다음 안내 표 → 이용허락 본문의 순서·비겹침, 모든 칸의 표 내부 포함과 2→3쪽 본문 문단 29의 순서·전체 내용·누락/중복을 정식 관계 검사로 보호했다. 이 새 검사는 원 행 높이 검사와 함께 최소 글꼴 집중 회귀에서 통과했다. 분할 알고리즘 자체를 변경하거나 내부 cut 단위의 새 계약을 주장하지 않는다.
- 1차 전7쪽 raw 최저: Native 99.32%, fresh WASM 99.98%. 영향 2쪽은 99.50/99.99%, 본문 이어받기 3쪽은 99.99/100.00%다. 독립 Print와 같은 표 외곽·뒤 본문·다음 쪽 첫 글줄을 직접 판독했다. 마지막 setter 비용 보정 후 최종 runtime 재출력과 편람 gate도 통과했다.

## 값의 생산·소비 코드 위치

| 단계 | 실제 경로 |
| --- | --- |
| 제약 풀이와 fallback 이전의 해결 상태 | [height_measurer.rs](../../src/renderer/height_measurer.rs#L6758)의 `solve_rowspan_row_heights` |
| 공통 선언/내용 요구 판정 | [plain_cell_flow.rs](../../src/renderer/layout/table_layout/plain_cell_flow.rs#L120)의 `resolve_reflow_span_requirements` |
| 측정의 최종 높이 소비 | [height_measurer.rs](../../src/renderer/height_measurer.rs#L5450) |
| 실제 행/칸 원점과 높이에 적용 | [table_layout.rs](../../src/renderer/layout/table_layout.rs#L5933) |

이 작업은 분할 cut 알고리즘을 바꾸지 않는다. 실제 대상 표는 2쪽에서 종료되며 그 뒤 문단의 2→3쪽 이어받기·순서·누락/중복을 관계 검사와 독립 Print에서 확인했다. 전체 회귀에서도 기존 rowspan 분할·물리 프레임·예산 실패 경계 검사를 유지한다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·새 통합 PR·merge·후속 댓글은 현재 아직 실행하지 않았다. 승인 뒤 해당 제출 head의 CI와 최신 base의 충돌 여부를 다시 확인한다.
