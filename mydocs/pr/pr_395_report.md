# PR #395 처리 보고서 — 그림 밝기/대비 SVG 반영 (#150)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#395](https://github.com/edwardkim/rhwp/pull/395) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) |
| 이슈 | [#150](https://github.com/edwardkim/rhwp/issues/150) |
| 처리 결정 | **옵션 C (cherry-pick + 메인테이너 후속 정정)** |
| 처리 일자 | 2026-04-28 |

## 처리 절차

### Stage 1: cherry-pick

`local/pr395` 브랜치 (`local/devel` 분기) 에서 PR head 까지 2 commit cherry-pick — 작성자 attribution 보존:

| commit | 작성자 | 내용 |
|--------|--------|------|
| `cb248cd` | @oksure | 본 기능 (brightness/contrast SVG 반영) |
| `e637687` | @oksure | 들여쓰기 정렬 + 단위 테스트 3개 |

cherry-pick 결과: 충돌 없이 자동 머지 (Task #418 와 인접한 영역이지만 분리 가능).

### Stage 2: 메인테이너 후속 정정 commit (`c9fe462`)

작업지시자 검토 의견에 따른 3가지 후속 정정:

#### 1. 누락된 들여쓰기 정정 (`paragraph_layout.rs:1957-1958`)

작성자의 정렬 commit (`e637687`) 에서 한 곳 누락. 40 칸 → 36 칸으로 주변 라인과 일치.

```diff
                                         effect: pic.image_attr.effect,
-                                            brightness: pic.image_attr.brightness,
-                                            contrast: pic.image_attr.contrast,
+                                        brightness: pic.image_attr.brightness,
+                                        contrast: pic.image_attr.contrast,
                                         ..ImageNode::new(bin_data_id, image_data)
```

#### 2. `i8` clamp 추가 (`svg.rs::ensure_brightness_contrast_filter`)

HWP 스펙은 brightness/contrast 를 -100..=100 으로 정의하지만 코드 레벨 가드 없음. 손상된 입력 (i8 max/min 등) 에 대비:

```rust
let brightness = brightness.clamp(-100, 100);
let contrast = contrast.clamp(-100, 100);
if brightness == 0 && contrast == 0 {
    return None;
}
```

#### 3. 수치 테스트 3개 추가 (`svg/tests.rs`)

slope/intercept 합성이 의도한 값과 일치하는지 검증:

```rust
test_brightness_contrast_filter_pure_brightness    // b=50, c=0 → slope=1.0, intercept=0.5
test_brightness_contrast_filter_pure_contrast      // b=0, c=50 → slope=1.5, intercept=-0.25
test_brightness_contrast_filter_clamp_out_of_range // i8 max/min → -100..=100 clamp 검증
```

### Stage 3: 자동 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1029 passed** (1023 → +3 작성자 + +3 메인테이너 = +6) |
| `cargo test --test issue_418` | ✅ 1/1 passed (Task #418 회귀 방지 보존) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 21s, 4,100,772 bytes (Task #418 → -247, wasm-opt 최적화) |

## 변경 요약

| 파일 | 작성자 변경 | 메인테이너 변경 |
|------|----------|---------------|
| `src/renderer/render_tree.rs` | `ImageNode` 에 brightness/contrast 필드 추가 | — |
| `src/renderer/svg.rs` | `ensure_brightness_contrast_filter` 추가 + 필터 래핑 | `i8.clamp(-100, 100)` 추가 |
| `src/renderer/svg/tests.rs` | 단위 테스트 3개 (zero/nonzero/dedup) | 수치 테스트 3개 (pure brightness/contrast/clamp) |
| `src/renderer/layout.rs` | brightness/contrast 전달 | — |
| `src/renderer/layout/paragraph_layout.rs` | 인라인 Picture 3곳에서 전달 | 들여쓰기 1곳 정정 (line 1957-1958) |
| `src/renderer/layout/picture_footnote.rs` | 머리말/꼬리말 2곳에서 전달 | — |
| `src/renderer/layout/shape_layout.rs` | 그룹 내 Picture 에서 전달 | — |
| `src/renderer/layout/table_cell_content.rs` | 표 셀 내 Picture 에서 전달 | — |

## 작성자 댓글 + 후속 개선 제안

PR 댓글 ([4334915978](https://github.com/edwardkim/rhwp/pull/395#issuecomment-4334915978)) 으로 다음 안내:

1. 처리 결과 (옵션 C, cherry-pick + 후속 정정) 보고
2. 메인테이너 후속 정정 3가지 설명
3. **HashSet dedup 개선 제안** (별도 PR 후보):
   - `SvgRenderer.defs` 의 dedup 가 현재 `Vec.iter().any()` — O(n)
   - ID 가 이미 unique 키이므로 `HashSet<String>` 으로 ID 만 추적하면 O(1)
   - `ensure_image_effect_filter` (기존) + `ensure_brightness_contrast_filter` (본 PR) 양쪽 통일 가능
   - 다른 `defs.push` 지점 (마커/그라디언트/패턴 등 7곳) 도 함께 정리 권장

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| PR 댓글 톤 — 과도한 표현 자제 | ✅ "정말 감사합니다" 등 자제, 사실 중심 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr395` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 PR close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 커밋
2. `local/pr395` → `local/devel` → `devel` 머지 + push
3. PR #395 close (작성자 댓글 후속)

## 참고

- 검토 문서: `mydocs/pr/pr_395_review.md`
- 작성자 다른 OPEN PR: [#396](https://github.com/edwardkim/rhwp/pull/396) (수식 렌더링 — 별도 검토)
- HashSet dedup 후보 task: 향후 `SvgRenderer` refactoring (별도 PR)
- 이슈 #150: 본 PR 은 brightness/contrast 만 — 워터마크 효과는 후속
