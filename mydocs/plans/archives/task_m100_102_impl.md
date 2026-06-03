# Task #102: 구현계획서 (v2) — 문단 줄 단위 페이지 경계 방어 로직

> **이슈**: [#102](https://github.com/edwardkim/rhwp/issues/102)
> **브랜치**: `local/task102`
> **작성일**: 2026-04-11
> **수정일**: 2026-04-11 (v2 — 전문가 검토 피드백 반영)

---

## 현황 분석

### 핵심 함수: `paginate_text_lines` (engine.rs:581-744)

문단 줄 배치 흐름:

1. **FullParagraph 적합성 판단** (라인 601-626): `current_height + para_height <= available_now + 0.5`이면 전체 배치
2. **줄 단위 분할 루프** (라인 648-735):
   - 첫 줄 체크 (라인 642-646): `remaining_for_lines < first_line_h`이면 다음 페이지
   - 줄 범위 결정 (라인 664-671): `avail_for_lines` 기준으로 `end_line` 결정
   - FullParagraph 낙착 시 (라인 681-695): `overflow_threshold > avail_for_lines` 체크 ✅
   - **PartialParagraph 배치 후** (라인 706-718): overflow 체크 없음 ❌

### 문제 경로 (pi=65)

- 18페이지 bottom = 1034.1px
- `cursor_line=0`, `end_line=1 < line_count=2` → PartialParagraph 경로
- `current_height += part_height` 후 1047.3px → 13.2px 초과
- 방어 체크 없어 overflow 상태로 배치 확정

---

## 방어 전략: 2개 레이어

### 레이어 1: 배치 전 방어 — 고정 GUARD_HEIGHT 구역

**위치**: `paginate_text_lines` 줄 범위 결정 루프 내 PartialParagraph 배치 직전

페이지 끝 근방 고정 구역 안에서 `part_height`가 남은 공간을 초과하면 다음 페이지로 이동.

```rust
// TODO: 높이 계산 오차에 대한 임시 방어 로직 (레이어 1).
// current_height 누적이 정확해지면 이 코드는 제거 가능하다.
const PAGE_GUARD_HEIGHT: f64 = 20.0; // 약 한 줄 높이
let available_now = st.available_height();
let guard_zone_start = available_now - PAGE_GUARD_HEIGHT;
if cursor_line == 0
    && st.current_height >= guard_zone_start
    && st.current_height + part_height > available_now + 0.5
    && !st.current_items.is_empty()   // 빈 페이지면 강제 배치 (무한 루프 방지)
{
    guard_advance_count += 1;
    if guard_advance_count <= line_count + 2 {
        st.advance_column_or_new_page();
        continue;
    }
    // 안전망: 반복 횟수 초과 시 강제 배치 (비정상 경로)
}
```

**`para_height` 기준 대신 고정값 사용 이유**:
`para_height` 자체가 오차를 포함할 수 있어, 오차에 의존하는 방어 구역은 신뢰성이 떨어짐.
고정값은 `para_height` 오차에 독립적으로 작동.

**적용 조건**:
- `cursor_line == 0`: 문단 첫 배치 시점에만 (연속 페이지에서는 미적용)
- `current_height >= guard_zone_start`: 방어 구역 진입 시에만
- `overflow > 0.5px`: 부동소수점 오차 허용 범위 초과 시에만
- `!current_items.is_empty()`: 빈 페이지면 강제 배치 (무한 루프 방지)
- `guard_advance_count <= line_count + 2`: 반복 횟수 상한 (무한 루프 안전망)

**레이어 2와의 상호 배제**:
레이어 1이 `advance_column_or_new_page()`를 호출할 때 레이어 2가 *다른 항목*을 carry로 꺼내
순서 역전이 발생하지 않도록, `advance_column_or_new_page()` 호출 시
`layer1_advancing: bool` 플래그를 설정하여 레이어 2가 스킵하도록 한다.

```rust
st.layer1_advancing = true;
st.advance_column_or_new_page();
st.layer1_advancing = false;
```

---

### 레이어 2: 페이지 전환 시 직전 마지막 항목 점검

**위치**: `state.rs`의 `advance_column_or_new_page` 내 `flush_column` 직전

페이지/단이 전환되는 모든 경로에서 `current_items`의 마지막 항목이 실제로 overflow인지 확인.
overflow 시 해당 항목을 꺼내 다음 페이지/단 첫 항목으로 이동.

```rust
pub fn advance_column_or_new_page(&mut self) {
    // TODO: 높이 계산 오차에 대한 임시 방어 로직 (레이어 2).
    // current_height 누적이 정확해지면 이 코드는 제거 가능하다.
    if !self.layer1_advancing {
        self.check_last_item_overflow();
    }
    self.flush_column();
    if self.current_column + 1 < self.col_count {
        self.current_column += 1;
        self.current_height = 0.0;
        self.reinsert_carry_if_any();   // 단 전환 후 carry 재삽입
    } else {
        self.push_new_page();           // push_new_page 내에서 reinsert_carry_if_any 호출
    }
}
```

#### `check_last_item_overflow`

```rust
fn check_last_item_overflow(&mut self) {
    // overflow_carry가 이미 있으면 스킵 (중복 방지)
    if self.overflow_carry.is_some() { return; }
    // FullParagraph / PartialParagraph만 대상 (표·글상자 제외)
    let is_para_item = self.current_items.last().map_or(false, |item| {
        matches!(item, PageItem::FullParagraph { .. } | PageItem::PartialParagraph { .. })
    });
    if !is_para_item { return; }
    let available = self.available_height();
    if self.current_height <= available + 0.5 { return; }
    // overflow: 마지막 항목을 꺼내 다음 페이지/단으로 예약
    self.overflow_carry = self.current_items.pop();
}
```

#### `reinsert_carry_if_any`

carry 재삽입 시 `current_height` 보정 및 `page_vpos_base` 설정 포함:

```rust
fn reinsert_carry_if_any(&mut self, measured: &MeasuredSection) {
    let carry = match self.overflow_carry.take() {
        Some(c) => c,
        None => return,
    };
    // current_height 보정 (필수: 누락 시 이후 모든 배치 계산 오류)
    let carry_h = match &carry {
        PageItem::FullParagraph { para_index } => {
            measured.get_measured_paragraph(*para_index)
                .map(|mp| mp.total_height())
                .unwrap_or(0.0)
        }
        PageItem::PartialParagraph { para_index, start_line, end_line } => {
            measured.get_measured_paragraph(*para_index)
                .map(|mp| mp.line_advances_sum(*start_line..*end_line))
                .unwrap_or(0.0)
        }
        _ => 0.0,
    };
    // page_vpos_base 설정 (carry 항목의 첫 줄 기준)
    if self.page_vpos_base.is_none() {
        // carry 항목의 para/line 정보로 vpos_base 설정은 engine.rs에서 처리
        self.pending_vpos_base_from_carry = true;
    }
    self.current_items.push(carry);
    self.current_height += carry_h;
}
```

**`PaginationState` 추가 필드**:
- `overflow_carry: Option<PageItem>` — carry 항목
- `layer1_advancing: bool` — 레이어 간 상호 배제
- `pending_vpos_base_from_carry: bool` — carry 후 vpos_base 설정 대기
- `defense_counts: HashMap<usize, u32>` — 페이지별 방어 실행 횟수

### 페이지별 방어 횟수 관리 (`defense_counts`)

레이어 1과 레이어 2가 같은 `HashMap<usize, u32>`를 공유하여 페이지 단위로 방어 실행 횟수를 누적한다.

```rust
// 방어 발동 시 (레이어 1, 레이어 2 공통)
let page_idx = self.pages.len();
let count = self.defense_counts.entry(page_idx).or_insert(0);
*count += 1;
if *count > DEFENSE_MAX_PER_PAGE {
    // 상한 초과 → 강제 배치 (무한 루프 최종 차단)
}
```

**`DEFENSE_MAX_PER_PAGE` 상수**: 페이지당 최대 방어 횟수 (예: 100).
정상 문서에서는 절대 도달하지 않는 값으로 설정.

**장점**:
- 메모리: 페이지당 12바이트(usize + u32). 200페이지 문서 기준 ~2.4KB로 부담 없음
- 통합 관리: 레이어 1·2 어느 경로든 같은 페이지 카운트 누적 → 상한 하나로 전체 제어
- 유연성: `DEFENSE_MAX_PER_PAGE` 상수 하나로 모든 레이어 일괄 조정
- 디버깅: 렌더링 완료 후 맵 덤프로 어느 페이지에서 방어가 몇 번 발동했는지 확인 가능
- 확장성: 향후 방어 로직 추가 시 같은 맵에 카운트만 추가

레이어 1의 `guard_advance_count` 지역 변수는 제거하고 `defense_counts`로 일원화한다.

**무한 루프 방지 정리**:
- `overflow_carry.is_some()` 가드: carry 중복 방지
- `layer1_advancing` 플래그: 레이어 1 advance 시 레이어 2 스킵
- 새 페이지 `current_height = 0` 리셋 + carry_h 보정: 재삽입 후 정상 상태 보장
- `force_new_page`(쪽 나누기): `check_last_item_overflow` 미호출 (의도적 — 쪽 나누기 의미 보존)

**적용 대상**:
- `FullParagraph`, `PartialParagraph`만
- 표(`Table`, `PartialTable`), 글상자 등 제외

---

## 단계별 구현

### 1단계: 레이어 1 구현 (engine.rs)

**파일**: `src/renderer/pagination/engine.rs`

1. `PAGE_GUARD_HEIGHT` 상수 정의
2. `PaginationState`에 `layer1_advancing: bool` 필드 추가
3. `paginate_text_lines` 줄 분할 루프 진입 전 `guard_advance_count: usize = 0` 초기화
4. PartialParagraph 배치 직전 레이어 1 guard 삽입 (`guard_advance_count` 상한 포함)
5. `cargo test` 전체 통과 확인

**검증**:
- `dump-pages -p 17`: pi=65 클리핑 해소 확인
- `export-svg -p 17 --debug-overlay`: LAYOUT_OVERFLOW page=5 para=65 제거

### 2단계: 레이어 2 구현 (state.rs + engine.rs)

**파일**: `src/renderer/pagination/state.rs`

1. `PaginationState`에 `overflow_carry`, `pending_vpos_base_from_carry` 필드 추가
2. `check_last_item_overflow` 함수 구현
3. `reinsert_carry_if_any` 함수 구현 (`current_height` 보정 + `page_vpos_base` 처리 포함)
4. `advance_column_or_new_page`에 레이어 2 통합 (단 전환 + 페이지 전환 양쪽 carry 재삽입)
5. `cargo test` 전체 통과 확인

**검증**:
- 전체 226개 샘플 LAYOUT_OVERFLOW 건수 비교 (새 회귀 0건)

### 3단계: 전체 회귀 테스트

- 226개 샘플 전체 SVG 내보내기
- LAYOUT_OVERFLOW 잔존 4건 → 감소 확인, 새 회귀 0건

---

## 엣지 케이스 처리 방침

| 케이스 | 처리 |
|--------|------|
| `line_count == 1` | 레이어 1 미작동 (FullParagraph 경로). 기존 `overflow_threshold` 체크에 의존 |
| 빈 페이지 overflow | 레이어 1/2 모두 `!current_items.is_empty()` 가드로 강제 배치 (무한 루프 방지) |
| 다단 단 전환 | `reinsert_carry_if_any`를 단 전환 분기에도 호출하여 carry 소실 방지 |
| 쪽 나누기(`force_new_page`) | 레이어 2 미작동 (의도적 — 쪽 나누기 의미 보존) |
| 각주 과점유 (`guard_zone_start < 0`) | `guard_advance_count` 상한으로 무한 루프 차단 |

---

## 제약 조건

- 표·글상자 배치 로직 변경 없음 (`paginate_table_control` 무변경)
- `force_new_page` 경로에서 레이어 2 미작동 (의도적)
- TODO 주석 필수: 근본 수정 시 제거 가능하도록
- 전체 샘플 새 회귀 0건 유지
