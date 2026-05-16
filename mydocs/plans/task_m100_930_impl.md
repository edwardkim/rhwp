# Task #930 구현 계획서

## 대상 코드 분석

### 결함 위치

`src/renderer/layout/shape_layout.rs` `layout_textbox_content()` (1163줄~)

```rust
// 1196~1227줄 (#874, commit a466e2ea)
let sa = &drawing.shape_attr;
let local_styles_scaled: Option<ResolvedStyleSet> = {
    let sw_ratio = sa.current_width / sa.original_width;   // table-in-tbox: 1.068
    let sh_ratio = sa.current_height / sa.original_height; // table-in-tbox: 8.197
    let max_ratio = sw_ratio.max(sh_ratio);                // 8.197
    if max_ratio > 1.5 {
        let inv = (1.0 / max_ratio).min(1.0);              // 0.122
        let mut local = styles.clone();
        for cs in local.char_styles.iter_mut() {
            cs.font_size *= inv;
            cs.letter_spacing *= inv;
            for ls in cs.letter_spacings.iter_mut() { *ls *= inv; }
        }
        Some(local)
    } else { None }
};
let styles: &ResolvedStyleSet = local_styles_scaled.as_ref().unwrap_or(styles);
```

`max_ratio > 1.5` 조건만으로 글꼴을 축소하므로, 이미 최종 박스 크기 기준으로 조판된 일반 글상자 본문(`table-in-tbox.hwp`)에도 잘못 발동한다.

### 두 케이스 비교

| 케이스 | 글상자 성격 | 기대 동작 |
|--------|------------|----------|
| `shortcut.hwp` 1p | 마스터 페이지 자동번호("1") 단독 글상자, 거대 글꼴(254pt) | 글꼴 축소 **유지** |
| `table-in-tbox.hwp` 2p | 본문 21문단, current 박스 기준 LINE_SEG 산출 완료 | 글꼴 축소 **제외** |

---

## 범위 개정 (작업지시자 결정)

Stage 1 측정 중 작업지시자 피드백("shortcut.hwp도 조금 작은 것 같아")으로 추가 결함이 확인되었다. `shortcut.hwp` 자동번호 "1"이 PDF 대비 약 1/2 크기로 과소 렌더된다. #874 축소 계수(`1/max_ratio`)가 과도하다.

작업지시자 결정으로 **#930 범위를 확장**하여 두 결함을 함께 수정한다.

- 결함 A (table-in-tbox 2p): 이방 스케일 글상자에 글꼴 축소가 **잘못 발동** → 판별자로 차단
- 결함 B (shortcut 1p): 등방 스케일 글상자에 글꼴 축소 **계수가 과도** → 계수 재유도

## 구현 단계 (4단계)

### 1단계: 판별자 측정 및 확정 — ✅ 완료

(완료. `mydocs/working/task_m100_930_stage1.md` 참조.)

- 두 샘플의 도형 스케일·LINE_SEG·글꼴 크기 측정
- 판별자 **(B) `min(sw_ratio, sh_ratio) > 1.5`** 확정 (등방 확대일 때만 글꼴 축소)
- 추가 결함 B(shortcut 자동번호 과소 렌더) 발견 및 측정

---

### 2단계: shortcut 자동번호 축소 계수 재유도 (코드 수정 없음, 조사 전용)

**작업 내용:**

1. `shortcut.hwp` 자동번호 글상자의 축소 의미를 재조사
   - 자동번호 char 글꼴(254pt) · 도형 matrix(sx 2.68, sy 2.51) · 박스 크기 관계 분석
   - HWP 스펙/내부 참조 자료로 글상자 matrix 스케일의 글꼴 적용 규칙 확인 (참조 자료 인용 금지, 이해 목적만)
2. PDF(한글 2022) `pdf/basic/shortcut-2022.pdf` 자동번호 글리프 측정값과 정합하는 축소 계수 공식 확정
   - 도형 스케일에서 유도 가능한 공식이 있으면 그것을 채택
   - 깔끔한 공식이 없으면, 측정 기반의 문서화된 보정 계수로 확정 (근거·한계 명시)
3. 결정한 공식이 `table-in-tbox.hwp` 등 다른 글상자에 회귀를 일으키지 않는지 교차 점검

**산출물:** `mydocs/working/task_m100_930_stage2.md` — 축소 계수 재유도 근거 + 확정 공식

---

### 3단계: 글꼴 축소 로직 정정 구현

**대상 파일:** `src/renderer/layout/shape_layout.rs`

**작업 내용:**

1. `layout_textbox_content()` 글꼴 축소 발동 조건을 판별자 (B)로 정정 — `min(sw_ratio, sh_ratio) > 1.5`일 때만 발동 (결함 A 해소)
2. 2단계에서 확정한 축소 계수 공식으로 `inv` 계산 정정 (결함 B 해소)
3. #874 의도가 보존되도록 주석 갱신 — 발동 조건·계수 변경 사유와 두 케이스 명시
4. 세로쓰기 경로 `layout_vertical_textbox_text_with_paras()`가 동일 `styles`를 받는지 확인, 일관성 점검
5. 임시 디버그 출력(`eprintln!`) 제거

**산출물:** 소스 커밋 + `mydocs/working/task_m100_930_stage3.md`

---

### 4단계: 시각 정합 검증 및 회귀 차단

**작업 내용:**

1. `rhwp export-svg samples/table-in-tbox.hwp` → 2페이지 글상자 본문 글꼴 정상 크기 확인, PDF 2페이지와 시각 정합 비교
2. `rhwp export-svg samples/basic/shortcut.hwp` → 1페이지 자동번호 "1" 글리프 높이가 PDF 정합(≈187px)인지 확인
3. `cargo test --release --lib` 실행 — 회귀 0 확인
4. 글상자 내부 "검사항목" 표 가로 흩어짐 잔존 여부 확인 → 잔존 시 후속 이슈 분리 기록
5. 최종 결과보고서 작성

**산출물:** `mydocs/report/task_m100_930_report.md` + 소스/문서 커밋

---

## 승인 기준

- `table-in-tbox.hwp` 2페이지 글상자 본문 글꼴 정상 크기 렌더링 + PDF 정합
- `shortcut.hwp` 1페이지 자동번호 "1" 글리프가 PDF(한글 2022) 정합
- `cargo test --release --lib` 회귀 0
- 임시 디버그 출력(`eprintln!`) 잔존 없음
