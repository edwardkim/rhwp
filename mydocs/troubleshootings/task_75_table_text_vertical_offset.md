---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 트러블슈팅: 표 앞 텍스트가 표 아래에 렌더링되는 문제

## 문제 상황

`samples/hwp-multi-001.hwp` 2페이지에서 문단28에 포함된 텍스트("※ 2024年 해외직접투자 상세통계는...")가 표8 아래에 렌더링되거나 아예 누락되는 문제.

### 문서 구조

```
문단21: 표7 (4행×6열) — 1페이지
문단22~27: 텍스트 문단
문단28: 표8 (2행×6열) + 텍스트 2줄 — 2페이지
문단29: 묶음 (그룹 이미지)
```

### 문단28의 특수성

문단28은 **표 제어문자와 텍스트를 동시에 포함**하는 혼합 문단이다.

- 문단 텍스트: "※ 2024年 해외직접투자 상세통계는 별첨 참고자료를 확인해주시기 바랍니다..." (284자, 2줄)
- line_segs: 2개 (LineSeg[0] text_start=0, LineSeg[1] text_start=71)
- char_offsets[0] = 8 → 표 제어문자(8 code unit)가 문자 스트림 시작에 위치
- 표 속성:
  - 본문과의 배치: **자리차지**
  - 세로: 문단 위 기준 **9.77mm**
  - 가로: 문단 왼쪽 기준 0.00mm

## 분석 과정

### 1차 시도: char_offsets 기반 접근 (실패)

타스크 66에서 텍스트+표 혼합 문단 처리를 위해 `PartialParagraph(start_line=1)`을 표 뒤에 배치하는 방식을 도입했다. 이 방식은 표가 문단 시작에 있고 텍스트가 뒤에 오는 일반적인 경우에 잘 동작했다.

문단28에서 문제가 발생한 이유:
- `char_offsets[0] = 8` → 표 제어문자가 char 스트림 시작(code unit 0~7)에 위치
- char_offsets 갭 분석으로는 "표가 텍스트 앞에 있다"고 판단 → 모든 텍스트를 표 뒤에 배치
- 결과: 텍스트가 표 아래에 렌더링되거나 누락

**근본 원인**: char_offsets는 문자 스트림 내 제어문자의 **논리적 위치**만 알려줄 뿐, 표의 **물리적 배치 위치**는 알 수 없다.

### 핵심 발견: 표의 CTRL_HEADER vertical_offset

HWP 도움말 문서(`objectattribute(arrange).htm`)에서 **자리차지** 배치 개념을 확인:
> "개체가 개체의 높이만큼 줄을 차지하고 있기 때문에 개체가 차지하고 있는 영역에는 본문이 오지 못합니다."

표의 CTRL_HEADER에는 `vertical_offset` 필드가 있다:

```
raw_ctrl_data 레이아웃 (attr 4바이트 이후):
  [0..4]  vertical_offset  (u32, HWPUNIT)
  [4..8]  horizontal_offset (u32, HWPUNIT)
  [8..12] width            (u32, HWPUNIT)
  [12..16] height           (u32, HWPUNIT)
  ...
```

문단28의 표8: `vertical_offset = 9.77mm ≈ 2769 HWPUNIT (> 0)`

이 값이 > 0이면 표가 문단 시작점 아래에 위치하므로, 그 공간에 텍스트 줄이 먼저 배치되어야 한다.

## 해결

### 수정 내용 (`pagination.rs`)

기존 `find_table_char_position()` / `find_line_for_char_pos()` 두 헬퍼를 제거하고, `get_table_vertical_offset()` 하나로 교체:

```rust
fn get_table_vertical_offset(table: &Table) -> u32 {
    if table.raw_ctrl_data.len() >= 4 {
        u32::from_le_bytes(table.raw_ctrl_data[0..4].try_into().unwrap())
    } else {
        0
    }
}
```

표 앞/뒤 텍스트 분리 로직:

```rust
let vertical_offset = Self::get_table_vertical_offset(table);

let pre_table_end_line = if vertical_offset > 0 && !para.text.is_empty() {
    total_lines  // 모든 텍스트 줄을 표 앞에 배치
} else {
    0            // 텍스트는 표 뒤에 배치 (기존 동작)
};
```

### 동작 원리

| vertical_offset | 의미 | 텍스트 배치 |
|-----------------|------|------------|
| > 0 | 표가 문단 시작점 아래 (자리차지) | 모든 텍스트 → 표 앞 |
| = 0 | 표가 문단 시작 위치 | 모든 텍스트 → 표 뒤 (타스크 66 동작) |

### 수정 전후 비교

**수정 전** (char_offsets 기반):
```
[표8]
[텍스트 누락 또는 표 아래]
```

**수정 후** (vertical_offset 기반):
```
[※ 2024年 해외직접투자 상세통계는...]  ← y=529.83
[통계는 한국수출입은행 해외투자통계...]  ← y=548.92
[표8: 담당부서 | 대외경제국...]          ← y=569.89
```

## 검증

- 488개 Rust 테스트 통과
- `hwp-multi-001.hwp` SVG 내보내기: 2페이지 텍스트 2줄이 표8 위에 정상 렌더링
- `img-start-001.hwp` SVG 내보내기: 기존 동작 유지 (회귀 없음)
- WASM 빌드 성공
- 웹 브라우저 렌더링 정상 확인

## 교훈

1. **문자 스트림 위치 ≠ 물리적 렌더링 위치**: char_offsets의 논리적 순서만으로는 개체의 실제 배치를 결정할 수 없다.
2. **CTRL_HEADER의 위치 정보 활용**: `vertical_offset`/`horizontal_offset`은 "자리차지" 배치에서 개체의 물리적 위치를 결정하는 핵심 데이터이다.
3. **HWP 도움말 참조**: 스펙 문서뿐 아니라 한컴 도움말의 기능 설명이 렌더링 로직 설계에 중요한 단서를 제공한다.
