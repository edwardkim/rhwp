---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 56 — issue2007 p11→p12 제목 소유권

## 목적

PR 후보 `829d6420e`에서 다시 발견된
`issue2007_nested_cell_pagination_42065.hwp` p11→p12 경계를 한컴 2020 PDF와 맞춘다.
Stage 55의 PR 준비는 이 결함을 해결하고 정확한 새 HEAD를 검증할 때까지 보류한다.

## 재현과 정답지

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 기준 등급: Hancom Office 2020 PDF oracle
- 현재/기준 쪽수: 17/17
- 비교 범위: p11–p12, 144dpi

PDF p11은 `국세기본법`의 마지막 문장으로 끝나고, p12가
`3 중앙선거관리위원회`로 시작한다. 현재 rhwp는 제목과 다음 점선 표 상단선을 p11 하단에
미리 그리며 p12를 `공직선거법` 본문부터 시작한다. 따라서 쪽수 정합만으로는 통과시킬 수
없는 source-owner 회귀다.

## 구조 증거

- 바깥 continuation: `PartialTable(pi=7, ci=1)`
- p11 cut: `[168] -> [226]`
- p12 cut: `[226] -> [271]`
- p11에 잘못 남은 제목: source paragraph `pi=89`, `y=978.1`, `h=17.3`
- 제목 직후 다음 중첩 표 상단 조각: `y=997.3`, `h=3.8`
- p11 visual accuracy proxy: `6.63872%`
- p12 visual accuracy proxy: `6.25419%`

`fidelity_compare --layout-ledger`는 p11의 `table_footer=1`과 p11→p12의 동일 source 표
fragment를 후보로 잡았다. 반면 `visual_sweep`의 구조 heuristic은 두 쪽을 `flagged=0`으로
놓쳤다.

## 기존 회귀의 오판

`issue_2007_single_cell_continuation_does_not_repaint_boundary_fragments`는 p12에서
`contains_painted_text(..., "중앙선거관리위원회")`를 검사한다. p12 본문의
`중앙선거관리위원회규칙`도 substring으로 일치하므로 실제 제목이 없어도 통과한다.

회귀는 다음 exact owner 계약으로 바꾼다.

1. p11에는 trim한 정확한 TextRun `중앙선거관리위원회`가 없어야 한다.
2. p12에는 trim한 정확한 TextRun `중앙선거관리위원회`가 있어야 한다.
3. 기존 p12 선행 문장 비재도색 및 p16→p17 소유권 계약은 유지한다.

exact helper와 p11 negative assertion을 먼저 추가한 뒤 focused test를 실행했다. 기존
구현에서는 다음과 같이 의도대로 실패했다.

```text
test issue_2007_single_cell_continuation_does_not_repaint_boundary_fragments ... FAILED
p11 must not paint the p12-owned heading after an explicit page break
```

따라서 새 회귀는 기존 substring 오탐을 제거하고 실제 결함을 재현한다.

## 원인 분석

source의 제목 앞 `cp88`은 단순 여백이 아니라 `ColumnBreakType::Page`인 명시적 쪽
나누기 문단이다. 순서는 `cp87` 국세청 중첩 표, `cp88` 빈 쪽 나누기 문단, `cp89`
중앙선거관리위원회 제목, `cp90` 공직선거법 중첩 표다.

일반 body typesetter는 문단의 `ColumnBreakType::Page`를 강제 쪽 경계로 처리한다. 그러나
`cell_units_uncached`는 셀 문단의 `column_type`을 확인하지 않고 저장 vpos reset만으로
`hard_break_before`를 만든다. 게다가 Task #1488의 빈 overlay 보호 조건이 비가시 빈 문단의
reset을 제거한다. 그 결과 명시적 쪽 나누기인 `cp88`도 장식용 빈 overlay처럼 접히며,
`cp89`와 `cp90`의 3.76px sliver가 p11에서 소비된다.

이 결함은 기존 `RecursiveBlockPreludeRole`의 fit 보정이 제목 바로 뒤의 block만 보는 탓에,
제목 뒤 3.76px 재귀 prefix까지 이미 들어간 형상을 놓친 문제다. 다만 `Page/Section`을
일반 hard break로 승격하면 저장 프레임 원장과 중복된다. 따라서 기존 prelude role에
`ExplicitPageBreakSeparator`만 추가해, **이미 들어간 prefix까지 되감을 수 있는 source
경계인지**만 구분한다.

## 기각한 가정

셀의 명시적 Page/Section 표식을 별도 strict 경계로 전파하는 첫 구현을 시험했다. 새 단위
회귀와 #1488은 통과했지만 issue2007 전체 쪽수가 17에서 19로 늘고 p15–p17의 기존 PDF
소유권 계약이 깨졌다. 1×1 RowBreak로 범위를 좁혀도 결과는 같았다. 이 표식은 저장된
물리 프레임 원장과 중복되어 새 물리 쪽을 추가하므로 채택하지 않는다.

## 실패한 중간 보정

