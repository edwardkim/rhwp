# Task #304 단계1 완료 보고서: SectionDef `hide_master_page` 비트 오프셋 수정

## 수행 내용

HWP5 스펙상 구역정의 속성(`SectionDef.flags`)의 "바탕쪽 감춤"(첫쪽 바탕쪽 숨김) 비트가 **bit 2 (0x0004)** 인데, 기존 파서는 **bit 10 (0x0400)** 으로 잘못 읽고 있었다. 이를 수정.

## 원인 분석 (덤프 기반)

### 증상
`samples/21_언어_기출_편집가능본.hwp`의 SVG 출력 1쪽이 PDF 원본과 크게 달랐다.
- 바탕쪽(Odd)의 "언어이해" 제목 글상자·"홀수형" 글상자·페이지번호·세로선이 body 표(header 4×5 표)와 겹쳐 2회 렌더됨
- 우측 단(column) 본문이 보이지 않음 (바탕쪽 오버레이가 덮고 있었음)
- 2쪽 이후는 정상

### 원인 특정
1. `dump -s 0 -p 0`으로 SectionDef flags=`0xC0080004` 확인 (bit 2 SET)
2. 현 코드 `src/parser/body_text.rs:549`: `sd.hide_master_page = sd.flags & 0x0400 != 0;` → bit 10 기준 → 오판정 (false)
3. 렌더링 로직(`rendering.rs:955`)은 "`hide_master_page=true`면 첫쪽에 바탕쪽 미적용"으로 동작 → 1쪽에도 바탕쪽 그림
4. HWP5 스펙 대조: 구역정의 속성 bit 0~5가 hide 플래그들(header/footer/master_page/border/fill/page_num)
5. bit 2 → 0x0004 기준으로 수정 시 `hide_master_page=true` → 1쪽 스킵 → PDF와 일치

### 이슈 B(우측 단 누락)는 파생 증상
바탕쪽 중복 렌더가 우측 단 위를 덮어 본문을 가렸던 것. 바탕쪽을 1쪽에서 숨기니 우측 단이 정상 표시됨. 별도 수정 불필요.

## 변경 파일

| 파일 | 라인 | 변경 |
|------|------|------|
| `src/parser/body_text.rs` | 549 | `flags & 0x0400` → `flags & 0x0004` (읽기) |
| `src/document_core/queries/rendering.rs` | 166 | `set_bit(flags, 0x0400, …)` → `set_bit(flags, 0x0004, …)` (쓰기) |

## 검증

### 단위 테스트
- `cargo test --release`: **1038 테스트 통과, 0 실패**
- `cargo clippy --release -- -D warnings`: 경고 없음

### 시각 회귀 검증
샘플별 flags 값 + 동작 변화:

| 샘플 | flags | bit 2 | 수정 전 | 수정 후 |
|------|-------|-------|---------|---------|
| 21_언어_기출_편집가능본.hwp | `0xC0080004` | SET | 1쪽 바탕쪽 중복 | 1쪽 PDF와 일치 ✓ |
| exam_kor.hwp | `0xC0000004` | SET | 1쪽 바탕쪽 중복 | 해소 ✓ |
| exam_eng.hwp | `0xC0000004` | SET | 1쪽 바탕쪽 중복 | 해소 ✓ |
| exam_math.hwp | `0x20000000` | unset | 정상 | 변화 없음 ✓ |

2쪽 이후 바탕쪽 렌더는 모든 샘플에서 정상 (회귀 없음).

## 발견 사항

- 동일 원인으로 exam_kor/exam_eng도 잠재 버그가 있었음. 시각 제보 전이었지만 이번 수정으로 같이 해소.
- 다른 hide 비트(header/footer/border/fill/page_num)도 HWP5 스펙(bit 0,1,3,4,5)과 현 코드(bit 8,9,11,12) 오프셋이 다를 가능성 있음. 그러나 현재 샘플에서 실증상 없음 → 범위에서 제외 (별도 이슈로 분리 권장).

## 범위 제한

본 수정은 `hide_master_page` 한 비트만 다룬다. 다른 5개 hide 비트의 오프셋 재정렬은 별도 조사·이슈 필요.
