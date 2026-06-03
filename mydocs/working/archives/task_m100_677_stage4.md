# Stage 4 — 회귀 가드 영구 보존 + 광범위 sweep 보고서 (Task #677)

## 회귀 가드 영구 보존

### svg_snapshot 회귀 차단 가드 추가

**대상**: `tests/svg_snapshot.rs::issue_677_bokhakwonseo_page1`

```rust
/// Issue #677: 복학원서.hwp 1페이지 — 다음 결함 영역의 회귀 차단
///   1. PartialParagraph y 누적 결함 (인라인 TAC 표 라인)
///      `layout.rs::PageItem::PartialParagraph` 가 y_offset 을
///      LineSeg.vpos 정합 위치로 리셋하는 동작을 잠가둔다.
///   2. U+F081C HWP PUA 채움 문자 폭 (0)
///      `text_measurement.rs::char_width` 의 5 사이트 모두 채움 문자
///      폭을 0 으로 처리하는 동작을 잠가둔다.
///   3. 한컴 워터마크 모드 표준 프리셋 (brightness=+70, contrast=-50)
///      `svg.rs::render_image` 의 워터마크 게이트 동작을 잠가둔다.
#[test]
fn issue_677_bokhakwonseo_page1() {
    check_snapshot("samples/복학원서.hwp", 0, "issue-677/bokhakwonseo-page1");
}
```

**골든 영구 보존**: `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` (414,812 bytes)

**골든 영역 정합 검증** (Stage 2/3 정정 결과):
- `cell-clip-174 x=63.69` (3×3 접수증 표 첫 셀, body left margin) — Stage 2 U+F081C 정정 정합
- `cell-clip-177 x=245.03` (둘째 셀, 첫 셀 + 181.33 셀 폭) — 정합
- `cell-clip-182 x=524.81` (셋째 셀) — 정합
- 워터마크 이미지 effect=GrayScale 적용 + brightness/contrast 변환 — Stage 3 정정 정합

## 광범위 sweep 회귀 검증

### 결정적 검증 (cargo)

```
cargo test --release --lib                       1155 passed (회귀 0)
cargo test --release --test svg_snapshot         8 passed (1 신규 + 7 기존, 회귀 0)
cargo test --release --test issue_546            1 passed
cargo test --release --test issue_554            12 passed
cargo test --release --test issue_598_footnote_marker_nav  4 passed
cargo test --release --test issue_501            1 passed
cargo clippy --release --lib                     0 warnings
cargo build --release                            success
cargo check --target wasm32-unknown-unknown --release --lib  WASM lib 빌드 success
```

### 페이지네이션 sweep (162+ HWP/HWPX fixture)

**전체 fixture 페이지 합계**: **1,964 페이지** (모든 samples/ 영역, .hwp + .hwpx)

**핵심 fixture 페이지 수 확인** (역사적 documented 값과 비교):

| Fixture | 페이지 수 | 역사적 영역 정합 |
|---------|----------|-------------|
| `aift.hwp` (HWP5) | 77 | ✅ PR #601 documented 77 |
| `aift.hwpx` (HWPX) | 74 | ✅ PR #601 documented 74 (한컴 정합) |
| `synam-001.hwp` | 35 | ✅ PR #601 documented 35 |
| `hwp3-sample5.hwp` (HWP3) | 64 | ✅ PR #609 documented 64 |
| `exam_kor.hwp` | 20 | ✅ 안정 |
| `exam_eng.hwp` | 8 | ✅ PR #592 documented 8 |
| `exam_math.hwp` | 20 | ✅ 안정 |
| `exam_science.hwp` | 4 | ✅ 안정 |
| `footnote-01.hwp` | 6 | ✅ PR #642 권위 샘플 |
| `복학원서.hwp` (본 fixture) | 1 | ✅ 본 task baseline |

**페이지 수 회귀: 0건**

### 영향 범위 좁힘 (이중 검증)

**U+F081C 보유 fixture**: 1개 (복학원서.hwp 만)
**워터마크 (effect != RealPic + brightness/contrast 비-zero) 보유 fixture**: 1개 (복학원서.hwp 만)

→ 본 정정 (Stage 2 U+F081C + Stage 3 워터마크) 의 영향을 받는 다른 fixture **0개**.

### 기존 svg_snapshot 골든 byte-identical 정합

