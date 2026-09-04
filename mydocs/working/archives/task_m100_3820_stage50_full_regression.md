---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 50 — 리베이스 보정 전체 회귀

## 목적

[Stage 49](task_m100_3820_stage49_recursive_partial_cache_identity.md)의 재귀 부분 표
cache identity와 마지막 source-owner viewport 보정이 issue2007 집중 fixture 밖의 표·셀
pagination 계약을 깨뜨리지 않았는지 전체 integration 회귀로 확인한다.

## 시작 기준

- 시작 commit: `fc758ff2d`
- 브랜치: `task/3820-3821-fidelity`
- 전용 target: `target/task-3820-3821-fidelity-rebase`
- issue2007 집중 회귀: 13 passed / 0 failed
- 새 프로세스 반복: 20/20 성공
- trailing reservation 음성 회귀: 1 passed / 0 failed
- focused clippy (`-D warnings`): 통과

## 검증 원칙

1. 전체 integration은 다음 명령을 한 번 실행하고 최종 exit code와 test summary까지
   기다린다. 출력 공백이나 장시간 실행을 이유로 종료하지 않는다.
2. 실패하면 같은 전체 회귀를 즉시 반복하지 않고 최초 로그의 실패 test와 assertion을
   기준으로 focused 재현·원인 분석을 수행한다.
3. 특히 이전 리베이스 과정에서 드러난 `overflow_cell_lines_do_not_grow`의 신규·증가
   항목은 baseline을 임의 갱신하지 않고 실제 쪽 밖 소실 회귀인지 먼저 판정한다.
4. 다른 worktree와 공용 target은 건드리지 않는다.

```bash
CARGO_INCREMENTAL=0 \
CARGO_TARGET_DIR=target/task-3820-3821-fidelity-rebase \
cargo test --profile release-test --tests
```

## 결과

첫 전체 실행은 compile 3분 5초 뒤 issue2007을 포함한 선행 테스트를 통과하고 다음
focused 회귀에서 중단됐다.

```text
issue_2308_sparse_width_overlay_keeps_nested_fragment_geometry
page 34 nested fragment must preserve Hancom-aligned geometry
expected y=77.1 h=395.2
got [(276.7, 395.2133), (680.9733, 124.6667)]
```

- 전체 명령 exit code: `101`
- 재실행 명령: `--test issue_2308_render_normalized_derived_state`
- 판정: baseline 갱신 대상이 아니라 p34 nested fragment의 시작 y가 199.6px 내려간
  실제 geometry 회귀 후보다.
- 원인: Stage 49의 stable-table identity가 아니라 리베이스 충돌 체크포인트에서
  `cell_was_split` 판정의 `cut_units` 절반이 소실됐다. p34 대상 셀은
  `cut=(37, usize::MAX)`인 명시적 continuation인데, 남아 있던 `line_ranges`만으로는
  전체 line range처럼 보여 미분할로 오판했다. 그 결과 원래 `Top`이어야 할 셀이
  `Center` 정렬되어 시작 y가 `77.1 → 276.7`(+199.6px) 이동했다.
- 수정: `start_unit > 0 || end_unit < unit_len`인 명시적 unit cut을 분할 판정에
  다시 포함했다. 단일 1×1 표 full-height heuristic 제외 규칙은 그대로 유지한다.
- 정확한 실패 테스트 재검증: 1 passed / 0 failed. p34 fragment y=77.1 계약 복원.
- 다음 조치: #2308 전체 파일, issue2007, #3637 집중 회귀 후 전체 integration을
  처음부터 다시 실행한다.

## 추가 실문서 확인 — 59043 p36

사용자가 지정한 `samples/issue1921/59043_regulatory_analysis.hwp` p36을
`pdf/issue1921/59043_regulatory_analysis-2022.pdf` p36과 최신 commit으로 144dpi
직접 대조했다. 동일하지 않으며 별도 blocker다.

