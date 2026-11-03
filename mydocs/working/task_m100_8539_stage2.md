# #8198 2단계 — 한 줄 이른 행 분할의 원인

- 기준: `upstream/devel` `ea86816b5`, release `rhwp` (진단 실행은 같은 커밋의 복사본)
- 입력: law.go.kr `flSeq=169566169`(HWP 5.0.3.4), 기준 한컴 2020 Print PDF `flSeq=169566171`
- 계획: [task_m100_8198.md](../plans/task_m100_8539.md)

## 1. 어느 분기가 컷을 정했나

`RHWP_DIAG_SCAN=1 rhwp dump-pages`:

```text
CUT_BASE r=9 avail=880.493 prior=796.347 padding=15.120 content_budget=69.027 height=41.600 end_cut=[2, 2, 2]
FRAGMENT rows=0..10 cut=[2, 2, 2] override=Some(82.32)
CUT_BASE r=16 avail=900.480 prior=647.333 padding=15.120 content_budget=238.027 height=220.800 end_cut=[9, 8, 9]
```

- 저장 재시작 전용 분기(`STORED_RESTART_PAGE_CUT`, `row_step.rs`)는 타지 않았다.
- 컷은 일반 물리 밴드 컷 `advance_row_cut_for_physical_band` → `advance_row_cut_inner`
  (`layout/table_layout.rs`)에서 나왔다.

`RHWP_DIAG_CELL_UNITS="9,1|9,2|16,0|16,2"`:

```text
DIAG_RESET_CUT row=9 col=1 next=2 height=51.2 available=69.026667 unit_height=25.6 hard=true strict=false
DIAG_RESET_CUT row=9 col=2 next=2 height=51.2 available=69.026667 unit_height=25.6 hard=true strict=false
DIAG_RESET_CUT row=16 col=0 next=9 height=230.4 available=238.026667 unit_height=25.6 hard=true strict=false
DIAG_RESET_CUT row=16 col=2 next=9 height=230.4 available=238.026667 unit_height=25.6 hard=true strict=false
```

모든 경계가 `strict=false`다. 저장 프레임 인증(선언·본문·plain zero 등) 없이,
relaxed 규칙의 "다음 줄이 들어가지 않음" 판정으로 저장 재시작에서 잘렸다.

## 2. 두 판정이 다른 높이를 쓴다

`advance_row_cut_inner`의 같은 유닛 반복문 안에 다음 줄의 수용 판정이 두 번 있다.

| 판정 | 위치 | 쓰는 높이 | 9행 셋째 줄 |
|---|---|---|---|
| 저장 재시작 흡수(relaxed) | `h + unit_fit_height > avail_height` | 끝 줄간격 포함 `25.6` | 51.2 + 25.6 = 76.8 > 69.03 → 재시작에서 자름 |
| 일반 용량 컷 | `row_cut_capacity_trailing_trim` → `cell_cut_capacity_tail_trim` | 끝 줄간격 9.6 제외 | 51.2 + 25.6 − 9.6 = 67.2 ≤ 69.03 → 수용 |

- `unit_fit_height`는 #7870(`bf4fc67e5`)이 "relaxed 흡수 판정이 용량 컷과 같은 점유 높이를 쓴다"는
  원칙으로 도입했다. 그때는 중첩 표 atom의 보이지 않는 이송 여백만 맞췄고 끝 줄간격 제외는 넣지 않았다.
- relaxed 규칙의 출처는 `a6b1df069`(2026-06-23)이고, 적용 범위는 이후
  `(col_count <= 2 || row_count > 5)` RowBreak로 넓어졌다. 이 표는 18×3이라 해당한다.

## 3. 독립 기준과의 대조

- 한컴 2020 PDF 1쪽: 9행 가운데·오른쪽 칸 셋째 줄 기준선은 1001px(96dpi)로 1쪽 안에 있다.
- rhwp가 그린 1쪽 9행 조각 상자는 82.32px다. 이것은 `3줄(67.2) + 위아래 안 여백(15.12)`과 같다.
  즉 상자는 셋째 줄까지 담을 크기로 예약됐는데, 내용 컷만 2줄에서 멈췄다.
- 16행(2→3쪽): 끝 줄간격을 빼도 230.4 + 16.0 = 246.4 > 238.03이라 지금 배치에서는 들어가지 않는다.
  2쪽 첫머리가 한 줄(25.6px)을 더 받아 예산이 줄었기 때문이다. 1→2쪽 컷이 3줄이 되면
  예산은 약 263.6px가 되어 열째 줄(246.4)이 들어가고 열한째 줄(272.0)은 들어가지 않는다 → `[10, 8, 10]`.
  이 값은 한컴 PDF와 같다(가설, 시험 패치로 확인 필요).

## 3-1. 시험 패치 A — 1→2쪽 확인

relaxed 판정의 다음 줄 높이에 용량 컷과 같은 `row_cut_capacity_trailing_trim`·`cell_cut_capacity_tail_trim`을
적용했다(진단용, 커밋하지 않음 — `output/pr-review/8198/probe-8198-AB.diff`).

