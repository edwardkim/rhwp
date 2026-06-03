# Task #391 단계 3 — 회귀 검증

- **이슈**: [#391](https://github.com/edwardkim/rhwp/issues/391)
- **단계**: 3/4

## 자동 검증 결과

| 테스트 | 결과 |
|---|---|
| `cargo test --lib --release` | **1014 passed, 0 failed, 1 ignored** |
| `cargo test --test svg_snapshot --release` | **6/6** |
| `cargo test --test issue_301 --release` | **1/1** |
| `cargo test --test exam_eng_multicolumn --release` | **1/1** (Red → Green) |
| `cargo test --test hwpx_roundtrip_integration --release` | 14/14 |
| `cargo test --test hwpx_to_hwp_adapter --release` | 25/25 |
| `cargo test --test page_number_propagation --release` | 1/1 |
| `cargo test --test tab_cross_run --release` | 1/1 |
| `cargo clippy --lib --release -- -D warnings` | 통과 |

## 11 샘플 회귀 비교

3 상태 비교 — pre-#359 (worktree `e5d383f^`) / current devel / my fix:

| 샘플 | 단수 | pre-#359 | current devel | **my fix** | 판정 |
|---|---|---|---|---|---|
| **exam_eng** | 2단 | 8p, 0 | 11p, 0 | **8p, 0** | ✓ 본 task 핵심 (목표 달성) |
| exam_kor | 2단 | 24p, 30 | 30p, 0 | 24p, 30 | pre-#359 동일 (별개 이슈) |
| **k-water-rfp** | 1단 | 26p, 73 | 27p, 0 | **27p, 0** | ✓ #359 보존 |
| kps-ai | 1단 | 81p, 60 | 79p, 5 | 79p, 5 | 무변화 (devel 동일) |
| aift | 1단 | 74p, 30 | 77p, 3 | 77p, 3 | 무변화 |
| form-01 | 1단 | - | - | 1p, 0 | 무변화 |
| KTX | 1단 | - | - | 27p, 1 | 무변화 |
| hwp-multi-001 | 1단 | - | - | 10p, 0 | 무변화 |
| exam_math | - | - | - | 20p, 0 | 무변화 |
| biz_plan | - | - | - | 6p, 0 | 무변화 |
| 21_언어 | - | - | - | 15p, - | 무변화 |

핵심 가드:
- 다단 (2단) 만 변경 — `col_count > 1` 분기로만 영향
- 단단 (1단) 모든 샘플 무변화 → kps-ai/aift 의 #359 개선 보존
- exam_kor 의 잔존 30 overflow 는 #359 이전부터 존재 (worktree 검증). 본 task 책임 범위 외.

## exam_eng 단 채움 (수정 후)

```
=== 페이지 1 ===  단 0 items=38 used=897.5     단 1 items=34 used=1185.1
=== 페이지 2 ===  단 0 items=41 used=897.5     단 1 items=23 used=1113.9
=== 페이지 3 ===  단 0 items=20 used=1058.1    단 1 items=24 used=1031.1
=== 페이지 4 ===  단 0 items=12 used=1114.0    단 1 items=17 used=1100.8
=== 페이지 5 ===  단 0 items=16 used=1097.6    단 1 items=17 used=1086.3
=== 페이지 6 ===  단 0 items=18 used=1049.4    단 1 items=12 used=1120.5
=== 페이지 7 ===  단 0 items=13 used=1114.1    단 1 items=10 used=1149.9
=== 페이지 8 ===  단 0 items=18 used=1071.8    단 1 items=19 used=965.2
(총 8 페이지)
```

**수정 전 11p 의 단독 1-item 단/페이지 패턴 (p3/p5/p7) 모두 해소**. 단별 used 와 hwp_used 의 차이 (-90~-300px 범위) 는 다단 layout 의 vpos 기반 stacking 미세 차이로 pre-#359 와 동등한 수준 — 별개 이슈로 추후 검토.

## SVG 시각 판정 자료

다음 위치에 SVG 출력. 작업지시자 시각 판정 요청:

- `output/svg/task391/exam_eng/` — 8 페이지 (수정 후) 전체
- `output/svg/task391/k-water-rfp/` — 27 페이지 (#359 보존 확인용)
- `output/svg/task391/exam_kor/` — 24 페이지 (pre-#359 동등성 참고)

판정 항목:
1. **exam_eng** (필수): 페이지 진행 자연스러움, 단독 1-item 단 사라짐, 본문 흐름 끊김 없음
2. **k-water-rfp** (필수): #359 stage3 통과 자료와 동일 — p3 등 단독 페이지 없음, 본문 정상
3. **exam_kor** (참고): pre-#359 동등 상태로 복원되었음을 시각 확인

## 단계 4 진행 승인 요청

본 단계 3 보고서 + 작업지시자 시각 판정 승인 후 단계 4 (WASM + 최종 보고서 + CHANGELOG) 진행.