- review: `tmp/pdfs/stage50-59043-p36/59043-p36/review/review_036.png`
- overlay: `tmp/pdfs/stage50-59043-p36/59043-p36/overlay/overlay_036.png`
- 내용 픽셀 보조 일치율: 11.15719%
- rhwp: 페이지 대부분을 빈 2열 표가 차지하고 본문 TextRun이 소실됨
- PDF: 흡연율 분석 본문, 중간 3열 소표, 후속 계산 문단이 p36 안에 표시됨
- rhwp의 `②피규제 이외 일반국민 :`과 `□ 편익`은 같은 y=1029.6에 겹침
- 표 하단 raster 좌표: rhwp y=1576, PDF y=1422(약 154px 지연)
- 진단: p35 source가 p36 화면에 나타나지 않고 `LAYOUT_OVERFLOW_CELL` 및
  `PartialTable overflow=1208.2px`가 발생함

단순 한 쪽 밀림이 아니다. 양쪽 p37은 모두 일반국민 편익 본문으로 시작하므로 p36의
source가 화면 밖으로 소실된 것이다. 현재 visual sweep의 `flags=[]`는 전체 표 mask가
line-band 5/44, max drift 1419.5px, large ink region 1/21 신호를 제거한 false negative다.

`cell_was_split`의 unit-cut 판정을 복원한 최신 바이너리로 p36을 다시 산출해도 이
결함은 그대로다.

- 최신 비교: `tmp/pdfs/stage50-59043-p36-cutfix/cmp-p035.png`
- 최신 pixel diff: 9.21%
- 텍스트층: 기준 PDF 전용 문자 886자, SVG 전용 문자 0자
- 페이지 수: PDF 37 / rhwp render tree 37

따라서 전역 페이지 수나 #2308의 valign 회귀가 아니라 p36 source unit이 2열 partial
table의 가시 viewport에 들어오지 못한 독립적인 표 조각 소유 결함이다.

물리 child table 높이가 page body를 넘을 때 fragment 합계만으로 atom 판정하지 않도록
`calc_nested_table_height()` 기반 gate를 추가했다. 이 중간 수정으로 p36의 근거 본문,
3×3 소표, 계산 문단과 후속 제목은 다시 표시되지만 저장 경계 직후 문단의 첫 줄이 p35
하단에 조기 노출된다.

- p35 비교: `tmp/pdfs/stage50-59043-p35-p36-physical-split/cmp-p034.png`
- p36 비교: `tmp/pdfs/stage50-59043-p35-p36-physical-split/cmp-p035.png`
- pixel diff: p35 20.20%, p36 16.31%
- 신규 회귀 test: `regulatory_59043_page36_keeps_nested_source_and_following_headings`
- 현재 결과: p35에 `흡연율 감소 효과 추정 근거`가 조기 노출되어 assertion 실패

fallback mixed-fragment 경로가 canonical `CellUnit`의 hard boundary를 잃는 것이 남은
원인 후보다. 현재 구현과 실패 회귀는 upstream rebase 전 체크포인트로 보존하며 해결
완료로 판정하지 않는다.

## 최신 upstream 리베이스와 #4159 충돌 보정

Stage 50 체크포인트를 최신 `upstream/devel` `3a16ddd40` 위로 리베이스했다. 리베이스 후
HEAD는 `757fa69e9`이며, `upstream/devel...HEAD`는 `0/64`로 최신 upstream을 모두
포함한다. 충돌 해소에서는 upstream #4159의 terminal nested-cell clip 계약과 이
브랜치의 issue2007 1×1 continuation viewport 계약을 모두 보존했다.

리베이스 직후 issue2007 integration 15종 중 다음 한 건이 실패했다.

```text
issue_4159_terminal_nested_bottom_border_is_inside_all_cell_clips
비종료 물리 2쪽에 종료 bottom 선이 미리 노출됐다:
bbox=(101.24, 996.34, 653.8533, 0.5)
```

실패 선은 upstream #4159의 terminal clip 확장이 아니라 이 브랜치의 fragment-frame
재구성 경로가 만든 합성선이었다. 물리 p2의 실제 대상은 비종료 9×2 표였지만,
`ends_after_clip`이라는 순수 기하 조건만으로 셀 clip 하단에 전체 폭 선을 추가했다.
반면 한컴 PDF 기준으로 합성 하단선이 필요한 p10 대상은 비종료 1×1 RowBreak
continuation이다.

