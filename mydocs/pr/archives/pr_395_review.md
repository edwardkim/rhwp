# PR #395 검토 — 그림 밝기/대비 효과 SVG 반영 (#150)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#395](https://github.com/edwardkim/rhwp/pull/395) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) |
| base / head | `devel` ← `oksure:contrib/image-brightness-contrast-v2` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | **BEHIND** (Task #416/#418 머지 후 — 본 검토에서 자동 머지 검증 통과) |
| 변경 통계 | +90 / -0, 8 files |
| CI | 모두 SUCCESS (Build & Test, CodeQL javascript / python / rust) |
| 이슈 | [#150](https://github.com/edwardkim/rhwp/issues/150) |
| 정황 | PR #387 의 devel 기반 재작업 (메인테이너 안내 후 작성자 정정 재제출) |

## 작성자 정황

@oksure (Hyunwoo Park) — 신뢰 컨트리뷰터:
- 머지 이력: PR #334 (replaceOne API), #335 (이미지 base64)
- 이전 PR #387 의 base=main 정정 후 본 PR 로 재제출 (CONTRIBUTING.md 안내 절차 준수)

## 변경 내용 정리

이슈 #150 의 **밝기 (brightness) / 대비 (contrast)** 효과 SVG 반영. 워터마크 효과는 본 PR 범위 외.

### 코드 변경 (8 files, +90 / -0)

| 파일 | 변경 |
|------|------|
| `src/renderer/render_tree.rs` | `ImageNode` 에 `brightness: i8`, `contrast: i8` 필드 추가 (기본값 0) |
| `src/renderer/svg.rs` | `ensure_brightness_contrast_filter()` 추가 + `render_image_node` 에서 `<g filter>` 래핑 |
| `src/renderer/svg/tests.rs` | 단위 테스트 3개 (zero / nonzero / dedup) |
| `src/renderer/layout.rs` | layout_shape_item 의 빈 문단 + TAC Picture 분기에서 brightness/contrast 전달 |
| `src/renderer/layout/paragraph_layout.rs` | 인라인 Picture (TAC) 경로 3곳 (line 1702, 1954, 2039) 에서 전달 |
| `src/renderer/layout/picture_footnote.rs` | 머리말/꼬리말 Picture 2곳에서 전달 |
| `src/renderer/layout/shape_layout.rs` | 그룹 내 Picture 에서 전달 |
| `src/renderer/layout/table_cell_content.rs` | 표 셀 내 Picture 에서 전달 |

### SVG 필터 수식

```rust
// 밝기: intercept 오프셋 (slope=1, intercept=brightness/100)
// 대비: slope 조정 (slope=(100+contrast)/100, intercept=0.5-0.5*slope)
// 합성: slope=contrast_slope, intercept=contrast_intercept + brightness_offset
let b = brightness as f64 / 100.0;
let slope = (100.0 + contrast as f64) / 100.0;
let intercept = (0.5 - 0.5 * slope) + b;
```

`<feComponentTransfer>` 의 `feFuncR/G/B` 에 `type="linear" slope intercept` 적용. 일반적인 밝기/대비 합성 공식 (slope = contrast 스케일, intercept = brightness offset + 대비 중심점 보정) — **공식 자체 합리**.

### 단위 테스트

```rust
test_brightness_contrast_filter_zero_returns_none      // 0,0 → None (필터 불필요)
test_brightness_contrast_filter_nonzero_adds_defs      // 30,-20 → defs 추가
test_brightness_contrast_filter_dedup                  // 같은 값 두 번 → defs 1개
```

## 검증

### 본 검토에서 dry-run merge 검증

devel (Task #418 머지 후) 위에 자동 merge 시도:

```bash
git merge --no-commit --no-ff pr395
# → Auto-merging src/renderer/layout.rs
# → Auto-merging src/renderer/layout/paragraph_layout.rs
# → Automatic merge went well
```

**자동 충돌 해결**:
- `paragraph_layout.rs:2042` (Task #418 의 `set_inline_shape_position`) + `2039` (PR #395 의 brightness/contrast) — 인접 라인이지만 다른 영역으로 깔끔히 머지
- `layout.rs:2554` (Task #418 의 가드) + `2572` (PR #395 의 brightness/contrast) — 충돌 없음

### 머지 후 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1026 passed** (1023 + 3 신규 svg 테스트) |
| `cargo test --test issue_418` | ✅ 1/1 passed (Task #418 이중 출력 방지 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |

→ 머지 후 본 task 의 다른 회귀 정정도 그대로 작동.

## 평가

### 강점

1. **이슈 명확** — #150 의 brightness/contrast 정확히 반영 (워터마크는 별도 후속)
2. **변경 범위 작음** — 8 files / +90 / -0 (deletion 0)
3. **일관된 패턴** — 모든 Picture ImageNode 생성 지점 (7곳) 에 동일 패턴으로 추가
4. **SVG 필터 공식 합리** — slope/intercept 합성 표준
5. **단위 테스트 양호** — zero / nonzero / dedup 3가지 케이스
6. **CI 모두 통과**
7. **devel 기반 재제출** — base 정정 안내 따름 (PR #387 → #395)
8. **CONTRIBUTING.md 절차 준수** — 사전 의향 확인 ([#383](https://github.com/edwardkim/rhwp/issues/383) 패턴은 아니지만) → PR 제출

### 약점 / 점검 필요

#### 1. paragraph_layout.rs:1955-1957 들여쓰기 어긋남

PR diff 확인 시:

```rust
                                        para_index: Some(para_index),
                                        control_index: Some(tac_ci),
                                        effect: pic.image_attr.effect,
+                                            brightness: pic.image_attr.brightness,
+                                            contrast: pic.image_attr.contrast,
                                        ..ImageNode::new(bin_data_id, image_data)
```

신규 라인의 들여쓰기가 **40 칸** (들여쓰기 4칸 더 깊음) — 주변 라인은 36 칸. **타이포 / 들여쓰기 결함**. clippy 가 통과한 것은 들여쓰기가 컴파일 / clippy 검증 대상 아니기 때문. 머지 전 정정 권장.

다른 6 곳의 신규 라인은 정상 들여쓰기 — **본 1 곳만 수정 필요**.

#### 2. 시각 검증 샘플

본 PR 이 정정한 효과는 **밝기/대비** — exam_eng / k-water-rfp 등 본 저장소 표준 샘플에 포함된 이미지들이 brightness/contrast 0이 아닌 케이스가 있는지 점검 필요. 작성자 환경에서는 검증됐을 것이지만, 작업지시자 환경에서 시각 판정 권장.

#### 3. 워터마크 효과 미구현

이슈 #150 의 일부 (밝기/대비) 만 처리. 워터마크는 별도 PR 후보. 이는 PR 본문에 명시 — 명확한 범위.

## 메인테이너 작업과의 관계

### Task #416 (`find_bin_data` 가드 결함)

- 영향 파일: `src/renderer/layout/utils.rs` (변경 없음 — PR #395 와 무관)

### Task #418 (빈 문단 + TAC Picture 이중 emit)

- 영향 파일: `src/renderer/layout/paragraph_layout.rs` (line 2042 다음), `src/renderer/layout.rs` (line 2554)
- PR #395 와 같은 파일 + 같은 영역 — **dry-run merge 자동 해결 + 머지 후 cargo test 1026 통과** 로 검증 완료
- Task #418 의 회귀 테스트 (`tests/issue_418.rs`) 통과 → 본 PR 머지로 이중 출력 회귀 재발 안 함

## 처리 방향 후보

### 옵션 A: 들여쓰기 정정 후 cherry-pick 머지

이유:
1. dry-run merge 자동 해결 + 머지 후 검증 통과
2. 들여쓰기 1 곳만 정정 권장 — 메인테이너가 직접 정정 또는 작성자에게 요청
3. 작성자 attribution 보존 (cherry-pick)

### 옵션 B: 들여쓰기 정정 요청 후 작성자 재푸시

작성자에게 들여쓰기 1 곳 (paragraph_layout.rs:1955-1957) 정정 요청 → 새 commit push → cherry-pick.

### 옵션 C: 그대로 cherry-pick + 별도 commit 으로 들여쓰기 정정

cherry-pick 후 들여쓰기만 정정하는 메인테이너 후속 commit 추가.

## 권장

**옵션 C (cherry-pick + 후속 들여쓰기 정정)** 권장:

이유:
1. 작성자 부담 최소화 — 들여쓰기 1 곳은 사소하고 메인테이너가 빠르게 정정 가능
2. 작성자 attribution 보존
3. PR 본 자체 통과 가능 — 들여쓰기는 기능에 영향 없음
4. 후속 commit 으로 lint 정리하면 git log 도 명확

또는 **옵션 A (단일 cherry-pick + 들여쓰기 자체 정정)** 도 가능 — 후속 commit 분리 vs 단일 commit 정리 선호도에 따름.

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 ✅ (PR #334, #335 머지 이력)
- [x] 코드 품질 — SVG 필터 공식 정확 ✅
- [x] CI 통과 ✅
- [x] 이슈 연결 명확 ✅ (closes #150 본문 명시)
- [x] devel 기반 재제출 ✅ (PR #387 정정 후속)
- [x] dry-run merge 자동 해결 ✅
- [x] 머지 후 cargo test --lib 1026 passed ✅
- [x] Task #418 회귀 테스트 통과 ✅
- [x] clippy warning 0 ✅
- [ ] 들여쓰기 어긋남 정정 (`paragraph_layout.rs:1955-1957`) — **머지 전 또는 후속 commit 으로 정정 필요**
- [ ] 작업지시자 시각 판정 (brightness/contrast 효과 적용된 hwp 샘플) — 권장

## 다음 단계 — 작업지시자 결정

A / B / C 중 결정 부탁드립니다.

권장: **C (cherry-pick + 후속 들여쓰기 정정 commit)**

## 참고

- PR: [#395](https://github.com/edwardkim/rhwp/pull/395)
- 이슈: [#150](https://github.com/edwardkim/rhwp/issues/150) (OPEN — brightness/contrast + 워터마크. 본 PR 은 brightness/contrast 만)
- 이전 PR: [#387](https://github.com/edwardkim/rhwp/pull/387) (CLOSED — base=main 정정 안내)
- 본 PR 의 작성자 다른 PR: [#396](https://github.com/edwardkim/rhwp/pull/396) (수식 렌더링 — TAC 높이 + 한글 이탤릭, 별도 검토)
