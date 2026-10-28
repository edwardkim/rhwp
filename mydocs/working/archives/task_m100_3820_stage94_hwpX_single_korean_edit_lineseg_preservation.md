---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 94 — HWPX 단일 문자 편집의 저장 LineSeg 보존

## 시작 근거

Stage 93의 12pt 한양신명조 metric 보정 후 #2214, #2424와 #2020 focused gate는 통과했다.
전체 release-test는 `issue_2185_korean_break_unit`에서 HWPX만 아래처럼 실패했다.

```text
expected [0, 44, 84, 122]
actual   [0, 45, 87, 125]
```

같은 giant-cell fixture의 HWPX 저장 lineseg는 원본 문서가 가진 줄 경계다.
끝 한 글자 추가가 기존 네 줄의 폭·문자 수·실제 line-flow를 바꾸지 않는다면 그 경계를
재조판해 drift시키면 안 된다.

## Stage 93과의 구분

- #2214/#2424: 끝에 ASCII를 누적 삽입해 실제 5번째 줄 경계를 넘기는 경우다. 재래핑과
  flow signal이 필요하다.
- #2185: 문단 끝에 ASCII 한 글자를 추가하되 새 줄을 만들지 않는 경우다. target cell geometry와
  기존 줄 수가 불변이면 저장
  LineSeg를 유지해야 한다.

따라서 metric을 다시 완화하지 않는다. 편집 종류·문자 수·reflow 필요성에 따라 stored
LineSeg를 보존할 수 있는 기존 incremental edit 경로를 조사한다.

## 재현·원인 분석

`issue_2185_korean_break_unit` 단독 실행도 같은 HWPX failure를 재현했다. test는 130자
문단 끝에 ASCII `1` 하나를 추가한다. 저장 LineSeg의 마지막 시작은 122이므로 이 edit는
마지막 줄만 영향을 받으며, 새 다섯째 줄은 아직 만들지 않는다.

`issue1949_giant_cell_nested_tables_perf.hwpx`에는 RHWP HWP5-origin marker가 없으므로,
HWP/HWPX container provenance는 이 분기의 기준이 될 수 없다.
`DocumentCore::reflow_cell_paragraph_with_edit`는 `source_format == Hwp`일 때만
`reflow_line_segs_after_cell_text_edit`를 호출하고, HWPX는 `reflow_line_segs` 전체 재래핑으로
보낸다. 전자는 token boundary가 유효한 저장 prefix를 보존한 뒤 영향 줄부터 다시 채우는
공용 helper인데, 현재 source-format gate 때문에 HWPX는 `[0,44,84]` prefix까지
`[0,45,87]`로 drift한다.

Stage 93의 61번째 ASCII boundary는 이와 양립한다. 130자 끝 edit에서는 마지막 segment만
reflow해 4줄을 유지하고, 누적 입력이 마지막 줄 폭을 넘으면 helper가 마지막 줄에서 시작해
정상적으로 다섯째 줄을 만든다. 즉 `FileFormat`이 아니라 저장 LineSeg/token boundary의
유효성이 재래핑 범위를 결정해야 한다.

### 반례와 최종 분기

첫 보정으로 HWPX에도 prefix helper를 그대로 적용했을 때 #2185는 통과했지만 #2214가
56번째 입력에서 조기 flow signal을 냈다. 저장 마지막 줄만 채운 local reflow가 다섯째 줄을
예측하는 순간, HWPX adapter의 전체 문단 재조판은 아직 네 줄(실제 boundary 61)을 유지하기
때문이다.

따라서 HWPX의 조건은 두 단계다.

1. 저장 LineSeg 수를 넘지 않는 edit: prefix-preserving reflow 결과를 유지한다. (#2185)
2. local suffix reflow가 새 줄을 만들려는 edit: 곧바로 full reflow로 다시 판정한다.
   이때만 adapter 문단 전체의 61회 boundary를 사용한다. (#2214/#2424)

HWP는 기존 동작처럼 prefix reflow를 그대로 유지한다. 이 조건은 source-format 자체가
저장 경계를 폐기하는 근거가 아니라, HWPX 새 줄 생성 여부를 판단한 뒤 full layout이 필요한
경우에만 좁게 쓴다. HWPCTRL 호환 문서는 API/저장 결과의 검증 범위만 정하므로, 이 판단의
시각 정답지는 한컴 PDF와 해당 fixture의 저장 LineSeg다.

## 계획

1. #2185를 단독 재현하고 HWP/HWPX의 edit 전후 text·char shape·LineSeg를 비교한다.
2. delete/insert가 하나의 replace로 처리되는지, line-seg reflow guard가 저장 HWPX에도
   적용되는지 확인한다. — 완료: HWPX format gate가 helper 적용을 막는다.
3. source 수정 전 #2430을 확인하고, 유효 저장 LineSeg를 가진 HWP/HWPX text edit 모두에
   공용 prefix-preserving reflow를 적용한다. — 1차 적용 뒤 #2185 통과/#2214 조기 boundary
   반례 확인.
4. HWPX에서 local suffix reflow가 새 줄을 만드는 경우에만 full reflow를 수행한다.

## 수정 뒤 focused 결과

수정 뒤 `issue_2430_cell_rewrap_threshold`를 먼저 실행해 `2 passed; 0 failed`를 확인했다.
그 다음 결과는 다음과 같다.

| gate | 결과 |
| --- | --- |
| `issue_2185_korean_break_unit` | `1 passed; 0 failed` — HWP/HWPX 모두 `[0,44,84,122]` 보존 |
| `issue2214_scoped_cache_coherence_preserves_transient_pagination` | `1 passed; 0 failed` — HWPX 61번째 flow boundary와 115 fragments 유지 |
| `issue2424_resumable_delete_commits_only_after_final_fragment` | `1 passed; 0 failed` |
| `issue_2020` | `4 passed; 0 failed` — receipt PDF contract 유지 |

전체 release-test에서 #3137 cache contract가 추가로 드러나 Stage 95로 이월했다.

## 전체 회귀 결과와 이월

전체 재실행에서 #2185/#2214/#2424/#2020은 통과했으나 마지막 lib 단계가
`issue3137_focused_cell_geometry_matches_exact_rect` 하나로 exit `101`이었다.

```text
hwpx: tail input 58 patch signal
expected true, actual false
```

HWPX local suffix reflow가 56번째부터 임시로 새 줄을 만들었다가 full reflow가 네 줄로
되돌리는 동안 page-tree patch가 이미 무효화된 것으로 보인다. 최종 line-flow만으로
cache mutation signal을 계산하도록 Stage 95에서 분리한다.

## 완료 기준

- HWPX 단일 문자 edit가 `[0,44,84,122]`를 보존한다.
- 실제 ASCII growth의 #2214/#2424 boundary와 receipt PDF contract는 유지된다.
- #3137 및 전체 release-test 최종 결과는 Stage 95에서 확인한다.
