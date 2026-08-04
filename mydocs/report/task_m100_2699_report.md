# task_m100_2699 처리결과 보고서 — Bottom 캡션 예약의 음수 line_spacing 미클램프

- **이슈**: [#2699](https://github.com/edwardkim/rhwp/issues/2699)
- **브랜치**: `task/m100-2699-caption-reserve-negative-linespacing` (base `devel` @ `2cd4d78b`)
- **범위**: `src/renderer/pagination/engine.rs`, `src/renderer/pagination/tests.rs`
- **분류**: 결함 수정 (페이지네이션 과소 예약 → 본문 하단 넘침)

## 1. 문제

`split_table_rows()` 가 Bottom 캡션 예약분을 계산할 때 호스트 문단의 `line_spacing` 을
**클램프 없이** 더했다 (`engine.rs:2449-2453`).

```rust
let host_line_spacing_for_caption = para
    .line_segs
    .first()
    .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
    .unwrap_or(0.0);
```

`line_spacing` 은 `i32`(`src/model/paragraph.rs:152`)이고, `hwpunit_to_px`
(`src/renderer/mod.rs:918`)는 부호를 그대로 보존한다. 이 저장소에서 **음수 `line_spacing`
은 고정값 줄간격 TAC 표 마커로 실제 사용되는 값**(Task #9)이므로 음수가 정상 경로로 들어온다.

### 실패 사슬

1. `:2470-2474` — `caption_overhead = caption_base_overhead + host_line_spacing_for_caption`.
   음수 ls 가 예약분을 **차감**한다.
2. `:2683` — `total_with_caption = partial_height + caption_overhead` 가 `|ls|` px 만큼 작아진다.
3. `:2689` — 참이어야 할 `total_with_caption > avail` 이 거짓이 된다.
4. `:2690` — `end_row = end_row.saturating_sub(1)` 이 **발동하지 않는다.**

결과: 마지막 표 행 + Bottom 캡션이 N 페이지에 남아 본문 하단선 아래 / 꼬리말 영역을 침범해
그려진다. 과소 예약이므로 **넘침 방향**의 오류다.

## 2. 분석

### 음수 ls 가 실제 값이라는 근거

- `engine.rs:977-988` — **결함 지점과 동일한 `para.line_segs.first()`** 를 읽어
  `seg.line_spacing < 0` 이면 고정값 줄간격 TAC 표로 분기한다. 같은 seg 를 한쪽은 마커로
  해석하고 다른 쪽은 예약량에 그대로 더하는 비대칭이었다.
- `layout.rs:5272-5286` — 렌더러도 `line_segs.first()` 의 음수 ls 로 같은 판별을 한다.
- `layout.rs:7154-7160` — 렌더러는 `ls > 0` 일 때만 `y_offset` 을 더하고, 음수는 별도 분기로
  처리한다. 즉 실제 조판에서 음수 ls 는 캡션 앞 간격을 **0** 으로 만들지 음수로 만들지 않는다.
  페이지네이션이 음수를 그대로 빼던 것은 렌더러 동작과 불일치였다.

### `engine.rs` 의 `line_spacing` 참조 전수 조사

| 줄 | 클램프 | 용도 분류 | 판정 |
|---|---|---|---|
| `532` | `.filter(> 0)` | 예약/트레일링 | OK |
| `557` | 없음 | advance 합 (`text_height + ls`) | OK |
| `589` | `.filter(> 0)` | 예약/트레일링 | OK |
| `642` | `.filter(> 0)` | 트레일링 차감 | OK |
| `712` | 없음 | advance 합 (`vpos + lh + ls`) | OK |
| `921` | `if > 0` | 예약/트레일링 | OK |
| `958` | 없음 | advance 합 (`lh + ls`) | OK |
| `1134` | 없음 | advance 합 (`vpos + lh + ls`) | OK |
| `1266` | 없음 | 트레일링 차감 | 자체 정합 — 5항(잔여) |
| `1941` | `.filter(> 0)` | 호스트 ls 예약 | OK |
| `1947` | `.filter(> 0)` | 호스트 ls 예약 | OK |
| `2017` | `if > 0` | 트레일링 제외 | OK |
| `2238` | 없음 | advance 합 (`th + ls`) | OK |
| **`2452`** | **없음** | **캡션 예약 항** | **결함 (수정 대상)** |

- 클램프됨 7건 / 클램프 없음 7건.
- 클램프 없는 7건 중 **5건은 advance 합산**이며, 렌더러 자신의 advance 공식
  (`layout.rs:727`)과 동형이므로 음수 포함 합산이 의도된 동작이다.
- **예약/트레일링 용도이면서 클램프가 없는 것은 `1266` 과 `2452` 둘뿐**이고,
  그중 **과소 예약(넘침 방향)을 만드는 것은 `2452` 하나뿐**이다.

인접 모듈 `src/renderer/height_cursor.rs` 는 일관되게 클램프한다
(`:284`/`:323`/`:332`/`:421` `.max(0)`, `:316`/`:829`/`:1188` `> 0` 가드,
`:639` 는 의도적 `< 0` 검사). 규약 위반 없음.

### 오차 크기

96 DPI 에서 `hwpunit_to_px(x) = x / 75`, 즉 1 px = 75 HWPUNIT. Task #9 의 음수 ls 는
`ls = (실제 문단 advance) - (표 높이로 부풀려진 lh)` 이므로, 행 2000 HU 짜리 3행 표 기준
`ls ≈ 1200 - 6000 = -4800 HU = -64 px` 이고 같은 표의 행 하나는 26.7 px 다.
→ 오차가 **표 행 약 2.4개 분량**으로, 행 넘김 판정을 뒤집기에 충분하다.

## 3. 변경

`engine.rs:2449-2453` 한 지점에 형제 코드와 동일한 필터를 추가했다.

```rust
// [#2699] 음수 line_spacing(고정값 줄간격 TAC 표 마커, Task #9)은 캡션 예약에서 제외.
// 클램프하지 않으면 :2471의 caption_overhead가 |ls|만큼 작아져 과소 예약이 되고,
// 아래 "Bottom 캡션 공간 확보" 판정이 발동하지 않아 마지막 행+캡션이 본문 하단을 넘는다.
// 렌더러도 음수 ls에서는 y_offset을 더하지 않는다(layout.rs:7154). 형제: :1941/:1947
let host_line_spacing_for_caption = para
    .line_segs
    .first()
    .filter(|seg| seg.line_spacing > 0)
    .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
    .unwrap_or(0.0);
```

`.filter(|seg| seg.line_spacing > 0)` 을 택한 이유: 같은 파일에서 의미가 가장 가까운 형제가
`:1941`/`:1947` 의 `host_line_spacing` 이다. 이름·용도(호스트 문단 ls 를 표 아래 공간으로
예약)가 같고 `Option` 체인 형태
(`.filter(> 0)` → `.map(hwpunit_to_px)` → `.unwrap_or(0.0)`)까지 일치한다.
`seg.line_spacing.max(0)`(`layout.rs:5250`/`:5261` 방식)도 수치 결과는 같지만 그 지점들은
`.filter()` 가 없는 형태라, 가장 가까운 형제의 형태를 따랐다.

## 4. 검증

### 신규 테스트

`bottom_caption_reserve_ignores_negative_host_line_spacing_issue2699`
(`src/renderer/pagination/tests.rs`) — **완전한 end-to-end 페이지네이션 재현**이다.
헬퍼 추출 없이 `paginate()` 를 그대로 구동한다.

구성 근거: 피트 판단에 쓰이는 `effective_height` 는 캡션을 제외하므로
(`engine.rs:1861-1874`) 캡션 예약은 **연속(continuation) 페이지**에서 판정된다.
따라서 2페이지에 걸치는 표로 시나리오를 구성했다.

- 본문 영역 ≈ 876.8 px (a4_page_def, 96 DPI — 실측 확인)
- 표 20행 × 6000 HU(= 80 px) = 1600 px → 분할됨
  - 1페이지: 행 0..10 = 800 px (11행이면 880 px 로 초과)
  - 2페이지: 남은 행 10..20 = 800 px → 행만으로는 들어감 (여유 76.8 px)
- Bottom 캡션 10275 HU = 137 px → 800 + 137 = 937 px > 876.8 px (60 px 초과)
  → 마지막 행(19)을 3페이지로 넘겨야 한다
- 호스트 `line_spacing = -9000 HU`(= -120 px): 클램프가 없으면 예약이 137-120 = 17 px 로 줄어
  800 + 17 = 817 px ≤ 876.8 px (60 px 여유) 가 되어 행 넘김이 사라진다

양쪽 분기 모두 약 60 px 마진을 두어 부동소수점·기하 오차에 둔감하도록 설계했다.

### red→green 실증 (실제 실행 출력)

**RED** — `.filter(|seg| seg.line_spacing > 0)` 한 줄을 제거하고 실행:

```
running 1 test
test renderer::pagination::tests::bottom_caption_reserve_ignores_negative_host_line_spacing_issue2699 ... FAILED

failures:

---- renderer::pagination::tests::bottom_caption_reserve_ignores_negative_host_line_spacing_issue2699 stdout ----

thread 'renderer::pagination::tests::bottom_caption_reserve_ignores_negative_host_line_spacing_issue2699' (20348) panicked at src\renderer\pagination\tests.rs:1404:5:
assertion `left == right` failed: 캡션 예약으로 3개 파트(0..10, 10..19, 19..20)여야 함, partials=[(0, 10), (10, 20)] pages=2
  left: 2
 right: 3
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    renderer::pagination::tests::bottom_caption_reserve_ignores_negative_host_line_spacing_issue2699

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2452 filtered out; finished in 0.00s
```

`partials=[(0, 10), (10, 20)]` 는 예측한 증상 그대로다 — 행 10..20 전체와 캡션이 2페이지에
남아 본문 하단을 넘는다.

**GREEN** — 필터 복원 후 실행:

```
running 1 test
test renderer::pagination::tests::bottom_caption_reserve_ignores_negative_host_line_spacing_issue2699 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2452 filtered out; finished in 0.00s
```

### 회귀

```
cargo test --lib pagination  →  19 passed / 0 failed / 3 ignored   (기준선 18 + 신규 1)
cargo test --lib renderer::  →  906 passed / 0 failed / 4 ignored
```

## 5. 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약
  (`mydocs/manual/codex/docs_and_git_workflow.md`)상 작업지시자 별도 승인 사항이라 실행하지
  않았다.
- **실제 HWP/HWPX 문서를 이용한 시각 검증**은 수행하지 않았다. 재현은 합성 문서 모델을
  구성한 페이지네이션 단위 테스트로 했다. 다만 이는 헬퍼 추출이 아닌 `paginate()` 전체를
  구동하는 end-to-end 경로이며, 분할 결과(`PartialTable` 행 범위)를 직접 단언한다.

## 6. 잔여 — 의도적으로 고치지 않은 것

**`engine.rs:1262-1283` 의 FullParagraph 적합성 검사(`:1266`)는 이번에 클램프하지 않았다.**

```rust
// 문단 적합성 검사: trailing line_spacing 제외
let trailing_ls = para
    .line_segs
    .last()
    .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
    .unwrap_or(0.0);
```

표면적으로 `2452` 와 같은 문제로 보이고, 40여 줄 앞의 거의 동일한 형제 `:639-645` 는
`.filter(|seg| seg.line_spacing > 0)` 과 `.max(0.0)` 을 둘 다 갖고 있어 불일치처럼 읽힌다.
그러나 `:1266` 은 **자체적으로 정합**이다.

차감 대상인 `para_height` 는 `src/renderer/height_measurer.rs:737-742` 의
`lines_total = Σ(lh_i + ls_i)` 에서 나오고, 이때 `line_spacings` 는 같은 파일 `:713-714`
에서 **클램프 없이** 만들어진다. 즉 `para_height` 는 이미 음수 ls 를 접어 넣은 값이므로,
`para_height - trailing_ls` 를 **같은 부호 규약으로** 빼야 "트레일링 제외 높이"가 정확히
복원된다. `trailing_ls` 만 0 으로 클램프하면 `para_height` 속 음수는 남고 차감만 사라져
**오히려 계산이 어긋난다.**

이 지점의 불일치는 `engine.rs` 내부가 아니라 **렌더러와의 불일치**다. `layout.rs:727` 은
advance 를 `hwpunit_to_px(lh + ls, dpi).max(0.0)` 로 바닥을 0 에 고정하는데
`height_measurer` 의 합산에는 그 `.max(0.0)` 이 없다. 그 결과 페이지네이션은 **과대 예약**을
하게 되고 실패 모드는 조기 개행 / 짧은 페이지다. **넘침은 발생하지 않는다** — `2452` 와
정반대의 안전한 방향이다.

따라서:

- `:1266` 에 무턱대고 클램프를 넣으면 자체 정합성이 깨져 현재보다 나빠질 수 있다.
- 올바른 수정은 `height_measurer` 의 합산에 렌더러와 같은 per-line `.max(0.0)` 을 넣는
  것이지만, 모든 문단 높이에 영향을 주는 광범위한 변경이라 이 이슈의 범위를 벗어난다.
- 해당 지점에 근거를 남기는 주석 추가를 권고하며, 별도 이슈로 다루는 편이 안전하다.

이번 변경은 `engine.rs:2449-2453` **한 지점만** 수정한다.
