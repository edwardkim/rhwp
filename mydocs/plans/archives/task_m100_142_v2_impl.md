# 구현 계획서 (v2): 수식 레이아웃 보정 + 탭/TAC 위치 보정

- **타스크**: [#142](https://github.com/edwardkim/rhwp/issues/142), [#159](https://github.com/edwardkim/rhwp/issues/159)
- **마일스톤**: M100
- **브랜치**: `local/task142`
- **작성일**: 2026-04-16
- **수행계획서**: `mydocs/plans/task_m100_142_v2.md`

## 단계 구성 (3단계)

---

### 1단계: 원문자 전각 폭 수정 및 텍스트 폭 추정 개선

**수정 파일**: `src/renderer/layout/text_measurement.rs`

**작업 내용**:

1. `is_fullwidth_symbol()` 함수에 전각 기호 범위 추가:
   - U+2460~U+24FF: Enclosed Alphanumerics (①②③ 등)
   - U+3200~U+32FF: Enclosed CJK Letters (㉠㉡ 등)
   - U+3300~U+33FF: CJK Compatibility (㎜㎝ 등)
   - U+2160~U+217F: Roman Numerals (Ⅰ Ⅱ Ⅲ 등)
   - U+2600~U+26FF: Miscellaneous Symbols (☆★ 등)
   - U+2700~U+27BF: Dingbats (✓✗ 등)

2. 탭 `/2.0` FIXME 주석을 정상 주석으로 변경:
   - `style_resolver.rs:618`의 FIXME → HWP 탭 position이 2배로 저장되는 것이 확인됨

**검증**:
- `cargo test` 전체 통과
- exam_math.hwp 2페이지 격자 SVG 출력 → 선택지 ①~⑤ 위치 비교

---

### 2단계: 시각 검증 (exam_math.hwp 전체)

**작업 내용**:

1. 전체 20페이지 SVG 출력 (격자 + 폰트 임베딩)
2. 한컴 격자 스크린샷과 비교
3. 발견된 추가 문제를 버그 목록(`task_m100_142_bugs.md`)에 기록

---

### 3단계: 최종 보고

**산출물**:
- 버그 목록 갱신
- 최종 보고서 작성
- 오늘할일 갱신

---

## 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| 1단계 | `cargo test` 통과, 2페이지 선택지 위치 격자 비교 오차 ≤ 2mm |
| 2단계 | 20페이지 시각 검증 완료, 추가 문제 기록 |
| 3단계 | 최종 보고서 승인 |
