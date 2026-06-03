# Task #688 1단계 완료보고서 — 1×1 래퍼 감지 조건 정밀화

## 변경 사항

`src/renderer/layout/table_layout.rs::layout_table()` 라인 150-170 수정.

### 변경 전

```rust
if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
    let cell = &table.cells[0];
    let has_visible_text = cell.paragraphs.iter()
        .any(|p| p.text.chars().any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n'));
    if !has_visible_text {
        if let Some(nested) = cell.paragraphs.iter()
            .flat_map(|p| p.controls.iter())
            .find_map(|c| if let Control::Table(t) = c { Some(t.as_ref()) } else { None })
        {
            return self.layout_table(...);
        }
    }
}
```

### 변경 후

```rust
if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
    let cell = &table.cells[0];
    if cell.paragraphs.len() == 1 {
        let p = &cell.paragraphs[0];
        let has_visible_text = p.text.chars()
            .any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n');
        let only_one_nested_table = p.controls.len() == 1
            && matches!(p.controls.first(), Some(Control::Table(_)));
        if !has_visible_text && only_one_nested_table {
            if let Some(Control::Table(t)) = p.controls.first() {
                return self.layout_table(...);
            }
        }
    }
}
```

핵심: unwrap 조건을 다음 4가지 모두 충족하는 경우로 좁힘.
1. 외부 표 1×1 단일 셀 (현행 유지)
2. **셀 paragraphs 가 정확히 1개**
3. **그 paragraph 의 control 이 정확히 1개의 nested table 만**
4. visible text 없음 (현행 유지)

## 검증 결과

### 정량 비교

| 항목 | 수정 전 | 수정 후 | 변화 |
|------|---------|---------|------|
| 페이지 5 SVG 크기 | 20KB | **132KB** | ×6.6 |
| 페이지 5 `<text>` 요소 수 | 55 | **343** | ×6.2 |
| LAYOUT_OVERFLOW (page 0) | 4.1px | **없음** | 해소 |
| 페이지 1~4 SVG 크기 | 450KB/245KB/213KB/296KB | 450KB/244KB/212KB/295KB | ±1% (회귀 없음) |

### 페이지 5 콘텐츠 복원 확인

수정 후 SVG 페이지 5에 다음 텍스트 모두 존재 (`grep -oE ">[가-힣...]+<"` 검증):

- 참고 / 정부혁신 비전 및 추진전략
- 국민이 주도하고 AI가 뒷받침하는 국민주권정부 (빨간 박스)
- **정부혁신 4대 추진전략, 12대 추진과제** (파란 헤더)
- **1 참여소통** / 국민 주도 참여·소통 거버넌스 구현
  - ① 대국민 소통 일상화로 국민의 의견을 정책으로 전환
  - ② 국민의 알권리 보장을 통해 정부 투명성 강화
  - ③ 국민의 목소리를 경청하여 생활 속 문제해결
- **2 기본사회** / 포용과 균형의 기본사회 구현
  - ④ 사각지대 없는 포용적 공공서비스 체계 구축
  - ⑤ 모두가 기본적 삶을 보장받는 사회 안전망 구축
  - ⑥ 자율과 혁신을 통한 지역균형성장 체계 구축
- **3 공직혁신** / 성과로 신뢰받는 일 잘하는 정부 구현
  - ⑦ 국민 누구에게나 쉽고 편리한 서비스 환경 조성
  - ⑧ 가짜 노동 없는 성과 중심의 조직 운영
  - ⑨ 열정이 넘치고 일 잘하는 공직사회 조성
- **4 공공 AX** / 공공부문 인공지능 대전환
  - 공공부문 AI 대전환을 위한 인프라 구축
  - 공공 AI 리터러시 강화 및 인재 양성
  - 공공 AI 윤리 및 신뢰성 확보

### DoD 충족 현황

| DoD | 항목 | 상태 |
|-----|------|------|
| 1 | pi=34 외부 표 외곽 그려짐 | ⚠️ 외부 표 BoundingBox 자체는 그려지나 height=57.72px (권위 778.8px) — 추가 조사 필요 |
| 2 | nested 1×1 헤더 박스 텍스트 | ✅ |
| 3 | nested 11×3 그리드 4그룹 + 12 추진과제 텍스트 | ✅ |
| 4 | PDF 권위본과 시각 정합 | ⏳ 시각 검증 필요 (단계 2 또는 3에서) |
| 5 | 회귀 없음 | ⏳ 단계 2에서 검증 |
| 6 | `cargo test` 통과 | ⏳ 단계 2에서 검증 |

### DoD 1 의 잔여 의문

debug overlay 의 pi=34 외부 표 BoundingBox 가 height=57.72 로 수정 후에도 변하지 않았다. 단, nested 11×3 그리드의 cell 들이 외부 표 영역 밖(y=287 ~ 850)까지 자유롭게 그려져 있어 콘텐츠는 모두 가시화됨. PDF 권위본은 외부 박스 외곽선이 nested 11×3 외곽선과 거의 일치하므로 시각적 차이가 무시할 수 있는 수준일 가능성이 있으나, 단계 3 시각 검증에서 최종 확인.

### 부산 효과: LAYOUT_OVERFLOW 자연 해소

페이지 1 마지막 문단 4.1px 오버플로우 경고가 사라졌다. 본 수정과 같은 원인(부당한 1×1 unwrap)이 페이지 1 의 어느 표에도 적용되어 위치가 4.1px 어긋났던 것이 자연 해소된 것으로 추정. 단계 3 보조 관찰에서 정밀 측정.

## 산출물

- 코드 수정: `src/renderer/layout/table_layout.rs` (+15 / -7 라인)
- 검증 SVG (수정 후): `/tmp/tvpos01_fix/table-vpos-01_005.svg` (132KB)
- 검증 SVG (debug overlay): `/tmp/tvpos01_fix_dbg/table-vpos-01_005.svg`

## 다음 단계

단계 2: 광범위 샘플 회귀 검증 (samples/ 하위 + `cargo test`)
