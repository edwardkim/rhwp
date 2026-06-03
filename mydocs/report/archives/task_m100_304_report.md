# Task #304 최종 결과보고서

## 개요

`samples/21_언어_기출_편집가능본.hwp` 의 SVG 1쪽이 PDF와 불일치하던 문제를 해결. 표면적으로는 "바탕쪽 글상자 중복 렌더 + 우측 단 누락" 두 증상이었으나, 실제 원인은 **`SectionDef.flags`의 `hide_master_page` 비트 오프셋 오류** 단일 버그.

## 증상 → 원인 → 수정

### 증상 (1쪽만)
- "언어이해" 제목·"홀수형" 박스·페이지번호가 바탕쪽과 body 표에서 이중 렌더 → 글자 깨짐처럼 보임
- 우측 단(2단 조판) 본문 누락

### 원인
HWP5 구역정의 속성에서 "바탕쪽 감춤"(첫쪽 바탕쪽 숨김) 비트는 **bit 2 (0x0004)** 이나, 기존 파서는 bit 10 (0x0400)으로 읽어 `hide_master_page` 가 false로 오판정. 이로 인해 1쪽에도 바탕쪽이 그려져 body 표와 충돌.

우측 단 누락은 바탕쪽 오버레이가 우측 단을 가린 파생 증상.

### 수정
2개 파일 2줄 변경:

| 파일 | 변경 |
|------|------|
| `src/parser/body_text.rs:549` | `flags & 0x0400` → `flags & 0x0004` |
| `src/document_core/queries/rendering.rs:166` | `set_bit(flags, 0x0400, …)` → `set_bit(flags, 0x0004, …)` |

## 검증 결과

### 테스트
- `cargo test --release`: 1038 passed, 0 failed
- `cargo clippy --release -- -D warnings`: 경고 없음

### 시각 회귀
| 샘플 | bit 2 | 결과 |
|------|-------|------|
| 21_언어_기출_편집가능본.hwp | SET | 1쪽 PDF 일치 ✓ · 2쪽+ 정상 ✓ |
| exam_kor.hwp | SET | 1쪽 바탕쪽 중복 해소 ✓ |
| exam_eng.hwp | SET | 1쪽 바탕쪽 중복 해소 ✓ |
| exam_math.hwp | unset | 변화 없음 (회귀 없음) ✓ |

## 부가 성과

- exam_kor / exam_eng 에서도 잠재 바탕쪽 중복 버그가 있었는데 동일 수정으로 함께 해소.

## 한계 · 향후 과제

`SectionDef.flags`의 나머지 hide 비트들(header·footer·border·fill·page_num)도 HWP5 스펙(bit 0,1,3,4,5)과 현 코드(bit 8,9,11,12) 오프셋이 다를 가능성이 있다. 현재 샘플에서 증상이 관측된 것은 `hide_master_page` 한 건뿐이므로 본 이슈에서는 범위 제한. 향후 별도 이슈로 스펙 전수 대조 · 수정 권장.

## 변경 파일

- `src/parser/body_text.rs`
- `src/document_core/queries/rendering.rs`
- `mydocs/plans/task_m100_304.md` (수행계획서)
- `mydocs/plans/task_m100_304_impl.md` (구현계획서)
- `mydocs/working/task_m100_304_stage1.md` (단계 보고)
- `mydocs/report/task_m100_304_report.md` (본 보고서)

## 관련 이슈

- #304 (본건)
- #295 exam_math 12쪽 좌단 붕괴 — 다단+표 wrap 관련 (구조 다름)
