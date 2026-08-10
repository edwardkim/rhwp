---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 120 — stored-reset 표 fragment geometry

## 범위

Stage 119에서 공통 Body-top 상단선 clip은 해결했지만, 정책연구의 다음 세 `1×1`
RowBreak 표에 동일한 paint geometry 결함이 남았다.

- p167→168 `pi=1775`
- p172→173 `pi=1806`
- p213→214 `pi=2548`

세 표의 페이지별 cell text owner와 순서는 한컴 PDF와 일치한다. 이 단계는 owner와
pagination을 유지하면서 첫 fragment의 과대 paint height와 successor fragment의
outer-margin paint origin만 고친다.

## 현상

| 표 | PDF 첫 fragment 높이 | 현재 높이 | 오차 |
|---|---:|---:|---:|
| `pi=1775` p167 | `233pt` | `243.225pt` | 약 `+10.2pt` |
| `pi=1806` p172 | `153pt` | `163.2pt` | 약 `+10.2pt` |
| `pi=2548` p213 | `113pt` | `123.225pt` | 약 `+10.2pt` |

successor 높이는 각각 PDF `413/253/153pt`와 일치하지만 현재 origin은
`x=70.875pt, y=62.4pt`, PDF는 약 `x=74pt, y=65pt`다. 차이는 source의 네 방향
outer margin `283HU=2.83pt`와 일치한다.

## 원인 1 — reset 직전 trailing line spacing 중복

세 표의 HWP5 저장 LINE_SEG는 cell 문단 내부에서 다음과 같이 `vpos=0`으로 rewind한다.

- `pi=1775`: reset 전 마지막 `vpos=22000`, text height `1000`
- `pi=1806`: reset 전 마지막 `vpos=14000`, text height `1000`
- `pi=2548`: reset 전 마지막 `vpos=10000`, text height `1000`

`last_pre_reset.vpos + text_height + inner_top 141 + inner_bottom 141`은 declared
table height `23282/15282/11282HU`와 정확히 같다. 그러나 현재 `CellUnit`은 reset
직전 줄도 `height + line_spacing`으로 만들고 `row_cut_content_height`가 이를 첫 cut에
합산한다. 그 결과 trailing line spacing `1000HU=10pt`가 첫 fragment paint height에
한 번 더 들어간다.

## 원인 2 — successor outer paint origin 누락

`layout_partial_table`은 successor fragment의 table/cell/frame subtree를 body origin에
직접 둔다. 이 저장 계약의 outer-left/top `283HU`는 pagination budget과 owner에는 이미
반영돼 있지만 paint origin에는 반영되지 않는다. Stage 115 p120과 마찬가지로 flow가
아닌 paint subtree만 이동해야 한다.

## 최소 판정 계약

fixture paragraph ID가 아니라 다음 구조를 모두 만족할 때만 적용한다.

1. native HWP5, top-level empty-host single table
2. `1×1`, non-TAC, `TopAndBottom/RowBreak`
3. `Para/Top + Column/Left`, x/y offset 0, caption 없음
4. 네 방향 outer margin이 양수·동일
5. cell stored LINE_SEG가 paragraph 안에서 `vpos=0`으로 rewind
6. 첫 cut이 reset 직전 line 끝과 일치
7. declared height가 reset 전 마지막 `vpos + text_height + cell top/bottom padding`과 일치

## 수정 계약

- 첫 fragment의 paint/clip height만 declared stored head height로 제한해 reset 직전
  trailing line spacing을 제외한다.
- logical consumed height, flow advance, PageItem owner는 유지한다.
- 첫/successor table·cell·frame subtree의 paint x/y에 outer-left/top을 더한다.
- width, successor height, pagination budget과 후속 본문 좌표는 바꾸지 않는다.
- Stage 119의 table top-stroke inset은 그대로 유지한다.

## 구현

`native_hwp5_stored_reset_fragment_paint_geometry`는 fixture 문단 번호 없이 위 최소
판정 계약을 검사한다. 첫 fragment에서는 `common.height`로 증명된 저장 head 높이까지만
row/cell/frame paint를 제한하고, 잘라 낸 trailing spacing은 반환 flow 높이에 다시 더해
logical consumed와 다음 문단 좌표를 유지한다. 첫 fragment와 최종 successor fragment의
table/cell/frame paint subtree에는 source outer-left/top만 복원한다.

