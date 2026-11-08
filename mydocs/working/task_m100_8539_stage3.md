# #8539 3단계 — 구현과 코퍼스 대조

- 기준: `upstream/devel` `7d06d5c70` (2026-11-03)
- 계획: [task_m100_8539.md](../plans/task_m100_8539.md), 원인: [stage2](task_m100_8539_stage2.md)

## 1. 수정 v1과 회귀

v1은 2단계 시험 패치 A·B를 정식 코드로 옮긴 것이다.

- A: `advance_row_cut_inner`의 relaxed 저장 재시작 흡수 판정이 용량 컷과 같은 끝 줄간격 제외로 다음 줄을 잰다.
- B: 이어받은 조각의 시작 컷이 저장 프레임 경계가 아니면 `source_frame_origin_unverified`를 켠다
  (`row_start_owns_stored_frame` 재사용).

대상 문서는 `[2,3,3]`·`[10,8,10]`이 됐다. 그러나 저장소 1,424개 문서의 `dump-pages` 대조에서
6개 문서의 표 컷이 바뀌었고(쪽수 변화 없음), 그중 4개가 한컴 PDF와 크게 어긋났다
(Native raw, devel → v1).

| 문서 | 바뀐 경계 | 뒤쪽 점수 |
|---|---|---|
| `issue6549/16418295_square_rowbreak_table.hwp` | 1→2쪽 | 100.00 → 33.53 |
| `issue6929/148776468_search_ad_terms_press_release.hwp` | 10→11쪽 | 99.96 → 52.85 |
| `issue1853_caption_precedes_body_split.hwpx` | 15→16쪽 | 99.53 → 64.27 |
| `issue1937_rowbreak_footnote_overpagination.hwp` | 24→25쪽 | 99.92 → 91.23 |

이 문서들에서는 한/글이 저장 재시작을 쪽 경계로 지킨다. 따라서 "끝 줄간격을 빼면 들어가면 흡수"는
일반 규칙이 아니다. 특히 16418295 6행은 대상 문서 9행과 판정 수치가 거의 같다
(쌓인 높이 51.2, 내용 예산 70.13, 다음 줄 25.6). 그런데도 한/글은 재시작을 지켰다.
문단 모양(외톨이줄·문단 보호 없음, 줄간격 160%)과 저장 버전(148776468도 5.0.3.4)으로도 둘을 가를 수 없다.

## 2. 선언 행 상자로 좁힌 v2

남은 차이는 행의 선언 상자(병합하지 않은 칸 저장 높이의 최댓값)다.

| 문서·행 | 선언 상자 | 남은 물리 공간 | 재시작 앞 내용 | 한/글 |
|---|---|---|---|---|
| 대상 9행 | 82.32px | 84.15 — 상자가 든다 | 51.2 < 상자 내용 67.2 | 재시작 무시, 3줄 |
| 16418295 6행 | 88.83 | 73.89 — 상자가 넘친다 | — | 재시작 따름 |
| 148776468 12행 | 264.0 | 220.77 — 상자가 넘친다 | — | 재시작 따름 |
| issue1937 6행 | 38.4 | 상자가 든다 | 221.9 > 상자 내용 | 재시작 따름 |

대상 문서 9행의 선언 상자 82.32px는 `3줄(67.2, 끝 줄간격 제외) + 위아래 안 여백(15.12)`과 같다.
rhwp가 이미 이 크기로 조각 상자를 예약했는데, 내용 컷만 2줄에서 멈췄다.

v2 규칙: 행의 첫 조각(`start == 0`)에서 선언 상자가 이 쪽 남은 공간에 통째로 들고, 저장 재시작이 그 상자의
내용 높이 안쪽에 있을 때만 재시작을 흡수한다. 다음 줄 수용은 용량 컷과 같은 끝 줄간격 제외(`trimmed_capacity_fit`)로
선언 상자 내용 높이 안에서 잰다. 그 밖의 경우는 종전과 같다.

- 선언 상자 높이는 `stored_restart_end_row_box`가 쓰던 계산을 `declared_row_frame_height` helper로 옮겨 함께 쓴다.
- 용량 컷의 끝 줄간격 제외 계산은 `trimmed_capacity_fit` closure 하나로 묶고, 용량 컷과 선언 상자 판정이 같이 쓴다.
  용량 컷 쪽 동작은 바뀌지 않는다.
- B는 v1과 같다.

v2(debug)로 v1에서 바뀐 7개 문서를 다시 출력하면, 대상 문서만 바뀌고 나머지 6개는 devel과 같다.

## 3. 소비 지점

