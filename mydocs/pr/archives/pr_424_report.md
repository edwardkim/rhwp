# PR #424 처리 보고서 — Task #412 다단 우측 단 단행 문단 줄간격 누락 정정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#424](https://github.com/edwardkim/rhwp/pull/424) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#412](https://github.com/edwardkim/rhwp/issues/412) (closes) |
| 처리 결정 | **cherry-pick 머지** (Task #412 본질 4 commits 분리) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 (옵션 A 결정)

PR 의 36 commits 중 본질은 Task #412 의 4 commits — 나머지는 Task #398/#402/#404/#409 (다른 PR 로 흡수 완료) + 샘플/계획서/Stage 보고서/merge.

→ PR #406/#408/#410 와 같은 분리 cherry-pick 패턴 채택.

### Stage 1: cherry-pick

`local/pr424` 브랜치 (`local/devel` 분기) 에서 4 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `3798bdc` (← `144ff53`) | @planet6897 | Stage 1: vpos 보정 진단 + 영향 범위 분석 |
| `01b3ac9` (← `cdeea6b`) | @planet6897 | Stage 2: vpos 보정 anchor (col_anchor_y) 도입 + curr_first_vpos 사용 |
| `1827f28` (← `3d395e2`) | @planet6897 | Stage 3: 다중 샘플 회귀 검증 |
| `d8b6479` (← `a6c2457`) | @planet6897 | Stage 4: 최종 결과 보고서 + orders 갱신 |

cherry-pick 결과:
- Stage 1, 2, 3 자동 적용
- Stage 4 의 `mydocs/orders/20260428.md` add/add 충돌 → HEAD 유지 후 `--continue`

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1062 passed** (회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 46s, 4,184,640 bytes (+144 from PR #419) |

### Stage 3: 광범위 byte 단위 비교

10 샘플 / 309 페이지 SVG 비교:

| 결과 | 카운트 |
|------|------|
| byte 동일 | 238 / 309 |
| 차이 발생 (vpos 보정 영향) | **71 / 309** |

차이 분포 (의도된 정정):
- exam_eng (8/8), exam_kor (16), exam_math (15) — 다단 레이아웃 정정 대상
- kps-ai (17), aift (4), synam-001 (3), 2025년 기부 (1)

본 PR 은 layout.rs 의 vpos 보정 공식 자체를 변경하므로 다단 페이지 광범위 변화는 의도된 정황. 회귀 vs 의도된 변화 구분은 작업지시자 시각 판정으로 결정.

### Stage 4: 시각 판정 (작업지시자 직접)

**비교 자료**: 한컴 hwp 정답지 (한컴 2010 / 2022 직접 열람) vs rhwp 출력

| 경로 | 시각 판정 결과 |
|------|--------------|
| Canvas (rhwp-studio 웹 에디터) | ✅ 통과 |
| SVG 내보내기 | ✅ 통과 |

작성자 명시 핵심 검증:
- p1 우측 단 item 7 ①~⑤ 22.55px 균일 (Pre-fix 15.33 → Post-fix 22.55) ✓
- p1 좌측 단 item 1 catch-up 회귀 정상화 (28.56 → 21.89) ✓
- p2 item 20 catch-up 정상화 ✓

다른 71 페이지 변화도 한컴 정답지 대비 회귀 없음 확인 완료.

## 변경 요약

### Task #412 핵심 — `layout.rs` vpos 보정 공식 정정 (4 단계)

| 단계 | 내용 |
|------|------|
| 1 | **`col_anchor_y` 도입** — `build_single_column` 진입 시 body_wide_reserved 푸시 직후 anchor 보존 |
| 2 | **`curr_first_vpos` 우선 사용** — paragraph spacing_after 정확도 (HWP 인코딩 정합) |
| 3 | **page_path / lazy_path 분리** — anchor / col_area.y 기준 구분 |
| 4 | **환경변수 가드 진단** (`RHWP_VPOS_DEBUG=1`) |

### 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | vpos 보정 공식 정정 (anchor 도입 + curr_first_vpos + path 분리) |

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1062 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ SVG + Canvas 양 경로 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ col_anchor_y (body_wide_reserved 푸시 명시 anchor) + lazy/page path 분리 |
| v0.7.6 페이지 레이아웃 회귀의 origin | ✅ 작업지시자 직접 시각 검증 게이트 통과 (광범위 71 페이지 변화 회귀 없음 확인) |
| output 폴더 가이드라인 | ✅ `output/svg/pr424-{visual,devel-baseline,regression-test,regression-baseline}/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr424` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr424` → `local/devel` → `devel` 머지 + push
3. PR #424 close + 작성자 댓글 (이슈 #412 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_424_review.md`
- PR: [#424](https://github.com/edwardkim/rhwp/pull/424)
- 이슈: [#412](https://github.com/edwardkim/rhwp/issues/412)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#410](https://github.com/edwardkim/rhwp/pull/410), [#415](https://github.com/edwardkim/rhwp/pull/415)