- `form-002/page-0.svg` — byte-identical (변경 없음)
- `issue-147/aift-page3.svg` — byte-identical
- `issue-157/page-1.svg` — byte-identical
- `issue-267/ktx-toc-page.svg` — byte-identical
- `issue-617/exam-kor-page5.svg` — byte-identical
- `table-text/page-0.svg` — byte-identical
- (form-002 page-0 의 다른 변형 포함)

기존 7개 골든이 byte-level 정합 통과 → **다른 fixture 의 SVG 출력 회귀 0**.

### TAC 표 인라인 영역 fixture 확인

`paragraph_layout.rs::PageItem::PartialParagraph` y 리셋 분기는 다음 모든 조건이 충족돼야 발동:
1. `start_line > 0`
2. `para.controls 에 TAC 표 존재` (treat_as_char=true)
3. `para_start_y` 등록 (Table item 선행)

**조건 발동 fixture 영역 sweep**:
```bash
$ for f in samples/*.hwp; do
    if rhwp dump "$f" | grep -q 'tac=true.*pi=.*Table'; then echo "$f"; fi
done
```

→ TAC 표 보유 fixture 가 여럿 존재하지만 (`exam_kor.hwp` / `aift.hwp` / `form-002.hwp` 등) 본 정정의 PP y 리셋 분기는 **block-TAC 표 (is_tac_table_inline=false) + start_line>0 PP** 영역에서만 발동. 회귀 가드:

- exam_kor.hwp `issue_617` 골든 byte-identical (다중 줄 TAC 표 padding 영역 회귀 0)
- aift.hwp `issue_147` 골든 byte-identical (인라인 표 영역 회귀 0)
- form-002.hwp `form-002/page-0` 골든 byte-identical (양식 표 영역 회귀 0)

## Stage 4 변경 LOC

| 파일 | 변경 | 영역 |
|------|------|------|
| `tests/svg_snapshot.rs` | +14 / 0 | `issue_677_bokhakwonseo_page1` 회귀 가드 |
| `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` | +414812 bytes | 골든 영구 보존 |
| **합계** | **+14 LOC + 1 골든 (414KB)** | 회귀 차단 영구 영역 |

## Stage 1~4 누적 변경 LOC

| 파일 | 변경 | 영역 |
|------|------|------|
| `src/renderer/layout.rs` | +30 / -2 | PP y 리셋 + max 누적 |
| `src/renderer/layout/text_measurement.rs` | +20 / 0 | U+F081C 폭 0 (5 사이트) |
| `src/renderer/svg.rs` | +9 / -1 | 워터마크 표준 프리셋 |
| `src/renderer/web_canvas.rs` | +9 / -1 | 워터마크 표준 프리셋 (WASM) |
| `tests/svg_snapshot.rs` | +14 / 0 | issue_677 가드 |
| `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` | +414812 bytes | 골든 |
| `mydocs/plans/task_m100_677.md` | +132 LOC | 수행계획서 |
| `mydocs/plans/task_m100_677_impl.md` | +219 LOC | 구현계획서 |
| `mydocs/working/task_m100_677_stage1.md` | +151 LOC | Stage 1 진단 |
| `mydocs/working/task_m100_677_stage2.md` | +119 LOC | Stage 2 정정 |
| `mydocs/working/task_m100_677_stage3.md` | +120 LOC | Stage 3 정정 |
| `mydocs/working/task_m100_677_stage4.md` | (본 보고서) | Stage 4 회귀 가드 |
| **합계** | **+82 / -4 src LOC + 1 가드 + 1 골든 + 5 거버넌스 LOC** | 본 task 전체 |

## 잔존 영역 (별도 task 후보)

- **body-clip width 1619.92** — Stage 1 식별, 시각 영향 없음. 별도 후속 task 후보 (코드 위생).
- **워터마크 표준 프리셋 외 사용자 customization 영역** — 본 정정은 모든 워터마크에 표준 프리셋 강제. 사용자가 custom 강도를 의도한 경우 무시. 다른 워터마크 fixture 추가 시 customization 보존이 필요한지 별도 검토.

## 승인 요청

본 Stage 4 결과 승인 후 **Stage 5 (시각 판정 + 잔존 점검)** 진행하겠습니다.

대상 영역 (Stage 5):
- 메인테이너 시각 판정 영역 (PNG 생성 + PDF 비교)
- rhwp-studio web 환경 시각 판정 (`feedback_visual_judgment_authority`)
- Docker WASM 빌드 + byte 변화 정량 측정
- rhwp-studio npm run build TypeScript 타입 체크 + dist 빌드
