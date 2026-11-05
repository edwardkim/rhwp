# PR #8347 검토 — fix(layout): 엇갈린 병합 칸의 행 높이를 칸 내용으로 푼다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8347, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `19aed211d178996cf83aae89388ed91d6ec93bcf`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:52.364267+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: Build & Test: failure, test-archive-b-shard-1 / Default-feature tests (Archive B): cancelled, Lint (fmt, clippy, WASM check): cancelled.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](../assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 공통 측정·배치 경로와 경계

- 선행 해결: #7685의 선언 초과분 처리·미지 행 풀이를 공통화한 `3eb7ff36287973f7569440fe5581dbc0b7f7bb9a`를 추적했다. 최신 plain-cell 흐름은 #8174/#8360의 `f125c735c7`도 포함한다. 기존 선언 초과분을 먼저 반영하는 순서와 Storage 소비 계약을 보존한다.
- 생산: `height_measurer::solve_rowspan_row_heights`가 균등 fallback 이전에 제약 행을 모두 풀었는지 반환한다. `LayoutEngine::resolve_reflow_span_requirements`는 선언으로 풀 수 없지만 실제 가로쓰기 내용으로 모두 풀 수 있는 경우만 공통 높이를 채택한다. 기존 plain reflow overflow 경로는 그대로 둔다.
- 측정 소비: `HeightMeasurer`의 span content 요구(글줄·개체·padding) → `resolve_reflow_span_requirements` → measured `row_heights`. 실제 배치 소비: `LayoutEngine::resolve_row_heights`의 같은 요구와 같은 helper → 실제 row/cell 원점. 이후의 미결 행 fallback은 이 사례의 완전히 풀린 양의 행 높이를 덮어쓰지 않는다. 선언보다 작은 값으로 예약하고 paint에서 늘리는 보정은 추가하지 않았다.
- 실제 사례는 2쪽 안에서 끝나는 27행 표다. 표 끝 → 다음 안내 표 → 이용허락 본문의 순서·비겹침, 모든 칸의 표 내부 포함과 2→3쪽 본문 문단 29의 순서·전체 내용·누락/중복을 정식 관계 검사로 보호했다. 이 새 검사는 원 행 높이 검사와 함께 최소 글꼴 집중 회귀에서 통과했다. 분할 알고리즘 자체를 변경하거나 내부 cut 단위의 새 계약을 주장하지 않는다.
- 1차 전7쪽 raw 최저: Native 99.32%, fresh WASM 99.98%. 영향 2쪽은 99.50/99.99%, 본문 이어받기 3쪽은 99.99/100.00%다. 독립 Print와 같은 표 외곽·뒤 본문·다음 쪽 첫 글줄을 직접 판독했다. 마지막 setter 비용 보정 후 최종 runtime 재출력과 편람 gate도 통과했다.

## 값의 생산·소비 코드 위치

