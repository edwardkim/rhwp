# Task #30 완료 보고서: 문단 삽입/삭제 후 페이지 수 과도 증가 수정

## 수행 결과

### 1단계: 버그 수정 ✅

**파일**: `src/document_core/queries/rendering.rs` (L601-L631)

`insert_composed_paragraph`와 `remove_composed_paragraph`에서 `dirty_paragraphs` 비트맵 조작을 `None` 설정으로 변경:

```rust
// Before (버그): 비트맵 insert/remove → prev_measured 인덱스와 불일치
if let Some(bits) = &mut self.dirty_paragraphs[section_idx] {
    bits.insert(para_idx, true);
}

// After (수정): 전체 재측정 강제 → 인덱스 불일치 원천 차단
self.dirty_paragraphs[section_idx] = None;
```

### 2단계: 회귀 테스트 ✅

`test_split_paragraph_page_count_stability` 테스트 추가 (`src/wasm_api/tests.rs`):
- kps-ai.hwp 로드 → `splitParagraph(0, 199, 0)` → 페이지 수 증가 ≤ 2 검증
- 수정 전: 78 → 86 (delta = +8) **FAIL**
- 수정 후: 78 → 78 (delta = 0) **PASS**

### 3단계: 기존 테스트 통과 확인 ✅

778개 테스트 전체 통과, 0 실패.
