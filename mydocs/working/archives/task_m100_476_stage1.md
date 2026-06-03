# Task M100 #476 Stage 1 완료 보고서

## 1. 코드 추적 결과

### 1-1. paginator의 PageItem::Shape 등록 위치

`src/renderer/pagination/engine.rs`:
- 라인 935: `fn process_controls()` — paragraph의 컨트롤 처리
- 라인 987-991: `Control::Shape` 분기 — **무조건** `st.current_items.push(PageItem::Shape{...})`
- 라인 1015-1019: `Control::Picture`도 동일 패턴
- 라인 1032-1036: `Control::Equation`도 동일 패턴

### 1-2. 호출 컨텍스트

`engine.rs:324-339`:

```rust
// 비-표 문단 처리
if !has_table {
    self.paginate_text_lines(...);   // line 324: 텍스트 라인을 페이지에 분배 (페이지 분할 발생 가능)
}
let height_before_controls = st.current_height;
let page_count_before_controls = st.pages.len();

// 인라인 컨트롤 감지 (표/도형/각주)
self.process_controls(...);   // line 335: 컨트롤 처리 (PageItem::Shape 등록)
```

**핵심**: `paginate_text_lines`가 paragraph를 페이지 분할할 수 있고, 그 후 `process_controls`가 호출된다. 따라서 `st.current_items`는 항상 **마지막 페이지** 상태 → PageItem::Shape는 항상 마지막 페이지에 등록된다.

### 1-3. 결과

paragraph 238 (line 0 = 페이지 11, lines 4..26 = 페이지 12)의 인라인 박스(char 0 → line 0)는:
- 박스가 그려져야 하는 페이지: **페이지 11**
- 그러나 paginator는 PageItem::Shape를 **페이지 12**에 등록
- 페이지 12의 layout 패스가 inline_pos를 찾지 못해 fallback 위치(y=742.45)에 박스 출현

이것이 본 이슈의 paginator 측 근본 원인이다.

## 2. D 임시 차단 적용

### 2-1. 변경 위치

`src/renderer/layout/shape_layout.rs:218` (추가 분기):

```rust
let inline_pos = if common.treat_as_char {
    tree.get_inline_shape_position(section_index, para_index, control_index)
} else {
    None
};
// [Issue #476] treat_as_char Shape는 paragraph_layout이 inline_pos 등록 후
// 본 함수가 그려야 한다. inline_pos 가 없는 경우는 paginator 가 PageItem::Shape 를
// 잘못된 페이지(박스가 속한 line이 라우팅되지 않은 페이지)에 등록한 결과이며,
// compute_object_position fallback 으로 그리면 절대 좌표(예: 문단 오프셋=0,0)
// 기준의 잘못된 위치에 박스가 출현한다 (= 다른 paragraph 영역에 침범).
// 본질 수정 전까지(paginator A 단계) fallback 그리기를 차단한다.
if common.treat_as_char && inline_pos.is_none() {
    if std::env::var("RHWP_DEBUG_LAYOUT").is_ok() {
        eprintln!("[#476 skip] inline Shape without inline_pos: sec={} para={} ci={}",
            section_index, para_index, control_index);
    }
    return;
}
```

### 2-2. 차단 조건과 의미

| 조건 | 의미 |
|------|------|
| `treat_as_char == true` | 인라인 도형(글자처럼 처리) — paragraph_layout이 그려야 정상 |
| `inline_pos.is_none()` | paragraph_layout이 박스의 라인을 처리하지 않아 좌표가 등록되지 않음 → 페이지 라우팅 오류 시그널 |

### 2-3. 비-TAC Shape는 영향 없음

`treat_as_char == false`인 Shape(글앞으로/글뒤로/어울림 등)는 inline_pos 검사를 통과하지 않으므로 기존 fallback 경로(`compute_object_position`)가 그대로 사용된다.

### 2-4. 안전장치

`RHWP_DEBUG_LAYOUT` 환경변수가 설정된 경우 차단된 박스를 stderr에 로깅하여, 정상 케이스에서 의도치 않게 차단되는 회귀를 빠르게 감지할 수 있다.

## 3. 빌드 결과

```
$ cargo build --release
   Compiling rhwp v0.7.8 (/Users/planet/rhwp)
    Finished `release` profile [optimized] target(s) in 58.47s
```

빌드 통과.

## 4. 다음 단계

Stage 2 — D 차단 적용 후 시각 검증 및 광범위 회귀 테스트.
