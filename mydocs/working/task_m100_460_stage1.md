---
타스크: #460 HWP3 파서 렌더러 중립 재구현
문서: Stage 1 완료 보고서
브랜치: local/task460
작성일: 2026-04-29
---

# Stage 1 완료 보고서

## 수행 내용

### 1. `src/parser/hwp3/mod.rs` — AutoNumber U+FFFC → 공백 치환

**변경 위치**: 줄 201 (`18..=21` match arm)

```rust
// 수정 전
text_string.push('\u{FFFC}');

// 수정 후
// AutoNumber(ch=18)은 HWP5 패턴("  ")과 일치하도록 공백으로 저장
if ch == 18 {
    text_string.push(' ');
} else {
    text_string.push('\u{FFFC}');
}
```

캡션 텍스트 "그림 " + `' '` = `"그림  "` → HWP5/HWPX와 동일한 `"  "` 패턴 ✓

### 2. `src/renderer/layout/paragraph_layout.rs` — U+FFFC 분기 제거

**변경 위치**: 줄 2783-2797

```rust
// 수정 전: HWP3 전용 '\u{fffc}' 탐색 분기 포함 (4줄)
// HWP3: AutoNumber 위치를 U+FFFC로 저장 → '\u{fffc}' 탐색
...
if run.text.contains('\u{fffc}') {
    run.text = run.text.replacen('\u{fffc}', &num_str, 1);
    return;
}

// 수정 후: HWP5/HWPX/HWP3 공통 "  " 패턴만 사용
// HWP5/HWPX/HWP3 공통: 공백 두 개("  ") 패턴 탐색
for line in &mut composed.lines { ... }
```

렌더러에서 HWP3 전용 코드 완전 제거 ✓

## 검증 결과

```
cargo test --lib
test result: ok. 1068 passed; 0 failed; 1 ignored; 0 measured
```

기존 통과 수(1068) 유지, 회귀 0 ✓
