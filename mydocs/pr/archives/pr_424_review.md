# PR #424 검토 — Task #412 다단 우측 단 단행 문단 줄간격 누락 (vpos 보정 anchor)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#424](https://github.com/edwardkim/rhwp/pull/424) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터 |
| 이슈 | [#412](https://github.com/edwardkim/rhwp/issues/412) (closes) |
| base / head | `devel` ← `planet6897:task412` |
| 변경 규모 | +4,726 / -136, 44 files (36 commits) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |

## PR 의 36 commits 누적 정황

| 영역 | 처리 |
|------|------|
| Task #398 (3) | PR #401 v2 흡수 완료 |
| Task #402 (3) | PR #406 흡수 완료 |
| Task #404 (4) | PR #408 흡수 완료 |
| Task #409 v1+v2+v3 (다수) | PR #410 흡수 완료 |
| 샘플 PDF + 계획서/Stage/merge | skip |
| **Task #412 (4)** | 분리 cherry-pick 대상 |

## 본질 — `layout.rs` vpos 보정 공식 정정

### 원인

`layout.rs:1392-1430` 의 vpos 보정 공식:
```rust
let end_y = col_area.y + hwpunit_to_px(vpos_end - base, self.dpi);
```

`col_area.y` 가 vpos=0 좌표라고 가정하지만, `body_wide_reserved` (페이지 너비 글상자/표) 푸시가 있는 다단 레이아웃에서는 첫 항목이 `col_area.y` 가 아닌 푸시 적용 후 위치에서 시작.

다단 우측 단처럼 `vpos_page_base` 가 큰 경우, 보정값 `col_area.y + (vpos_end - base) * scale` 이 sequential `y_offset` 보다 항상 작아져 조건 검사를 통과하지 못하고 스킵 → 단행 문단의 trailing line_spacing 이 누락된 채로 렌더.

### 정정 (4 단계)

1. **`col_anchor_y` 도입**: `build_single_column` 진입 시 body_wide_reserved 푸시 직후의 `y_offset` 을 `col_anchor_y` 로 보존. 첫 PageItem 의 실제 렌더 y = `vpos_page_base` 좌표에 대응
2. **`curr_first_vpos` 우선 사용**: `prev_seg.vpos + lh + ls` 대신 현재 paragraph 의 first seg vpos 우선 (HWP 가 paragraph spacing_after 를 다음 paragraph first vpos 에 인코딩하므로 더 정확). vpos reset(0) 또는 prev 보다 작아진 경우 prev 기반 fallback
3. **page_path / lazy_path 분리**:
   - page_path: `col_anchor_y + (vpos_end - base) * scale`
   - lazy_path: `col_area.y + (vpos_end - base) * scale` (lazy_base 가 sequential y_offset 으로부터 역산되어 col_area.y 기준이 일관)
4. **환경변수 가드 진단** (`RHWP_VPOS_DEBUG=1`)

### 작성자 명시 검증 결과

| 케이스 | Pre-fix | Post-fix | 기대 |
|--------|---------|----------|------|
| **p1 우측 단 item 7 ①→②** | 15.34 ❌ | 22.54 ✓ | 22.55 |
| **p1 우측 단 item 7 ②~⑤** | 15.33 ❌ (촘촘) | 22.55 균일 ✓ | 22.55 |
| **p1 좌측 단 item 1 ①→②** | 28.56 (catch-up) | 21.89 ✓ | 균일 |
| **p2 item 20** | catch-up 회귀 | 정상 ✓ | 균일 |

## 처리 방향

**옵션 A — Task #412 본질 4 commits 분리 cherry-pick** (PR #406/#408/#410/#419 와 같은 패턴).

이유:
1. Task #398/#402/#404/#409 commits 는 이미 devel 흡수 완료
2. 본질 변경은 Task #412 의 4 commits

## dry-run cherry-pick 결과

`local/pr424` 브랜치 (`local/devel` 분기) 에서 4 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `3798bdc` (← `144ff53`) | @planet6897 | Stage 1: vpos 보정 진단 + 영향 범위 분석 |
| `01b3ac9` (← `cdeea6b`) | @planet6897 | Stage 2: vpos 보정 anchor (col_anchor_y) 도입 + curr_first_vpos 사용 |
| `1827f28` (← `3d395e2`) | @planet6897 | Stage 3: 다중 샘플 회귀 검증 |
| `d8b6479` (← `a6c2457`) | @planet6897 | Stage 4: 최종 결과 보고서 + orders 갱신 |

cherry-pick 결과:
- Stage 1, 2, 3 자동 적용
- Stage 4 의 `mydocs/orders/20260428.md` add/add 충돌 (다른 PR 패턴 동일) → HEAD 유지 후 `--continue`

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1062 passed** (PR #419 동일, 회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 46s, 4,184,640 bytes (+144 from PR #419) |

## 광범위 byte 단위 비교

10 샘플 / 309 페이지 SVG 비교 (devel baseline ↔ PR #424 적용):

| 결과 | 카운트 |
|------|------|
| byte 동일 (영향 없음) | **238 / 309** |
| 차이 발생 (vpos 보정 영향) | **71 / 309** |

차이 발생 분포 (의도된 정정 영향):
- **exam_*** (eng + kor + math): 46 페이지 — 다단 레이아웃 본 PR 정정 대상
- **kps-ai**: 17 페이지 — 다단 레이아웃
- **aift**: 4 페이지
- **synam-001**: 3 페이지
- **2025년 기부·답례품**: 1 페이지

본 PR 은 layout.rs 의 vpos 보정 공식 자체를 변경하므로 다단 레이아웃 페이지에 의도된 변화가 광범위하게 발생하는 정황. 회귀 vs 의도된 변화 구분은 작업지시자 시각 판정으로 결정.

## 시각 판정

**한컴 정답지 (한컴 2010 / 2022) vs rhwp 출력 비교** — 작업지시자 직접 판정 (SVG + Canvas 양 경로).

| 경로 | 시각 판정 결과 |
|------|--------------|
| Canvas (rhwp-studio 웹 에디터) | ✅ **통과** |
| SVG 내보내기 | ✅ **통과** |

산출물:
- SVG (devel): `output/svg/pr424-devel-baseline/exam_eng_{001,002}.svg` + `output/svg/pr424-regression-baseline/` (309 SVG)
- SVG (PR #424): `output/svg/pr424-visual/exam_eng_{001,002}.svg` + `output/svg/pr424-regression-test/` (309 SVG)
- Canvas (rhwp-studio): WASM `pkg/rhwp_bg.wasm` (4,184,640 bytes)

**메모리 `feedback_v076_regression_origin.md` 원칙 부합** — 다단 레이아웃 광범위 변화에도 작업지시자가 직접 한컴 정답지와 비교 후 회귀 없음 확인.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1062 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ SVG + Canvas 양 경로 작업지시자 직접 판정 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ `col_anchor_y` (body_wide_reserved 푸시 anchor) 명시 + `curr_first_vpos` 우선 + lazy/page path 분리 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr424` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close 예정 |
| PR 댓글 톤 | ✅ |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr424` → `local/devel` → `devel` 머지 + push
3. PR #424 close + 작성자 댓글 (이슈 #412 자동 close)

## 참고

- PR: [#424](https://github.com/edwardkim/rhwp/pull/424)
- 이슈: [#412](https://github.com/edwardkim/rhwp/issues/412)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#410](https://github.com/edwardkim/rhwp/pull/410), [#415](https://github.com/edwardkim/rhwp/pull/415)