따라서 `reconstruct_nested_table_fragment_frame()`와
`repair_clipped_nested_table_fragment_frame()`의 **하단 가로 frame 재구성만** 실제
대상 `TableNode`가 1×1일 때 허용했다. 다음 동작은 변경하지 않았다.

- continuation 상단 frame 재구성
- 좌·우 세로선과 paint-safe `frame_bottom` 좌표
- upstream #4159의 실제 terminal 셀 clip 확장
- p10 비종료 1×1 하단 frame

보정 후 상반된 두 focused 계약과 integration 파일 전체가 통과했다.

```text
issue_4159_terminal_nested_bottom_border_is_inside_all_cell_clips: 1 passed
issue_2007_continuation_frame_restarts_and_drops_previous_page_residual: 1 passed
issue_2007_nested_cell_pagination: 15 passed / 0 failed
```

이 결과는 p2 9×2 표의 조기 종료선을 제거하면서 p3의 실제 terminal 하단선과 p10의
비종료 1×1 physical fragment frame을 동시에 보존한다. 전체 release-test, Native Skia
3종, clippy는 이 focused 보정 커밋 뒤 별도로 실행해 최종 상태를 판정한다.

## 집중 회귀 진행

`cell_was_split` unit-cut 복원 후 다음을 확인했다.

- #2308 derived-state: 4 passed / 0 failed
- #2308 guard: 1 passed / 0 failed
- issue2007: 13 passed / 0 failed
- #3592: 2 passed / 0 failed
- #3595: 2 passed / 0 failed
- #3637 첫 게이트: 31쪽 기대, 32쪽 산출로 실패

#3637은 unit-cut 판정을 잠시 제거한 음성 실험에서도 32쪽 그대로였으므로 이번 최소
복원의 결과가 아니다. Stage 48/49에서 이미 생긴 말미 빈 페이지 owner 회귀로 분리해
원인을 추적한다.

추가 commit 비교에서 pre-rebase `405a8e4c7`은 31쪽과 p26/p27 source-owner assertion을
통과하지만, 현재 `fc758ff2d`는 32쪽이다. 차이는 `nested_table_mixed_fragment_heights()`의
canonical `CellUnit` 조기 반환이 direct HWPX에도 적용되면서 pi197 말미 행의 fragment가
한 조각 증가한 경로로 좁혀졌다. 단일 guard 또는 nested 높이 차감만 적용하면 owner 소실
또는 30쪽 과보정이 발생하므로, 최신 upstream rebase 후 HWP5-origin과 direct HWPX profile을
구분해 검증된 legacy fallback 세 동작을 함께 복원할지 재판정한다.

## WASM 사용자 검증 대기

현재 working tree의 renderer 수정본으로 사용자 확인용 WASM을 다시 만들었다. 이 빌드는
로컬 focused 회귀를 대체하지 않으며, 브라우저에서 실제 페이지 경계·표 조각을 사람이
확인하기 위한 중간 산출물이다.

```text
명령: CARGO_INCREMENTAL=0 \
      CARGO_TARGET_DIR=target/task-3820-3821-fidelity-wasm \
      wasm-pack build --target web --out-dir pkg
결과: exit code 0, release compile 2m 08s, wasm-opt 포함 총 2m 23s
산출물: pkg/rhwp_bg.wasm (7,668,654 bytes)
SHA-256: 2a442d1ddba9fedfe5f47c30ab108f214b09532f49546402dcc243b186d666e4
```

사용자 확인이 필요한 우선 대상은 `59043_regulatory_analysis.hwp` p35–p36의 nested table
소유 경계와 `issue2007_nested_cell_pagination_42065.hwp`의 기존 문제 페이지다. 사용자가
확인하기 전에는 시각 결함 해결 완료로 판정하지 않는다.

## Stage 종료

최초 전체 회귀의 #2308 충돌 누락과 후속 59043/#3637 상충 원인을 분리하는 데까지를
이 Stage의 범위로 종료한다. 59043의 단일 저장 경계, 중복 scalar 보정, direct HWPX의
PR #4122 투영 회귀 수정과 집중 검증 결과는
[Stage 51](task_m100_3820_stage51_59043_hwpx_projection_repair.md)에 이어 기록했다.
