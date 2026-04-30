# Task #470 구현계획서

## 수정 대상

### 1차: `src/renderer/typeset.rs:415, 439`

`cv == 0` / `nv == 0` → `cv < pv` / `nv < cl` 으로 완화. `pv > 5000` / `cl > 5000` 가드는 유지.

### 2차: `src/renderer/pagination/engine.rs`

현재 cross-paragraph vpos-reset 검출이 부재. typeset.rs 와 동일 로직 추가하여 일관성 확보 (`RHWP_USE_PAGINATOR=1` 시).

### 3차: 신규 통합 테스트

`src/renderer/layout/integration_tests.rs` 또는 별도 모듈에 다음 검증 추가:
- 21_언어_기출_편집가능본 1p col 0 에 pi=10 가 배치되지 않음
- 21_언어_기출_편집가능본 1p col 1 첫 항목이 pi=10 (start_line=0) 임

## 변경 코드

### typeset.rs L415

```rust
// before
if cv == 0 && pv > 5000 {
    st.advance_column_or_new_page();
}
// after
// Task #470: cv == 0 만 검출하면 컬럼 헤더 오프셋 (cv=9014 등) 으로 시작하는
// 새 컬럼의 vpos-reset 을 놓침. HWPUNIT vpos 는 컬럼 내 단조 증가하므로
// cv < pv 는 항상 reset 시그널.
if cv < pv && pv > 5000 {
    st.advance_column_or_new_page();
}
```

### typeset.rs L439

```rust
// before
matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl)) if nv == 0 && cl > 5000)
// after
matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl)) if nv < cl && cl > 5000)
```

### pagination/engine.rs (신규 추가)

`paginate_paragraph` 진입 직후 (force_page_break 처리 후) 동일 가드 추가.

위치: `src/renderer/pagination/engine.rs` 의 `paginate_with_measured_opts` 메인 루프 내 force-break 처리 직후 (현재 구조 추가 분석 필요).

## 단계 구성

### Stage 1: 신규 통합 테스트 작성 (red)

`samples/21_언어_기출_편집가능본.hwp` 페이지 1 의 PaginationResult 를 직접 호출하여 col 0 의 PageItem 목록에 pi=10 PartialParagraph 가 없음을 검증. 또는 LAYOUT_OVERFLOW 메시지가 발생하지 않음을 검증.

### Stage 2: typeset.rs + pagination/engine.rs 수정

조건 완화 적용 후 Stage 1 테스트 green 확인.

### Stage 3: 회귀 검증

- `cargo test --release` 전체
- exam_kor / exam_eng / hwpspec-w 다단 샘플 페이지 수 비교
- LAYOUT_OVERFLOW 메시지 변화 검토

## 검증 명령

```bash
cargo build --release
cargo test --release
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 -o /tmp/p0/ 2>&1 | grep OVERFLOW   # 비어있어야 함
./target/release/rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 0 2>&1 | head -25
# 회귀 점검
for f in exam_kor exam_eng hwpspec-w; do
  echo "=== $f ==="
  ./target/release/rhwp export-svg samples/$f.hwp -o /tmp/regress/$f/ 2>&1 | grep -c OVERFLOW
done
```
