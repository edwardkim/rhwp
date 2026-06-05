# PR #1066 검토 — 시험지 미주(Endnote) 본문 하단 overflow 정합

- 검토일: 2026-05-22
- PR: https://github.com/edwardkim/rhwp/pull/1066
- 관련 이슈: `#1062` (closes), 후속 분리 `#1065`
- 검토자: Codex

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1066 |
| 제목 | Task #1062: 시험지 미주(Endnote) 본문 하단 overflow 정합 (closes #1062) |
| 작성자 | planet6897 |
| base ← head | `devel` ← `pr/task1062-endnote-overflow` |
| head SHA | `d80af1704864e5475f371995cd472511d117c0f6` |
| GitHub merge SHA | `123c15989209d7dfdf85c11a0573be92a67cef0b` |
| 상태 | OPEN / mergeable true / draft false |
| 변경 | 12 files, +722 / -6 |
| 본질 변경 | `src/renderer/typeset.rs` 1파일 |
| 문서 변경 | `mydocs/plans/task_m100_1062*.md`, `mydocs/working/task_m100_1062_stage*.md`, `mydocs/report/task_m100_1062_report.md` |
| GitHub status | API 기준 status check 없음 |

로컬 `devel`은 리뷰 시작 시 원격보다 4커밋 뒤였으므로 `git pull --ff-only`로
`1cd948d3`까지 동기화했다. PR base `be2a71c4`는 현 `devel`의 조상이며,
GitHub merge commit 기준으로도 핵심 endnote 루프 패치가 최신 `devel`의
`#1067` 변경과 충돌 없이 들어간다.

## 2. 문제와 원인

다단 시험지 문서에서 본문 컬럼 하단을 넘어 `LAYOUT_OVERFLOW`가 발생하고
한컴 2022 PDF 대비 쪽수가 부족했다.

PR의 재진단 내용:

- 본문 문단은 468개, 표 0개인데 overflow 항목의 `para_index`가 468~1181.
- `typeset.rs`의 미주 가상 문단 인덱스 산식상 이 범위는 본문이 아니라
  `endnote_paragraphs`에 해당한다.
- 종전 다단 미주 누적은 `height_for_fit`을 사용해 마지막 `line_spacing`을
  제외했다.
- 렌더러는 미주를 vpos 흐름으로 배치하므로 실제 전진은
  `last.vpos + line_height + line_spacing - first.vpos`에 가깝다.
- 결과적으로 미주당 약 6px 과소 누적되어 페이지당 미주가 과밀 배치되고,
  렌더러 단계에서 본문 하단 overflow가 발생했다.

초기 가설이었던 "본문 빈 문단 trailing line spacing drift"는 위치 오귀속으로
정정되었고, 본 PR은 미주 경로만 좁게 수정한다.

## 3. 변경 내용

핵심 변경은 `src/renderer/typeset.rs`의 미주 루프다.

- 다단(`col_count > 1`) 미주의 fit/누적 값을 렌더러 vpos 전진 기준으로 계산한다.
- `advance = last.vpos + line_height + line_spacing - first.vpos`.
- fit 판정은 마지막 항목의 trailing line spacing을 제외하되,
  종전 `fmt.height_for_fit` 미만으로 내려가지 않도록 floor를 둔다.
- 누적은 `advance.max(fmt.height_for_fit)`로 한다.
- 단단(`col_count == 1`)은 종전 `height_for_fit` / `total_height` 정책을 유지한다.

이 설계는 기존 다단 본문 정책을 전부 바꾸지 않고, 렌더러가 vpos 기반으로
배치하는 미주 경로만 보정한다는 점에서 범위가 적절하다.

## 4. 검토 결과

### 4.1 차단 이슈

**차단 이슈 없음.**

패치가 건드리는 영역은 `typeset.rs`의 미주 삽입 루프에 한정되어 있고,
최신 `devel`의 `#1067` float/table 변경과는 직접 겹치지 않는다. 새 산식도
렌더러 `HeightCursor`가 이전 문단의 `vertical_pos + line_height + line_spacing`
을 기준으로 전진하는 모델과 방향이 맞다.

### 4.2 코드 리스크

- `advance` 산출이 `line_segs` 기반이고, invalid/empty line segment에서는
  `fmt.total_height`로 fallback한다.
- `fit`과 `acc` 모두 `fmt.height_for_fit` floor를 둬서 기존보다 더 조밀하게
  배치되는 회귀를 막는다.
- 단단 문서 경로는 명시적으로 유지되어 영향 범위가 다단 미주로 제한된다.
- 첫 미주 문단에 `"문N) "` prefix를 붙이는 기존 동작은 그대로이며, 이번 패치가
  새로 만든 리스크는 아니다.

### 4.3 검증 근거

PR 본문 및 보고서 기준:

- 251 샘플 `LAYOUT_OVERFLOW`: 1624 → 769 (-53%)
- 대상 시험지 3종은 PDF 쪽수 일치: 20/20, 18/18, 21/21
- 3-09 2022는 155 → 20으로 개선되나 22/23 잔여
- 골든 SVG 8종 무회귀
- 비회귀 표본: `exam_eng`, `exam_kor`, `k-water-rfp`, `복학원서`,
  `footnote-01`, `endnote-01`

1차 검토 시점에는 GitHub API 기준 status check가 비어 있었다. 이후 처리 단계에서
PR head를 현 `devel` 위에 cherry-pick한 뒤 본 환경 검증을 수행했다. 최종 검증
결과는 `mydocs/pr/pr_1066_report.md`에 기록한다.

### 4.4 문서 정합 메모

PR 본문은 `cargo test --release: 1557 passed / 0 failed`라고 쓰고,
`mydocs/report/task_m100_1062_report.md`는 `1550 passed / 0 failed`라고 쓴다.
코드 차단 이슈는 아니지만, 하이퍼-워터폴 감사 로그 관점에서는 병합 전 숫자를
하나로 맞추는 것이 좋다.

### 4.5 잔여 범위

PR 본문과 보고서가 잔여를 `#1065`로 분리했다.

- 3-09 2022 잔여 20건
- 소폭 악화 4파일(+8, 3~23px)
- 원인은 "typeset 미주 분할점 ↔ 렌더러 vpos base reset 지점 미정렬"로
  분석되어, 이번 trailing line spacing 과소 누적 정정과는 별도 축이다.

잔여를 숨기지 않고 별도 이슈로 분리한 점은 scope 정직성 측면에서 적절하다.

## 5. 1차 판단

| 영역 | 판단 |
|------|------|
| 설계 | 적합 — 미주 경로만 vpos 전진 기준으로 정정 |
| 범위 | 적절 — `typeset.rs` 미주 루프 한정 |
| 충돌 | GitHub mergeable true, 최신 `devel`과 본질 충돌 없음 |
| 검증 | 정량 개선 근거 충분, 단 본 환경 재검증 필요 |
| 잔여 | `#1065`로 분리되어 merge blocker 아님 |
| 문서 | 테스트 passed 수 불일치 정정 권고 |

**잠정 결론**: 코드상 차단 이슈는 발견하지 못했다. 이후 cherry-pick 통합,
WASM 빌드, `cargo fmt --check`, `cargo test --release --lib`, 작업지시자 시각
판정까지 통과하여 PR #1066은 cherry-pick 방식으로 처리 완료했다.
