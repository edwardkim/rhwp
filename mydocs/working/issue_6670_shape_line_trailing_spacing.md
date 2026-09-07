---
kind: working
status: active
canonical: mydocs/working/issue_6670_shape_line_trailing_spacing.md
issue: 6670
last_verified: 2026-09-07
---

# #6670 — 도형 줄 꼬리 줄간격 뒤에 드러난 두 문서의 어긋남

## 1. 판정

**#6665 수정(도형 전용 줄 뒤 꼬리 줄간격)은 맞다.** CI 의 실패 3건은 그 수정이 드러낸
기존 어긋남이다.

- `hwp3-sample16` 3쪽 시험 두 개는 한/글 PDF 글자 **top** 을 rhwp `y`(baseline 계열)와
  맞추고 있었다. 수정 전 어긋남(도형 뒤 −10.4px)이 그 22.8px 차이와 우연히 겹쳐 숫자만
  맞았다. → 한/글 baseline 간격으로 재고정(이 PR).
- `2025 행정업무운영 편람` 285쪽 조각 넘침(5.8px)은 25pt 라벨 셀(`10.`/`11.`)의 행 성장
  때문이다. 한/글은 저장 lineseg 한 줄짜리 글자 셀이 줄 상자로 선언 안쪽을 넘어도 행을
  키우지 않고 글자를 괘선 밖으로 그린다. rhwp 는 #5751 이래 행을 내용+여백으로 키우는
  쪽을 택했고, 글자 넘침을 회귀로 세는 기준선(`text_overlap_baseline`,
  `overflow_cell_baseline`)이 그 결정을 지킨다. 한/글 쪽으로 바꿔 보니(§4) 그 기준선과
  오라클 쪽수 기준선이 17건 깨졌다 — 이 PR 에서는 하지 않는다. 그 시험 한 건은 빨간 채
  메인테이너 판단에 맡긴다.

## 2. 근거 — 저장 사다리

세 문서 모두 도형 문단 → 다음 문단 저장 vpos 가 `lh + ls (+ sa)` 와 HWPUNIT 단위로 같다.

| 문서 | 사다리 |
|---|---|
| 3-09월_교육_통합_2024 | pi=203: 2551 + 452 = 3003 = pi=204 vpos |
| 2025 행정업무운영 편람 s10 | pi=21: 2400 + 500 + 1000 = 3900 = pi=22 vpos |
| hwp3-sample16-hwp5 | pi=71: 5760 + 9764 + 780 = 16304 = pi=72 vpos |

### 전수 계측 (samples 856개 문서)

바닥값 블록에 임시 계측기(`RHWP_FLOOR_DOC`)를 심고 `export-render-tree` 로 전부 렌더했다.

| | 값 |
|---|---|
| 블록 도달 문서 / 이긴 문단 | 131 / 465 |
| 꼬리 ls>0 로 실제 움직인 문단 (문서) | 440 (104) — 대개 6~13px, 최대 31.2 |
| "도형이 글줄보다 큼"(주석의 원래 목적) | 1 |
| 움직인 문단 중 덤프로 사다리 대조 가능 | 181 |
| 그중 `다음.vpos − (vpos + lh) = ls (+ sa/2)` (한/글도 꼬리 ls 넣음) | **181** (172 + qsb 포함 9) |
| 반대(`sa/2` 만) / 기타 | **0 / 0** |

대조 불가: hwpx 중복 191, 하위 폴더 덤프 없음 33, 같은 pi 도형 문단 복수 25, 다음 문단이 다른 쪽 10.
`y_before = 문단 top + ls + sa` 로 나오는 것은 `layout_paragraph` 이 도형 전용 줄의 줄 높이를 세지
않는다는 뜻이다 — 바닥값 블록은 그 누락을 메우는 자리이고, 이 수정으로 결과 값은 한/글과 같아진다.
스크립트: `floor_census.sh` · `floor_census_summary.py` · `ladder_check.py`(세션 scratchpad).

## 3. sample16 3쪽 — `pdf/hwp3-sample16-2020.pdf` baseline 대조

| 줄 | 한/글 baseline | 수정 전 `y` (y−base) | 수정 후 `y` (y−base) |
|---|---|---|---|
| 우리공사 전산기… (도형 앞) | 204.0 | 192.7 (−11.3) | 192.7 (−11.3) |
| 비즈니스 연속성… (도형 앞) | 260.0 | 248.2 (−11.8) | 248.2 (−11.8) |
| 2. 추진방향 (도형 뒤) | 360.0 | 337.2 (−22.8) | 347.6 (−12.4) |
| 터의 전산기… | 424.0 | 402.4 (−21.6) | 412.8 (−11.2) |
| 재해복구시스템… | 513.3 | 490.6 (−22.7) | 501.0 (−12.3) |
| 3. 주요 추진내용 | 772.0 | 749.3 (−22.7) | 759.7 (−12.3) |

수정 후에는 쪽 전체가 같은 상수(−11.2~−12.4). 새 시험은 도형 앞 첫 본문 줄에서 제목까지의
거리를 한/글 baseline 거리(156.0 / 568.0)와 비교한다. 수정 전 코드(`upstream/devel`)에서
제목 시험은 실패하고, BCP `립` 접힘 시험은 계약(같은 줄에 접힘)이 양쪽에서 성립해 통과한다.

## 4. 편람 285쪽 — 25pt 라벨 셀, 시도와 철회