| 단계 | 실제 경로 |
| --- | --- |
| 제약 풀이와 fallback 이전의 해결 상태 | [height_measurer.rs](../../../src/renderer/height_measurer.rs#L6758)의 `solve_rowspan_row_heights` |
| 공통 선언/내용 요구 판정 | [plain_cell_flow.rs](../../../src/renderer/layout/table_layout/plain_cell_flow.rs#L120)의 `resolve_reflow_span_requirements` |
| 측정의 최종 높이 소비 | [height_measurer.rs](../../../src/renderer/height_measurer.rs#L5450) |
| 실제 행/칸 원점과 높이에 적용 | [table_layout.rs](../../../src/renderer/layout/table_layout.rs#L5933) |

이 작업은 분할 cut 알고리즘을 바꾸지 않는다. 실제 대상 표는 2쪽에서 종료되며 그 뒤 문단의 2→3쪽 이어받기·순서·누락/중복을 관계 검사와 독립 Print에서 확인했다. 전체 회귀에서도 기존 rowspan 분할·물리 프레임·예산 실패 경계 검사를 유지한다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](../assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](../assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](../assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

병합 칸의 마지막 내용이 표 안에 남고 2쪽 표 끝과 안내 표·후속 본문 및 3쪽 이어받기의 순서와 누락·중복을 확인했다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `rowspan_staggered_heights` 2쪽 | native | 99.49615% | 89.23774% | 14.42343% | [review](../assets/semanticist21-20261105/comment-evidence/pr8347-rowspan_staggered_heights/native/review/review_002.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8347-rowspan_staggered_heights/native/overlay/overlay_002.png) |
| `rowspan_staggered_heights` 2쪽 | wasm | 99.98928% | 91.83435% | 29.06896% | [review](../assets/semanticist21-20261105/comment-evidence/pr8347-rowspan_staggered_heights/wasm/review/review_002.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8347-rowspan_staggered_heights/wasm/overlay/overlay_002.png) |
| `rowspan_staggered_heights` 3쪽 | native | 99.99338% | 91.43521% | 15.79487% | [review](../assets/semanticist21-20261105/comment-evidence/pr8347-rowspan_staggered_heights/native/review/review_003.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8347-rowspan_staggered_heights/native/overlay/overlay_003.png) |
| `rowspan_staggered_heights` 3쪽 | wasm | 99.99509% | 94.14644% | 35.87646% | [review](../assets/semanticist21-20261105/comment-evidence/pr8347-rowspan_staggered_heights/wasm/review/review_003.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8347-rowspan_staggered_heights/wasm/overlay/overlay_003.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 해당 기능 검증 문서의 Native/fresh WASM 전쪽 raw 90% 미만은 0쪽이다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](../assets/semanticist21-20261105/visual-manifest.json)를 연결한다.

## GitHub CI와 self-review

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)의 code head `da0074f41ee01d7fa79e6bd4ffd5cadf7c69665d`에서 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41937077195)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41937077068)·Adapter·Proptest와 최종 CI 정책 집계의 성공을 확인했다. self-review의 추가 차단 사항은 없으며 통합 후보는 승인이다. [실제 check·job 결과](../assets/semanticist21-20261105/github-ci-code-head.json), [self-review 근거](../assets/semanticist21-20261105/self-review.json).

이 기록은 녹색 code head 뒤의 문서 전용 single-parent trailing commit이다. 새 trailing head의 실제 fast-pass·필수 집계·최신 base 충돌 상태를 확인한 뒤 승인된 merge를 진행한다.

공통 보안 후속으로 기존 행 높이 진단 출력 두 그룹과 같은 채널의 span 진단만 제거했다. 보정 source `d75809517f`에서 전체 회귀 12,499개·Native CI 22개 명령 192개·Native 전체 라이브러리 4,040개를 통과했고, 실제 암호 Native CLI의 SVG 24쪽은 이전과 같다. 새 code head `da0074f41e`의 독립 GHAS CodeQL도 성공했다. 진단 외 조판 코드와 기존 시각 producer를 구분한 [실제 근거](../assets/semanticist21-20261105/ci-security-followup.md)를 후속 댓글과 함께 연결한다.

## Merge 확인

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)을 merge commit `e9d4a23be4523a97c29ce6f5e407aba6c4cfb26c`로 통합했다. 원 PR 10건 모두 간접 MERGED를 확인했으며 원 기여 commit·fork branch를 보존했다. [code CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[문서 head CI](https://github.com/edwardkim/rhwp/actions/runs/41941236661)와 실제 fast-pass·최종 정책 status·fresh-base merge-tree를 확인했고, [COMMENT self-review](https://github.com/edwardkim/rhwp/pull/8609#pullrequestreview-5778015173)를 최종 head `1557f70b03b92f0af88fb0e6f4ef280df4b8683a`에 게시했다. 이 원 PR도 GitHub `MERGED`다. [원 PR별 확정 상태](../assets/semanticist21-20261105/remote-completion.json). 위 `Merge 후 contributor PR comment 계획`의 해당 문서·페이지 review/overlay를 실제 merge SHA에 고정해 감사 댓글을 게시한다.

후속 처리 완료: [해당 원 PR 감사 댓글](https://github.com/edwardkim/rhwp/pull/8347#issuecomment-6646332079)을 게시하고 API 본문 및 GitHub 화면의 8개 review/overlay 표시를 확인했다. 통합 PR의 전용 branch/worktree·root output·검토 debug 종료 정리도 완료했다.
