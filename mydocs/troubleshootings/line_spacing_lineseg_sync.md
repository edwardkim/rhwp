---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 줄간격 변경 시 LineSeg 동기화 문제

## 증상

서식 도구 모음에서 줄간격을 변경(예: 160% → 300%)해도 문서에 시각적 변화가 없음.
HWP 저장 후 한컴에서 열면 스타일바에는 300%로 표시되지만, 실제 렌더링은 160%로 보임.

## 원인

HWP 문서의 줄간격은 두 곳에 저장된다:

1. **ParaShape** (`line_spacing`, `line_spacing_type`) — 문단의 논리적 줄간격 설정
2. **LineSeg** (`line_spacing`) — 각 줄의 물리적 줄간격 값 (HWPUNIT)

### 렌더링 파이프라인

```
ParaShape (논리값)
    ↓ (문서 로드 시 1회만 계산)
LineSeg (물리값)
    ↓
compose_lines() → ComposedLine.line_spacing
    ↓
layout → 실제 Y 좌표 계산
```

`compose_lines()`는 **LineSeg의 값을 그대로 사용**한다 (line 193-199 of `composer.rs`):
```rust
lines.push(ComposedLine {
    line_height: line_seg.line_height,
    line_spacing: line_seg.line_spacing,  // LineSeg에서 직접 복사
    ...
});
```

### 문제 1: `apply_para_format`이 LineSeg를 갱신하지 않음

`apply_para_format_native()`는 ParaShape만 변경하고 `rebuild_section()`을 호출했다.
그러나 `rebuild_section()` → `recompose_section()` → `compose_section()`은
기존 LineSeg를 읽기만 하므로, 변경된 ParaShape의 줄간격이 반영되지 않았다.

### 문제 2: `reflow_line_segs`가 기존 값을 보존

`reflow_line_segs()`의 `make_line_seg` 클로저에서, 원본 LineSeg가 유효하면
(`line_height > 0`) **모든 dimension을 원본에서 복사**했다:

```rust
// 수정 전 (문제 코드)
if let (true, Some(ref o)) = (has_valid_orig, &orig) {
    LineSeg {
        line_spacing: o.line_spacing,  // ← 원본 값 그대로 (ParaShape 무시)
        ...
    }
}
```

이는 문서 로드 시 원본 HWP의 LineSeg dimension을 보존하기 위한 의도였으나,
줄간격을 UI에서 변경한 후에도 원본 값이 유지되는 부작용이 있었다.

## 수정

### 1단계: `formatting.rs` — LineSeg 재계산 호출 추가

`apply_para_format_native()`에서 줄간격 관련 mods가 있을 때,
`rebuild_section()` 전에 `reflow_line_segs()`를 호출하여 LineSeg를 갱신:

```rust
if mods.line_spacing.is_some() || mods.line_spacing_type.is_some() {
    let styles = resolve_styles(&self.document.doc_info, self.dpi);
    // ... available_width 계산 ...
    reflow_line_segs(&mut para, available_width, &styles, self.dpi);
}
```

셀 내 문단(`apply_para_format_in_cell_native`)에도 동일하게 적용.

### 2단계: `line_breaking.rs` — 기존 LineSeg에서도 line_spacing 재계산

`make_line_seg` 클로저에서, 원본 LineSeg가 유효해도 `line_spacing`만은
ParaShape에서 재계산하도록 수정:

```rust
// 수정 후
if let (true, Some(ref o)) = (has_valid_orig, &orig) {
    let line_spacing_hwp = compute_line_spacing_hwp(ls_type, ls_value, o.line_height, dpi);
    LineSeg {
        line_height: o.line_height,          // 원본 보존
        text_height: o.text_height,          // 원본 보존
        baseline_distance: o.baseline_distance, // 원본 보존
        line_spacing: line_spacing_hwp,      // ← ParaShape에서 재계산
        ...
    }
}
```

## 핵심 교훈

- HWP에서 **ParaShape 변경만으로는 렌더링에 반영되지 않는다**.
  LineSeg가 실제 렌더링의 물리적 값을 보유하며, compose/layout은 LineSeg를 직접 참조한다.
- ParaShape는 "설정값", LineSeg는 "계산된 레이아웃 값"이라는 이중 구조를 인식해야 한다.
- 줄간격 외에도 줄 높이(`line_height`), 텍스트 높이(`text_height`) 등
  ParaShape/CharShape 변경 시 LineSeg 동기화가 필요할 수 있다.

## 관련 파일

| 파일 | 역할 |
|------|------|
| `src/document_core/commands/formatting.rs` | `apply_para_format_native` — ParaShape 변경 + LineSeg 재계산 |
| `src/renderer/composer/line_breaking.rs` | `reflow_line_segs` — LineSeg 생성/갱신 |
| `src/renderer/composer.rs` | `compose_lines` — LineSeg → ComposedLine 변환 |
| `src/model/paragraph.rs` | `LineSeg` 구조체 정의 |
