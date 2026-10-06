# Task #2287 최종 보고서 — 표 밀집 문서 과소분할 (2결함 복합 규명·수정)

- 이슈: #2287 / 브랜치: `local/task2287` / 관련: #2237, #2279, #1921, #2070, 10k 서베이 r14
- 수행 계획서: `mydocs/plans/task_m100_2287.md`

## 결론 요약

이슈의 "cut 행높이 잔여 누적" 가설을 delta staircase 로 실측 분해한 결과,
대형 음수 델타는 **서로 다른 2개 결함의 복합**이었다. 두 결함 모두 수정했다.

| 문서 | 수정 전 | 수정 후 | 한글 실측 | 잔여 |
|---|---:|---:|---:|---:|
| 범교과 연결 맵 (−40) | 375 | **380** | 415 | −35 |
| 미래부 정서분석 (−41) | 88 | **127** | 129 | −2 |
| 농촌 S-OJT (−64) | 206 | **262** | 270 | −8 |

게이트 샘플 불변: 86712=65쪽, byeolpyo4=26쪽, byeolpyo1=4쪽.

## 결함 1 — RowBreak rowspan 블록 연속 조각의 잔여 증발

`typeset.rs` 블록 컷 `rowbreak_use_row_offsets` 분기(#1486)에서 연속 조각
(start_cut 보유)의 `block_fragment_height` 는 row_span==1 셀만 집계하므로,
걸친 rowspan 셀의 잔여 유닛이 **0 으로 평가**되어 블록이 즉시 "fits" 종료된다.

- 실측 (교육부 연결맵 s1 pi0 47×9, `samples/task2287/1342000_edu_curriculum_map.hwp`):
  - 셀[13] (r=2, rs=2, 85문단+중첩표 2, 선언 178719HU≈2382px): 유닛 123개 중
    첫 조각이 26유닛(450.7px) 소비 후, 연속 조각의 잔여 97유닛(≈1700px)이 0 평가
  - 표 cut 합 6016.8px 인데 조각 소비 합 4078px — 선언 잔여 1938.6px ≈ 3.3쪽 증발
  - 렌더러는 잔여를 그대로 그려 페이지 밖 오버플로 (p26 SVG text y=3026 > 718px)
  - 표 지면 소비: rhwp 8쪽 vs 한글 11쪽 (한글 PDF p25~35 행 앵커 실측)
- 수정: 연속 조각 한정으로 `block_h`/`split_total` 을
  `row_block_content_height`(rowspan 셀 포함)와 `max()`. 첫 조각(#1486)은 불변.
- 수정 후 47×9 표: p25~35 = 11쪽 (한글과 동일 소비).

## 결함 2 — 저장 LINE_SEG 없는 TAC 그림 anchor 문단 높이 0 붕괴

빈 텍스트 + TAC(글자처럼) 그림 + 저장 LINE_SEG 부재 문단은 composed lines 가
비고 `empty_paragraph_fallback_line_metrics` 가 컨트롤 보유 문단에 None 을
반환해 **문단 플로우 높이가 0** 이 된다. 차트/스캔 그림 수십 장(그림당 ~387px,
스캔 ~855px)이 한 쪽에 응축된다.

- 실측: 미래부 pi854 (그림 156.0×102.5mm=590×387px, tac=true, ls=0) → h=0.0;
  rhwp p65 한 쪽 = 한글 p101~107. 농촌 꼬리(부록 스캔 22~29장/문단) rhwp 2쪽 =
  한글 ~26쪽. 이웃 문단 저장 vpos 델타 437px(그림+캡션)와 한글 배치 정합.
- 수정: `tac_object_stack_line_metrics`(renderer/mod.rs) — TAC 그림/도형 폭을
  가용 폭에 greedy wrap 하여 줄별 (최대 높이, 0) 합성. `format_paragraph`(typeset)
  / `measure_paragraph`(height_measurer) 의 pairs 빈 폴백에 연결. 저장 LINE_SEG
  보유 문단·비-TAC 는 제외(이중 계상/오버레이 경로 침범 방지).
- 단위 테스트 3건 (`src/renderer/mod.rs` tests).

## 59043 핀(#1921) 귀속과 국소화

결함 1 하한 보정 초기안은 59043 규제영향분석서 핀을 41→44 로 흔들었다.
동일 기반 stash 실측으로 내 수정 귀속을 확정한 뒤 분해한 결과:

- 59043 의 3×3 병리 표(pi160/163: 음수 패딩 −32768, 선언 h=282HU 에 104문단)는
  per-row 합산이 **0 이 아니라 부분값**을 주는 사례로, 측정/렌더 발산(#2237
  계열)이 겹쳐 마지막 조각에서 오버플로가 수정 전(p28 ymax=1591)·후(p29
  ymax=4727) 모두 잔존 — 하한 보정으로 고쳐지는 클래스가 아님
- 반면 교육부 47×9 는 per-row 합산이 **정확히 0**(행의 row_span==1 셀이 전부
  spacer 소진)인 완전 증발 클래스
- → 하한 보정을 `per-row 합산 == 0` 인 완전 증발 클래스로 국소화. 59043=41
  핀 복구. 최종 보완에서 교육부는 380쪽이며 p26 내용 보존과 p30 frame-tail overflow
  해소를 확인했다.

## 잔여 (후속 추적)

- 범교과 연결맵 −35: 최종 보완으로 대표 47×9 RowBreak rowspan 조각의 p26 sliver와
  p30 frame-tail overflow는 해소했다. 후속 섹션 동종 거대 표의 선언>콘텐츠 슬랙과
  그 밖의 표 cut 잔여는 [#2237](https://github.com/edwardkim/rhwp/issues/2237) 축으로
  계속 추적한다.
- 59043 병리 표(음수 패딩)의 마지막 조각 오버플로: 수정 전부터 잔존하는
  측정/렌더 발산(#2237 axis B) — 본 이슈 범위 밖, 별도 추적.
- 농촌 −8 / 미래부 −2: 그림 캡션 높이 미계상(문단당 ~50px) + 본문 드리프트.
- 결함 2 는 캡션(dir=Bottom) 높이를 줄 메트릭에 더하면 추가 수렴 여지.

## 재현·검증

```
# 결함 1 재현 샘플 (동봉, 1.3MB)
target/debug/rhwp.exe dump-pages samples/task2287/1342000_edu_curriculum_map.hwp | head -1
# 델타 계단 (한글 COM 필요)
python tools/task2287/delta_staircase.py samples/task2287/1342000_edu_curriculum_map.hwp
# 진단: RHWP_TABLE_DRIFT / RHWP_DIAG_SCAN(BLOCK_DECIDE) / RHWP_DIAG_BLKCUT / RHWP_DIAG_BLKH
```

대용량 재현 원본(corpus): `hwpdocs/prism_downloads/과학기술정보통신부/1710000-201500050_...hwp`(18MB),
`hwpdocs/prism_downloads/농촌진흥청/1390000-201200009_...hwp`(45.6MB) — 크기 제약으로 미동봉.

## 게이트

- `cargo nextest run --no-fail-fast`: **3162/3162 통과** (22 skipped, 실패 0).
  `issue_2070` 시장구조조사 잠정 핀 307→309 갱신 (정답 315 방향 +2, 잔여 −6);
  `issue_1921` 59043=41 핀 유지 (국소화로 보존).
- 92 컨트롤셋 (`tools/render_page_gate.py`): 전/후 TSV **완전 동일** — 85/92
  일치(92.4%), 기존 −1 7건 그대로, 회귀 0.
- 시각 검증: 교육부 p26 ymax 3027→741 (오버플로 해소), 59043 p24/25 = 한글
  정답지 p20/21 첫 줄 문자열 일치, 시장구조조사 연속 조각 구간(p70~73)
  ymax ≤ 975 (오버플로 없음).
- `cargo fmt --check`: 변경분 포맷 이탈 없음 (개행 스타일 경고는 기지 로컬
  CRLF 노이즈). `cargo clippy --all-targets -- -D warnings`: 통과.
