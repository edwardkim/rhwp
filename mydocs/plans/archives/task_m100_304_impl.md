# Task #304: 구현 계획서

## 원인 결론 (Stage 1 통합 결과)

`SectionDef` flags의 `hide_master_page` 비트 오프셋이 HWP5 스펙과 불일치.

- **HWP5 스펙**: bit 2 (0x0004)
- **기존 파서**: bit 10 (0x0400) ← 오류

### 영향
- 21_언어_기출_편집가능본.hwp: flags=`0xC0080004` (bit 2 SET) → 기존 코드는 `hide_master_page=false`로 오판정 → 1쪽에 바탕쪽 글상자("언어이해"·"홀수형"·페이지번호·세로선)가 body 표 header와 겹쳐 중복 렌더. 이 겹침 때문에 우측 단(column) 본문이 가려져 보이지 않았음.
- 동일 증상 샘플: exam_kor.hwp, exam_eng.hwp (flags=`0xC0000004`)

### 이슈 B(우측 단 누락)는 이슈 A의 파생 증상
바탕쪽 글상자 렌더 시 body 우측 단 위에 배경/테두리가 덮여 본문을 가림. 바탕쪽을 1쪽에서 숨기자 우측 단이 정상 표시됨. 별도 수정 불필요.

## 수정 범위

| 파일 | 라인 | 변경 |
|------|------|------|
| `src/parser/body_text.rs` | 549 | `flags & 0x0400` → `flags & 0x0004` (읽기) |
| `src/document_core/queries/rendering.rs` | 166 | `set_bit(flags, 0x0400, …)` → `set_bit(flags, 0x0004, …)` (쓰기) |

## 구현 단계

### 단계 1: 파서/쓰기 경로 비트 오프셋 수정 ✓ (완료)
- `src/parser/body_text.rs:549`: bit 10 → bit 2
- `src/document_core/queries/rendering.rs:166`: bit 10 → bit 2
- 주석에 "HWP5 스펙, 첫쪽 바탕쪽 감춤" 명시

### 단계 2: 회귀 검증 ✓ (완료)
- `cargo test --release`: 1038 테스트 통과 (0 실패)
- `cargo clippy --release -- -D warnings`: 경고 없음
- 시각 회귀 검증:
  - 21_언어 1쪽: PDF와 일치 ✓
  - 21_언어 2쪽+: 바탕쪽 정상 렌더 ✓
  - exam_math 1쪽 (bit 2 미설정): 변화 없음 ✓
  - exam_kor 1쪽 (bit 2 설정): 바탕쪽 중복 해소 ✓
  - exam_eng 1쪽 (bit 2 설정): 바탕쪽 중복 해소 ✓

### 단계 3: 단계별 완료보고서 + 최종 보고서 작성
- `mydocs/working/task_m100_304_stage1.md` — 단계 1·2 통합 보고
- `mydocs/report/task_m100_304_report.md` — 최종 결과

### 단계 4: 브랜치 merge (승인 후)
- `task304` → `local/devel` (local merge)
- `local/devel` → `devel` (local merge + push)

## 범위 제한 사항

- **다른 hide 비트(header/footer/border/fill/page_num)의 오프셋은 건드리지 않음**
  - 해당 비트들도 HWP5 스펙과 어긋났을 가능성 있음 (현 코드는 bit 8-12 사용, 스펙상 bit 0-5)
  - 그러나 21_언어 샘플에서 문제 증상이 확인된 것은 `hide_master_page`뿐
  - 다른 비트도 바꾸면 회귀 위험이 커짐 → 별도 이슈로 처리 권장 (향후 작업)

## 커밋 구성

한 커밋으로 통합 (소형 변경):
- 파서 + 쓰기 경로 수정
- 단계 보고서 + 최종 보고서 + orders 갱신 포함

## 완료 조건

- 21_언어 1쪽 SVG가 PDF와 시각 일치 ✓
- 전체 테스트 + clippy 통과 ✓
- exam_kor / exam_eng 동일 증상도 개선 ✓
- 회귀 없음 ✓