이 보정은 top-level outer table에만 적용된다. 중첩 표, TAC 표, caption 표, 비대칭 margin,
저장 reset 증거가 없거나 cut이 reset 경계와 다른 표는 모두 기존 경로를 사용한다.

## 회귀 계획

- p167/p168, p172/p173, p213/p214의 exact table/cell/frame bbox
- 각 페이지 cell paragraph owner 및 중복 0
- 첫 fragment height `233/153/113pt`, successor height `413/253/153pt`
- successor origin 약 `74/65pt`
- p33 일반 body-top 표, Stage115 p120, #2439, #2097 불변
- `issue_2430_cell_rewrap_threshold`를 코드 수정 후 첫 gate로 실행

## 검증 결과

실물 회귀는 정책연구 215쪽을 유지하면서 다음 여섯 물리 fragment를 고정했다.

| 구간 | 첫 fragment cell 높이 | successor cell 높이 | paint origin |
|---|---:|---:|---|
| p167→p168 `pi=1775` | `23282HU` | `41282HU` | body/host + `283HU` |
| p172→p173 `pi=1806` | `15282HU` | `25282HU` | body/host + `283HU` |
| p213→p214 `pi=2548` | `11282HU` | `15282HU` | body/host + `283HU` |

각 쪽의 cell paragraph owner는 기준 범위를 유지했고 첫 fragment와 successor 사이의
중복 owner는 0건이다. table/cell/frame의 좌·우·상·하 선도 보정된 cell 경계와 일치한다.

집중 회귀 결과는 다음과 같다.

- stored-reset helper unit: `2/2`
- 정책연구 stored-reset 실물 회귀: `1/1` (6쪽 계약)
- `issue_2430_cell_rewrap_threshold`: `2/2`
- Stage 119 Body-top 상단선: `2/2`
- `issue_3738_rowbreak_table_footnote_fragment`: `33/33`
- `issue_2439`: `4/4`
- `issue_1772_table_outer_margin_sync`: `2/2`
- #2097 관련 표 배치: `4/4`
- `cargo clippy --profile release-test --all-targets -- -D warnings`: 통과

최신 release-test CLI로 세 구간을 한컴 PDF와 직접 다시 비교했다. 요청 6쪽은 모두
완료됐고 누락, 인접 text owner-shift, owner-sequence, page-boundary, visible text excess
후보는 0건이다. 직접 확대 판독에서도 첫 fragment의 약 10pt 과대 높이가 사라졌고,
successor의 좌·상단 frame이 PDF와 같은 outer-margin 원점으로 이동했다. raw pixel diff는
p167/p168 `12.46/15.83%`, p172/p173 `14.13/12.48%`, p213/p214
`12.44/13.74%`이며, 이 수치는 글꼴 raster 차이를 포함하므로 구조 판정 대신 쓰지 않았다.

## 증적

- [p167 보정 전](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p167_before.png)
- [p168 보정 전](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p168_before.png)
- [p172 보정 전](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p172_before.png)
- [p173 보정 전](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p173_before.png)
- [p213 보정 전](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p213_before.png)
- [p214 보정 전](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p214_before.png)
- [p167 보정 후](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p167_after.png)
- [p168 보정 후](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p168_after.png)
- [p172 보정 후](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p172_after.png)
- [p173 보정 후](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p173_after.png)
- [p213 보정 후](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p213_after.png)
- [p214 보정 후](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/compare_p214_after.png)
- [p172~173 픽셀 보고서](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/p172-p173-report-before.tsv)
- [p167~168 보정 후 픽셀 보고서](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/p167-p168-report-after.tsv)
- [p172~173 보정 후 픽셀 보고서](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/p172-p173-report-after.tsv)
- [p213~214 보정 후 픽셀 보고서](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/p213-p214-report-after.tsv)
- [실행 provenance](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/provenance-before.tsv)
- [보정 후 실행 provenance](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/provenance-after.tsv)
- [보정 후 실행 완료 원장](../pr/assets/task_m100_3820_stage120_policy_stored_reset_fragment_geometry/run-state-after.tsv)

Stage 120의 stored-reset fragment geometry 범위는 완료했다. 정책연구 전체 #3820의
완료를 뜻하지 않으며, p33 TAC 표 뒤 URL 줄 소유권 등 확인된 잔여 결함은 다음 stage에서
별도 원인 계약으로 계속 처리한다.