```text
PROBE_8198 row=9 col=1 j=2 h=51.200 unit=25.600 avail=69.027 trim=9.600 trimmed=67.200
CUT_BASE r=9 content_budget=69.027 height=67.200 end_cut=[2, 3, 3]
```

1→2쪽은 `[2, 3, 3]`으로 한컴과 같아졌다. 2→3쪽은 여전히 `[9, 8, 9]`다.

## 3-2. 두 번째 원인 — 이어받은 조각이 어긋난 저장 쪽 경계를 따른다

A를 적용한 뒤 16행 진단:

```text
CUT_BASE r=16 cursor=9 content_budget=263.627 height=246.400 end_cut=[10, 8, 10]
CUT_TAIL r=16 enabled=true extension=-16.000 owns=true candidate_cut=[9, 8, 9]
CUT_TRY  r=16 budget=230.4 end_cut=[9, 8, 9]
```

- 일반 컷은 예측대로 `[10, 8, 10]`(예산 263.6px)이다.
- 이어서 `row_step.rs`의 원본 프레임 꼬리 단계가 `source_frame_contracts_capacity`로 저장 프레임 컷 `[9, 8, 9]`를
  채택한다. 이 저장 프레임은 `source_frame.rs`에서 `native_stored_row_has_text_frame_boundary`로 인증된다.
- 저장본의 2쪽 프레임은 9행 `[2, 2, 2]`에서 시작해 16행 `[9, 8, 9]`에서 끝난다. A 적용 후 이번 조각은
  `[2, 3, 3]`에서 시작하므로, 이 조각의 끝을 저장본 2쪽의 끝으로 인증할 근거가 없다.
- 같은 문제를 막는 `source_frame_origin_unverified`가 이미 있다("현재 컷이 원본 프레임 중간이면 저장 재시작을 이
  물리 쪽의 끝으로 강제할 수 없다"). 그러나 이 값은 `continuation/fragment/scan.rs`에서 첫 조각에만 계산되고,
  이어받은 조각에는 적용되지 않는다.

시험 패치 B: 이어받은 조각에서 이 값을 강제로 켰다(모든 이어받은 조각에 적용하는 진단 스위치이며 정식 조건이 아니다).

| 시험 | 1→2쪽 | 2→3쪽 | Native TSV raw (1/2/3쪽) |
|---|---|---|---|
| devel `ea86816b5` | `[2,2,2]` | `[9,8,9]` | 97.72 / 57.68 / 63.72 |
| A | `[2,3,3]` | `[9,8,9]` | — |
| A + B | `[2,3,3]` | `[10,8,10]` | 99.90 / 100.00 / 100.00 |
| 한컴 2020 PDF | `[2,3,3]` | `[10,8,10]` | — |

fresh WASM 기준선은 97.72 / 57.69 / 63.73이다. 두 경우 모두 쪽수는 3 = 3이다.
글꼴은 HY신명조가 없어서, 시스템 NanumMyeongjo.ttc만 담은 폴더를 `RHWP_FONT_PATH`로 공급했다.

## 4. 계획서 4절 가설의 정정

- 계획서는 "5.0.3.4 저장본의 낡은 저장 경계"를 원인 후보로 보고, 저장 버전 등으로 판별 조건을 찾으려 했다.
- 실제로는 relaxed 규칙이 이미 "다음 줄이 들어가면 저장 재시작을 흡수"하는 계약을 갖고 있다.
  다음 줄이 들어가는지를 잴 때 용량 컷과 다른 높이를 쓰는 것이 원인이다. 따라서 저장 버전에 따른 분기는 필요 없다.
- 21761835 반례는 버전 판별에 대한 반례였으므로 직접 관련이 줄어든다. 다만 수정 후 같은 판정 경로를 타는지는
  5단계 코퍼스 대조에서 확인한다.

## 5. 결론과 다음 단계

- 원인은 두 겹이다. ① relaxed 흡수 판정과 용량 컷이 다음 줄 높이를 다르게 잰다.
  ② 이어받은 조각이 저장 쪽 경계 체인과 어긋난 위치에서 시작해도 그 뒤의 저장 프레임 끝을 인증한다.
- ②는 ①을 고친 뒤에야 드러난다. 저장본과 같은 위치에서 시작한 조각에서는 저장 프레임을 지금처럼 인증해야 한다.
- 메인테이너 의견(2026-11-02, issuecomment-6579028228)에 따라 이 차이는 #8198에서 분리한 새 이슈에서 다룬다.
  새 이슈에서 할 일:
  1. B를 정식 조건으로 좁힌다. 이어받은 조각의 시작 컷이 저장 프레임 경계와 일치하지 않으면 원점 미인증으로 본다.
  2. 저장소 RowBreak 표 문서에서 수정 전후 `dump-pages` 컷·쪽수 차이를 수집하고, 바뀐 경계를 독립 PDF로 판정한다.
  3. 반례 후보: 한컴이 저장 재시작을 지키고 다음 줄을 넘긴 경계 중, 끝 줄간격을 빼면 들어가는 경우.
