# 단계 3 보고서 — Task #356 페이지네이션 엔진 통합

- **단계**: 3/5
- **브랜치**: `local/task356`

## 변경 사항

### 1. 함수 가시성 변경

`src/renderer/pagination/engine.rs`:
```rust
pub fn detect_inter_paragraph_vpos_reset(...) -> bool
```

### 2. 모듈 재내보내기

`src/renderer/pagination.rs`:
```rust
pub use engine::detect_inter_paragraph_vpos_reset;
```

### 3. 헬퍼 정의 정제

`prev.last.vpos_end` 대신 `prev.first.vertical_pos` 비교로 정의 변경:

```rust
cur_first.vertical_pos < prev_first.vertical_pos
```

이유:
- vpos_end 비교는 prev 의 lh/ls 누적이 정확할 때만 성립.
- 합성 테스트나 비정상 데이터에서 lh>0 / vpos=0 인 경우 false positive 발생.
- "cur 이 prev 시작점보다 위 = 새 페이지/단" 가 더 강한 의미적 신호.

단위 테스트 6개 모두 통과 (조건 변경 반영).

### 4. 페이지네이션 엔진 통합 (`paginate_with_measured_opts`)

`process_page_break` 처리 직후, 본 문단 처리 진입부에 추가:

```rust
if !st.current_items.is_empty() {
    if let Some(prev_pi) = prev_pagination_para {
        if let Some(prev_para) = paragraphs.get(prev_pi) {
            if detect_inter_paragraph_vpos_reset(prev_para, para) {
                st.advance_column_or_new_page();
            }
        }
    }
}
```

게이팅:
- `current_items.is_empty()` → 페이지 첫 문단/직전 분기 직후 자동 제외
- `prev_pagination_para` Option → 첫 문단(None) 자동 제외
- 헬퍼 내부의 `column_start` 일치 체크 → 다단 단 변경과 구분

### 5. typeset.rs 통합 (보조 트리거)

`src/renderer/typeset.rs` Task #321 위치에 헬퍼를 보조 트리거로 추가:

```rust
// cv==0 strict 트리거 (기존 #321) 우선, 미발화 시 헬퍼 보조 트리거
let mut advance = false;
if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
    if cv == 0 && pv > 5000 { advance = true; }
}
if !advance && detect_inter_paragraph_vpos_reset(prev_para, para) {
    advance = true;
}
if advance { st.advance_column_or_new_page(); }
```

이중 트리거를 사용하는 이유:
- 기존 `cv==0` 조건은 다단 레이아웃에서 column_start 가 다른 경우(`cv==0` 이지만 column_start≠0)에도 advance 되어 column 변경을 처리해 왔음
- 우리 헬퍼는 column_start 가 다르면 false 반환하므로 다단 처리 누락 발생 가능
- 따라서 기존 cv==0 트리거는 그대로 두고, 헬퍼는 cv≠0 부분 리셋(예: cv=500)을 보충하는 보조 트리거로 운용

## 검증

### 단위 테스트

```
cargo test --release inter_para_vpos_reset_tests
test result: ok. 6 passed; 0 failed
```

### 전체 회귀

```
cargo test --release
test result: ok. 1014 passed; 0 failed; 1 ignored
+ 14 + 25 + 6 + 1 + 1 PASS
```

### 골든 SVG (`tests/golden_svg/`)

```
cargo test --release --test svg_snapshot
6 passed; 0 failed
(form-002, issue-147, issue-157, issue-267, table-text, deterministic)
```

### 본 샘플 동작 확인

`./target/release/rhwp dump-pages "samples/2022년 국립국어원 업무계획.hwp" -p 2`:

```
페이지 3: items=20 (이전 23), used=792.6px (이전 913.9px)
  ...
  Table  pi=38  vpos=60167
  FullParagraph  pi=39  vpos=66281..68681  ← 페이지 3 마지막
페이지 4: pi=40 (vpos=500) ~  ← 정상적으로 새 페이지 시작
```

`export-svg`: LAYOUT_OVERFLOW **0건** (이전 5+건)

### 다중 샘플 회귀 비교 (overflow / 페이지 수)

| 샘플 | 베이스라인 | 본 fix |
|------|-----------|--------|
| `2022년 국립국어원 업무계획.hwp` | 35p / 5+ | **35p / 0** |
| `aift.hwp` | 74p / 30 | **86p / 16** |
| `exam_eng.hwp` | 8p / 0 | **8p / 0** (회귀 없음) |
| `exam_math.hwp` | 20p / 0 | **20p / 0** |
| `2010-01-06.hwp` | 6p / 0 | **6p / 0** |

- 본 샘플: 명시 증상 100% 해결
- aift: 12 페이지 추가 + overflow 30→16 (47% 감소). 잔여 16건은 본 fix 와 별개 케이스 → 단계 4 에서 분석
- 다단 샘플 (exam_eng) 회귀 없음

## 발견 및 보존

- 초기 통합 시 typeset.rs 의 `cv==0 strict` 트리거를 헬퍼로 단순 치환했더니 exam_eng 가 7건 overflow 회귀. 다단 column 변경 케이스에서 `column_start` 가 달라 헬퍼가 false 반환하기 때문. 이중 트리거(cv==0 strict + 헬퍼 보조)로 해결.
- 헬퍼 정의에서 `vpos_end` 가 아닌 `prev.first.vpos` 를 쓰는 이유는 합성 테스트 회귀(test_partial_paragraph_after_content) 발견 후 결정.

## 다음 단계

**단계 4**: 골든 SVG 픽셀 diff 분석, aift.hwp 잔여 16건 overflow 케이스 분류 (본 fix 적용 가능 / 별개 이슈 분리), 결과 표 정리.