| | 값 |
|---|---|
| 저장 lineseg | `vertsize=2500 textheight=2500 spacing=2000` (33.3px), charPr height 2500 |
| 선언 셀 | 2349HU = 31.3px, 여백 566×2 → 안쪽 16.2px |
| rhwp | `DIAG_ROWH r=1 c=3 decl=31.3 req=48.4 content=33.3 pad=15.1` |
| 한/글 PDF 괘선 | 표 199.3→464.4(선언 265.1), 1행 위 211.8, rowspan 셀 아래 265.3(53.5), 글자 bbox 213.9→246.3 |
| 한/글 사다리 | 다음 문단 vpos 24349 − 3900 = 20449 = 선언 19883 + 바깥 아래 여백 566 |

`samples/*.hwp` 30문서에서 다음 문단 vpos 가 "선언 그대로"인 자리차지 표 158개 · 2405 셀 중
**한 줄짜리 저장 셀 501개**가 같은 모양(hwpctl_ParameterSetID 249 · 편람 224 · 정책연구 14 …).

시도: `composer::stored_single_line_text_cell_overflows_declared`(문단 1·저장 줄 1·개체 없음·
`lh > 안쪽`)를 HeightMeasurer(depth 0, native HWP5, 비-TAC)와
`table_layout::row_cut_content_height` 양쪽에 걸어 선언 높이를 쓰게 했다.
결과(편람): `10.` 행 48.4→31.3, rowspan 셀 53.5(한/글 일치), 표 284.7→269.6, 285쪽 조각
안쪽, s10 pi=63·65 가 한/글처럼 같은 297쪽, 쪽수 384→383. 그러나 전체 스위트 9021 중 **17 실패**:

- `issue_5751_dense_row_grows_to_content_plus_padding` — 행 성장 계약 자체
- `text_overlaps_do_not_grow_partition_{1,10}` · `overflow_cell_lines_do_not_grow_partition_{7,11}` ·
  `off_canvas_does_not_grow_partition_11` — 글자 넘침 기준선
- `page_counts_do_not_drift_from_hancom_oracle_partition_{1,8,11}` — 오라클 쪽수 기준선
- `issue_2439 …`, `issue_3820 …`, `issue_6307 …`, `issue_rowbreak_chart_overlap …`,
  wasm_api `issue2424`·`issue2214`, `text_security::scan_cost_stays_linear`(부하 중 timing 가능)

즉 한/글의 "안 키우고 넘친다"는 rhwp 의 기준선과 정면으로 충돌한다. 철회하고 브랜치
`backup/6670-cell-rule` 에 두었다. 쪽수 383/384 도 근거를 남긴다: 우리 281쪽 첫 항목(s10
pi=10)이 한/글은 282쪽이라 **281쪽부터 이미 한 쪽 앞서** 있고, 384 는 그 −1 을 pi=65
자리의 +1(한/글은 63·65 를 한 쪽에 둠)이 상쇄한 값이다.

`LAYOUT_OVERFLOW`(s10 pi=14~17, 25~66px)는 규칙 전·후·`upstream/devel` 모두 같다.

### 왜 셀 높이가 곧장 이 시험으로 이어지는가 — walk 추적

```
DIAG_PRESPLIT pi=23 remaining=384.1 blk=(0,3,81.1)
DIAG_SCAN CUT_TRY r=4 budget=303.4 consumed_h=303.4 end_cut=[1, 1, 12]
DIAG_SPLITSCAN pi=23 consumed=416.2 avail=384.1
```

4행의 산술 예산은 ≈283(11줄)인데 `source_tail_cut` 확장이 저장 프레임 꼬리(12줄, 303.4)로
예산을 늘리고, 초과는 `stored_frame_tail_overflow = (조각 높이 − avail_for_rows).max(0)` 로
그대로 허용된다(자리차지 RowBreak 는 #5584 ② 의 24px 상한 밖). 조각 **경계**는 한/글 저장
프레임(12+4줄), 조각 **높이**는 rhwp 행 높이 — 둘이 한/글과 같을 때만 안전하다. 렌더러는
`#3931` 회수 규칙으로 첫 조각을 앞선 표 바닥에 붙이지만(호스트 `sa+ls` 20.0 회수) 쪽 나눔
커서는 회수하지 않아, walk 의 초과(32.1)와 실제 페인트 초과(5.4)가 다르다.

대안 B(그 초과에 본문 하한 상한, 넘치면 용량 컷으로 물러남)는 편람 핀들의 확장
15.3~107.4px 이 같은 축이라 좁지 않고, 이 시험의 12+4 계약도 11+5 가 된다. A·B 모두
제품 방향이라 #6842 에 넘겼다.

## 5. 하지 않은 것

- 빈 호스트 자리차지 표 뒤 간격(rhwp `ls + sa` 20.0 vs 한/글 바깥 여백 7.55). 코퍼스 350쌍
  근거로 별도 이슈. 렌더러의 #3931 회수 규칙이 첫 조각을 앞선 표 바닥에 바로 붙이므로 이
  PR 의 실패와 무관하다.

## 6. 검증

- `cargo fmt --all -- --check` · `cargo clippy -- -D warnings` · wasm clippy
- `cargo nextest run --no-fail-fast` — `issue_3931_pi23_stored_reset_splits_across_adjacent_pages`
  1건만 빨강(§1).
- 환경: macOS 15 (Darwin 25.6), 한/글 PDF 는 저장소 `pdf/` 의 2020·2024 판, pymupdf.
- 시각 근거: `mydocs/pr/assets/issue_6670_20260907/`.