첫 번째 prelude 확장은 separator와 제목뿐 아니라 이미 들어간 재귀 prefix도 일반적으로
되감았다. p11은 `[168,226) -> [168,223)`으로 바로잡았지만 p13의 일반 빈 separator에도
같은 규칙이 적용되어 제목과 작은 prefix만 가진 sliver 쪽이 생겼다. 결과는 18쪽이었고,
기존 p15-p17 source-owner 회귀 5개가 실패했다.

두 번째 보정은 다음 prelude의 separator만 p12 끝에 소비되는 경우도 되감았다. p12는
`[223,272) -> [223,271)`이 됐지만, p13에서 일반 prelude prefix rewind가 다시 발동해
`[271,274)` sliver가 남았다. 따라서 prefix-aware rewind 자체를 명시적 source 쪽 나누기로
한정해야 했다.

## 최종 구현

실제 p11 walk는 separator와 제목뿐 아니라 다음 재귀 block의 3.76px 첫 prefix까지 현재
쪽에 넣은 뒤, 그 다음 재귀 unit에서 overflow한다. 최종 규칙은 다음과 같다.

1. pending unit에서 뒤로 연속된 `recursive + nontrailing + role=None` prefix를 건너뛴다.
2. 가장 가까운 `OneLineHeadingBeforeSingleCellTable`과 바로 앞 separator를 요구한다.
3. 이미 들어간 prefix까지 되감는 경우는 separator가 source의 `Page/Section`에서 온
   `ExplicitPageBreakSeparator`일 때만 허용한다. 일반 `EmptySeparator`의 기존 direct-next
   보정은 유지한다.
4. 다음 제목 앞 separator 하나만 현재 조각에 들어간 경우에는 그 separator를 함께
   되감아 다음 쪽의 고아 제목을 막는다.
5. hard break, stored frame break, vpos gap, trailing, 비재귀 unit에서 탐색을 멈춘다.
6. `separator_idx <= start`이면 되감지 않아 새 viewport의 무진행 cut을 금지한다.

수정은 파일명·페이지·문구 특례 없이 source 구조와 실제 fit만 사용한다. focused 회귀를
먼저 실패시키고 최소 구현 보정 후 p11–p13 PDF 대조를 다시 수행했다.

## 결과

- 정확한 페이지 수: rhwp 17 / 기준 PDF 17
- outer `PartialTable(pi=7, ci=1)` cut:
  - p10 `[115,168)`
  - p11 `[168,223)`
  - p12 `[223,271)`
  - p13 `[271,282)`
  - p14 `[282,331)`, p15 `[331,381)`, p16 `[381,417)`, p17 `[417,end)`
- p11: 정확한 제목 `중앙선거관리위원회`와 다음 표 상단선이 제거됨
- p12: 정확한 제목이 첫 source owner로 복원됨
- p13 이후: 기존 PDF 경계와 p15-p17 회귀 계약 유지
- `layout-candidates`: p11-p13 모든 구조 후보 0
- `text-owner-shift`, `text-owner-sequence`, `visible-text-excess`: 후보 0

검증 결과:

```text
cargo test --profile release-test --lib \
  recursive_block_prelude_rewinds_already_fit_prefix_before_overflow
1 passed; 0 failed

cargo test --profile release-test --test issue_2007_nested_cell_pagination
15 passed; 0 failed

cargo fmt --all -- --check
git diff --check
PASS
```

144dpi 페이지별 직접 대조에서 p11은 `국세기본법`으로 끝나고 p12는
`3 중앙선거관리위원회`로 시작한다. overlay의 낮은 절대 ink 지표는 이 문서 전반의 기존
글꼴/자간 차이를 함께 집계하므로, 이번 단계의 합격 판정은 exact TextRun owner, 17쪽 cut
원장, p11-p13 육안 대조와 구조 원장을 함께 사용했다.

## 증적

- [p11 review before](../pr/assets/task_m100_3820_stage55_pr_readiness/review_p011_before.png)
- [p12 review before](../pr/assets/task_m100_3820_stage55_pr_readiness/review_p012_before.png)
- [overlay metrics before](../pr/assets/task_m100_3820_stage55_pr_readiness/overlay_metrics_before.json)
- [layout ledger before](../pr/assets/task_m100_3820_stage55_pr_readiness/layout_candidates_before.tsv)
- [p11 review after](../pr/assets/task_m100_3820_stage56_issue2007_p11_heading_owner/review_p011_after.png)
- [p12 review after](../pr/assets/task_m100_3820_stage56_issue2007_p11_heading_owner/review_p012_after.png)
- [p13 review after](../pr/assets/task_m100_3820_stage56_issue2007_p11_heading_owner/review_p013_after.png)
- [p11-p13 contact sheet after](../pr/assets/task_m100_3820_stage56_issue2007_p11_heading_owner/review_p011_p013_after.png)
- [page cut dump after](../pr/assets/task_m100_3820_stage56_issue2007_p11_heading_owner/page_cut_dump_after.json)
- [overlay metrics after](../pr/assets/task_m100_3820_stage56_issue2007_p11_heading_owner/overlay_metrics_after.json)
- [layout ledger after](../pr/assets/task_m100_3820_stage56_issue2007_p11_heading_owner/layout_candidates_after.tsv)
