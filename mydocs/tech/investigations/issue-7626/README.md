---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-10-07
---

# #7626 — 미배치 줄 캐시와 문단 끝 글자 상자

[이슈 #7626](https://github.com/edwardkim/rhwp/issues/7626)의 기준 source는
`7076f836e2300f7d760d74b58c468cc0f73098a2`다. 원본·한컴 재저장 HWP·독립 Print PDF의
출처와 해시는 [재현 자료](../../../../samples/issue7626/README.md)에 고정했다.
원본의 52개 LineSeg는 모두 폭과 원점이 0인 미배치 기록이다. 한컴 PDF는 2쪽이지만
기준 Native는 1쪽이고 마지막 본문은 y=1278.4px까지 내려가 용지 밖을 넘는다.

## 원인과 공통 결과

1. 폭 0인 원본 기록은 외부 분할 줄로 분류되면서 재조판을 거절한다. 글자 크기로
   보정한 실제 줄 높이와 달리 쪽 예산은 원본의 짧은 vpos span을 재사용한다.
   `compute_render_normalized`에서 **구역 전체의 source 줄에 폭·가로 원점·세로 원점이
   모두 없는 경우**에만 파생 렌더 사본의 캐시를 제외한다. 표 셀·캡션도 같은 사본에서
   제외하고 재구성하며 원본 document/composed·저장 정보는 보존한다.
   폭만 0인 유효 높이 사다리, 구현이 생성한 줄, HWP3 저장 기하는 이 조건에 넣지 않는다.
2. 캐시 제외만 적용하면 2쪽이 되지만 최저 Sweep은 77.80%다. 표 앞 p30의 가시 글자는
   10pt이고 끝의 빈 run은 12pt다. 한컴 재저장 줄 높이 1200HU·진행 1920HU와 달리
   재조판이 1000HU·1600HU만 예약해 표와 뒤 본문이 약 4.27px 위로 당겨진다.
   `layout_paragraph_in_frame_impl`은 `ParagraphEnd`를 채운 **마지막 물리 줄에만**
   종단 CharShapeRef의 크기를 반영한다. 이 `FrameRowMetrics`에서 줄 높이·줄간격·기준선을
   함께 게시하며 텍스트 폭이나 앞선 줄은 바꾸지 않는다.

소비 경로는 `compute_render_normalized`의 파생 문단/구성 →
`layout_paragraph_in_frame_impl`의 프레임 줄 → `resolve_line_metrics`의 formatted 높이 →
HeightMeasurer·pagination의 쪽 예산 → 실제 layout의 줄/표 배치다.
원본의 미배치 높이를 이후 fit/flow에 재적용하는 대신 일반 no-cache 경로에서 같은 줄을 소비한다.
편집 후 파생 문단 갱신에도 같은 캐시 조건을 적용한다. 편집 저장본의 독립 Print 비교는 미검증이다.

## 집중 회귀

`tests/cases/issue_7626_unplaced_lineseg_pagination.rs`는 CLI의 최종 render tree를 검사한다.
원본 전체 Native/fresh WASM 최저 98.71%를 확인한 뒤 추가했다.
기대값은 원본 Print의 쪽 소속과 한컴 재저장 줄 메트릭에서 정하며 절대 표 원점이나 SVG 해시를 고정하지 않는다.

| 검사 | 기준 source CLI | 수정 CLI |
| --- | --- | --- |
| 41개 본문/표 문단의 쪽 소속·순서, 본문 내부 포함, 본문/표 token 누락·중복 | FAIL: 1쪽 | PASS: 2쪽 |
| 끝의 빈 run 줄 상자와 다음 표 진행·표 뒤 주석 관계 | FAIL: 1쪽 | PASS |
| 한컴 재저장본의 유효 줄 진행과 폭 0 표 host 보존 | PASS | PASS |

실행은 같은 정식 Rust test source를 `rustc --test`로 컴파일하고
`CARGO_BIN_EXE_rhwp`를 각각 보존한 기준 CLI와 수정 CLI로 고정했다.
기준 exit 101: 1 PASS/2 FAIL, 수정 exit 0: 3 PASS/0 FAIL이다.
파생 integration suite를 primary checkout에서 준비하거나 변경하지 않았다.
전체 Cargo integration suite·Clippy·코퍼스 래칫 및 신규 sample 보안 게이트는 아직 실행하지 않았다.

정상 대조군 8개는 동일한 `export-svg --profile print --font-style` 명령으로 전쪽 비교했다.
쪽수와 SVG가 모두 수정 전과 동일하다. 해시는 진단용 비교이며 회귀 golden으로 추가하지 않았다.

| 대조군 | 쪽수 |
| --- | ---: |
| `253E164F57A1BC6934-empty.hwp` | 2 |
| `hwp3-empty-cell.hwp` | 1 |
| `issue1639_empty_host_negative_offset_float.hwpx` | 2 |
| `issue1639_empty_host_positive_only_float.hwpx` | 2 |
| `issue1880_anchor_stack_sb_convert.hwpx` | 13 |
| `issue1880_takeplace_host_before.hwpx` | 10 |
| `basic/BlogForm_BookReview.hwp` | 1 |
| `tac-case-003.hwp` | 1 |

## 시각 검증

Windows Native debug/fresh WASM dev, print profile, Chrome webfont rasterizer,
96dpi, `--embed-fonts full --font-path C:\Windows\Fonts`로 동일 원본과 독립 PDF의 2쪽 전체를 비교한다.
후보 작업 트리의 양쪽 최저 `tolerant_content_match_percent`는 98.70902%이며 gate는 `passed`다.
최종 코드 commit의 재출력 결과·페이지별 TSV·대표 review/overlay PNG를 이 절에 고정한다.
엄격 픽셀 ink match는 별도 지표이며 90% 실루엣 gate의 의미로 바꾸어 보고하지 않는다.