# PR #415 처리 보고서 — Task #352 dash 시퀀스 Justify 폭 부풀림 정정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#415](https://github.com/edwardkim/rhwp/pull/415) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#352](https://github.com/edwardkim/rhwp/issues/352) (closes #352) |
| 처리 결정 | **옵션 A (Task #352 7 commits 분리 cherry-pick)** |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 본 PR 정황 분석

PR #415 의 40 commits 가 다른 OPEN PR 들 (#401, #406/#408, #410, #414) 의 변경분 누적 형태. 본 task #352 핵심 commit 은 7 개. 전체 PR 머지 시 **PR #401 의 synam-001 회귀** (작성자 재정정 대기) 가 함께 도입되는 정황.

→ **Task #352 7 commits 만 분리 cherry-pick** 결정 (옵션 A).

### Stage 1: cherry-pick

`local/pr415` 브랜치 (`local/devel` 분기) 에서 7 commits 만 cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `c0a5592` (← `428b01d`) | @planet6897 | Stage 1: 원인 확정 (HY신명조 dash 메트릭 853/1024 em) |
| `61db5d6` (← `69e420b`) | @planet6897 | Stage 2: dash advance 자연 폭 보정 (leader-aware) |
| `5b473f1` (← `2248752`) | @planet6897 | Stage 3: dash run 시각 라인 통합 (underline 일치) |
| `a515cd9` (← `7f45fc0`) | @planet6897 | Stage 4: 최종 결과 보고서 |
| `e77beae` (← `037cba6`) | @planet6897 | 폭 보정 0.32 em → 0.5 em (PDF 실측) |
| `363b7bb` (← `6926c4b`) | @planet6897 | Stage 5: elastic Justify 분배 (PDF 모방) |
| `368849b` (← `3dc8049`) | @planet6897 | WASM 측정 경로 dash leader 패치 |

cherry-pick 결과:
- 첫 4 commit 자동 머지 성공
- Stage 4 (`7f45fc0`) 에서 `mydocs/orders/20260428.md` modify/delete 충돌 (devel 에는 본 파일 없음) → 작성자 버전 그대로 추가 후 `--continue`
- 나머지 3 commit 자동 머지 성공

### Stage 2: 검증 (회귀 점검)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1037 passed** (1031 → +6 신규 Task #352 테스트) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 21s, 4,117,073 bytes (PR #413 시점 → +9,901 bytes) |

### Stage 3: synam-001 회귀 분리 확인 (핵심)

`samples/synam-001.hwp` 5 페이지 — PR #401 회귀가 본 PR 에 도입되지 않았는지 확인:

| 항목 | devel | PR #415 cherry-pick 후 | PR #415 전체 머지 시 (참고) |
|------|-------|----------------------|---------------------------|
| 전체 페이지 수 | 35 | **35** ✅ | 37 (PR #401 회귀 도입) |
| 페이지 5 의 PartialTable pi=69 | rows=0..5 | **rows=0..5** ✅ | rows=0..2 (회귀) |

→ **옵션 A 분리 cherry-pick 으로 PR #401 회귀 정황 회피 성공**.

### Stage 4: 작업지시자 시각 판정

| 시나리오 | 결과 |
|---------|------|
| `samples/exam_eng.hwp` 5 페이지 Q32 dash 시퀀스 처리 | ✅ 통과 |

작성자 정량 결과 (Stage 5 elastic 분배):

| 항목 | devel | PR #415 적용 | PDF 목표 |
|------|-------|-------------|---------|
| Q32 dash advance | 12.11 px | **7.06 px** | ~7.4 px |
| 29 dash 폭 | 351 px | **204.7 px** | ~218 px |
| dash 글리프 | 29 개 | 0 개 (line 대체) | 0 개 |

작업지시자 시각 판정: **dash 시퀀스 처리 정상 — 통과**.

## 변경 요약

### 본질 — Task #352 dash leader Justify 폭 정정 (5 단계)

| Stage | 내용 |
|-------|------|
| 1 | 원인 확정 — HY신명조 dash 메트릭 비정상 (853/1024 em) + Justify Branch A min-clamp |
| 2 | dash advance 자연 폭 보정 (`is_dash_leader_run` ≥3 연속 dash 만 0.3 em 강제 좁힘) |
| 3 | dash run 시각 라인 통합 (`<text>` 글리프 → 단일 `<line>`, PDF 동일 표현) |
| 4 | 폭 미세 보정 (PDF 실측 반영) |
| 5 | elastic Justify 분배 — `extra_dash_advance` 신규 필드 + word slack → dash slack 흡수 |
| WASM | text_measurement.rs 의 leader 패치를 WASM 경로에도 적용 |

### 변경 파일 (실 코드)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/text_measurement.rs` | `is_dash_leader_run()` + 세 char_width 클로저 leader-aware 적용 |
| `src/renderer/layout/paragraph_layout.rs` | Justify 7 분기에 `extra_dash_advance` 일관 적용 (3-tuple 확장) |
| `src/renderer/svg.rs` | dash run 글리프 → `<line>` 통합 |
| `src/renderer/web_canvas.rs` | 동일 패턴 (Canvas 경로) |
| `src/renderer/mod.rs` | TextStyle 신규 필드 `extra_dash_advance` |
| `tests/golden_svg/issue-147/aift-page3.svg` | golden snapshot 갱신 (dash 글리프 → line 대체) |

## 시각 판정 결과

| 항목 | 결과 |
|------|------|
| `samples/exam_eng.hwp` 5 페이지 Q32 dash leader | ✅ 통과 |
| `samples/exam_eng.hwp` 6 페이지 Q33 공백 팽창 해소 | (Stage 5 elastic 분배 효과, 작성자 보고 95.6 px) |
| 다른 hwp 샘플 (exam_kor / exam_math / aift / biz_plan) | 무회귀 (작성자 보고) |

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1037 + svg_snapshot 6/6 + clippy 0 + WASM 빌드 |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 직접 판정 통과 |
| PR 댓글 톤 | ✅ |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr415` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |
| **본 PR 의 다른 PR 누적 정황 점검** | ✅ Task #352 7 commits 만 분리 cherry-pick — synam-001 회귀 회피 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr415` → `local/devel` → `devel` 머지 + push
3. PR #415 close + 작성자 댓글 (분리 cherry-pick 정황 안내 + 다른 task 들의 자체 PR 흐름 권장)

## 참고

- 검토 문서: `mydocs/pr/pr_415_review.md`
- PR: [#415](https://github.com/edwardkim/rhwp/pull/415)
- 이슈: [#352](https://github.com/edwardkim/rhwp/issues/352)
- 본 PR 누적 다른 task PR (분리 처리): #401 (Task #398, 작성자 재정정 대기), #406, #408 (Task #402/#404), #410 (Task #409), #414 (Task #412, base=main)