| 단계 | 위치 | 값 |
|---|---|---|
| 유닛 높이 | `cell_units` | 9행 1·2열 줄 25.6px(끝 줄 16.0) |
| 재시작 흡수 판정 | `advance_row_cut_inner` relaxed 조건 | 선언 상자 내용 67.2 안에서 셋째 줄 수용(67.2) |
| 용량 컷 | 같은 함수 `trimmed_capacity_fit(j, h, avail)` | 셋째 줄 끝 줄간격 9.6 제외 수용, 넷째 줄 이월 |
| 물리 밴드 | `advance_row_cut_for_physical_band` | 재맞춤 없음(초과 0) |
| scan 예약 | `row_step.rs` `CUT_BASE` | `end_cut=[2,3,3]`, 내용 67.2 |
| 조각 상자 | `FRAGMENT override` | 82.32px(선언 상자, 변화 없음) |
| 이어받은 조각 | `fragment/scan.rs` → `source_frame.rs` | 시작 `[2,3,3]`은 저장 프레임 경계 아님 → 원점 미인증 → 16행 원본 프레임 꼬리 미적용 |
| 16행 컷 | `CUT_BASE` | 재시작이 남은 공간 안이라 흡수, `[10,8,10]` |

## 4. 회귀 테스트

`tests/cases/issue_8539_rowbreak_stored_restart_cut.rs` (3개). 기대값은 한/글 2020 PDF의 쪽별 줄과 원본 저장 줄 수에서 정했다.

| 코드 | `row_split_lines_follow_hancom_page_boundaries` | 보존·겹침 검사 2개 |
|---|---|---|
| devel `7d06d5c70` | FAIL — 9행 `[2,2,2]` ≠ `[2,3,3]` | PASS |
| A만(B 없음, v1) | FAIL — 16행 `[9,8,9]` ≠ `[10,8,10]` | PASS |
| v1 | PASS | PASS |
| v2 | PASS | PASS |

보존·겹침 검사는 #8198의 증상(누락·겹침) 재발을 막는 경계 검사이며, 이번 결함을 검출한다고 주장하지 않는다.

## 5. v2 검증 결과

검증한 코드는 rustfmt 적용 전 v2 작업 트리다. rustfmt는 줄바꿈만 바꿨고 동작은 같다.
최종 커밋의 Rust lint·전체 nextest·TSV 재산출은 4단계에서 별도 review worktree로 수행한다.

### 코퍼스 `dump-pages` 대조 (release, `samples/**` 1,424개)

| 비교 | 컷이 바뀐 문서 | 쪽수가 바뀐 문서 |
|---|---|---|
| devel `7d06d5c70` ↔ v1 | 7 | 0 |
| devel `7d06d5c70` ↔ v2 | **1** (대상 문서만) | 0 |

v1에서 바뀐 7개 문서의 `layout-anomaly`(overflow·overlap) 수와 위치는 devel과 같았다.
산출물: `output/pr-review/8539/corpus-v1`, `corpus-v2` (ignored).

### 대상 문서 전쪽 TSV (`--silhouette-only`, 96dpi, `--compat 2022`)

글꼴: HY신명조가 없어 시스템 NanumMyeongjo.ttc만 담은 폴더를 `RHWP_FONT_PATH`로 공급했다
(이 Mac의 사용자 NanumMyeongjo.ttf와 PostScript 이름이 충돌해 Native Skia 한글이 깨지는 것을 피하려는 것).

| 출력 | 바이너리·패키지 | 1쪽 | 2쪽 | 3쪽 | 쪽수 | gate |
|---|---|---|---|---|---|---|
| Native devel `ea86816b5` | release skia | 97.72 | 57.68 | 63.72 | 3 = 3 | re_review_required |
| fresh WASM devel `ea86816b5` | `c01b3c7d…` | 97.72 | 57.69 | 63.73 | 3 = 3 | re_review_required |
| Native v2 | release skia | **99.90** | **100.00** | **100.00** | 3 = 3 | 통과 |
| fresh WASM v2 | `43f4743b…` | **99.90** | **100.00** | **100.00** | 3 = 3 | 통과 |

대표 review·overlay PNG(1~3쪽, Native·WASM)는 `output/pr-review/8539/v2-{native,wasm}-review/`에 있다.
2쪽 review를 직접 판독했다. 첫머리 「부양 인정 / 혼ㆍ사별한 경우 자녀인」과 끝 줄 「라. 「국가유공자 / 있어도 보수
또는 소득이」가 PDF와 같은 위치다.

회귀 테스트 추가 선행 조건(같은 원본·독립 PDF의 Native·fresh WASM 최저 90% 이상)을 충족했다(최저 99.90%).
