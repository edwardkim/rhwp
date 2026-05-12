# 구현 계획서 — Task #855

## 대상 이슈

[#855] 21_언어_기출_편집가능본.hwp 14p 우측 단: Square-wrap 표 뒤 문단(pi=300) 렌더링 누락

## 원인 분석 (확정)

`src/renderer/pagination/engine.rs` 의 어울림(Square wrap) 오버랩 처리 (라인 289~323).

- 앵커 문단 `pi=299` 의 표 컨트롤 검출 시 `wrap_around_cs = 3455`, `wrap_around_sw = 27581` 로 설정 (앵커 첫 LINE_SEG 값).
- 다음 문단 `pi=300` 의 LINE_SEG:
  - `ls[0]`: cs=3455, sw=27581 — 표를 피해 들여쓰기된 첫 줄 (표와 같은 y)
  - `ls[1..11]`: cs=852, sw=30184 — 표 아래, 본문 전체 폭
- 현재 판정 `para_cs == wrap_around_cs && para_sw == wrap_around_sw` 는 **첫 LINE_SEG만** 검사 → `pi=300` 전체를 "표 옆에 나란히 배치되는 0-높이 문단"으로 간주 → `continue` 로 높이 소비 없이 `WrapAroundPara` 에만 등록 → `pi=300` 의 줄 12개가 페이지 흐름에서 사라짐.
- 결과: `pi=300` 통째 누락, `단 1` 높이 약 300px 부족.

`WrapAroundPara` 메커니즘은 본래 좁고 긴 어울림 표 **옆 공간을 채우는, 전부 표 옆에 들어가는 문단**(주로 빈 리턴 문단)을 위한 것. `pi=300` 처럼 첫 줄만 표 옆이고 나머지가 표 아래로 흐르는 문단은 일반 텍스트 배치(`paginate_text_lines`)로 처리되어야 함 — LINE_SEG 의 cs/sw 가 이미 wrap 형상을 인코딩하므로 레이아웃은 그대로 정상 렌더.

## 구현 단계

### Stage 1 — 어울림 문단 판정 조건 보정

`engine.rs` 어울림 오버랩 블록(라인 ~304 조건문) 수정:

- "전부 표 옆 문단" 판정을 **모든 (비어있지 않은) LINE_SEG 가 wrap zone(cs/sw) 과 일치**할 때로 한정.
  - 구체: `para.line_segs.iter().all(|s| s.column_start == wrap_around_cs && s.segment_width as i32 == wrap_around_sw)` 를 추가 조건으로 요구 (빈 문단·`sw0_match` 경로는 기존 유지).
  - 혹은 동치로 `para.line_segs.last()` 도 wrap zone 과 일치하는지 검사.
- 일치하지 않으면(= 일부 줄만 표 옆) `continue` 하지 않고 `wrap_around_cs/sw` 리셋 후 일반 텍스트 배치로 폴백 (현 `else` 분기와 동일 동작).
- `wrap_around_any_seg` (어울림 그림 any-seg 경로) 도 동일 원칙 적용 검토 — any-seg 가 true 여도 "전부 일치"가 아니면 폴백.

### Stage 2 — 회귀 검증 (대상 샘플)

- `cargo build --release`
- `rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 13` → `단 1` 에 `pi=300` 항목이 정상 높이로 등장, `pi=301` 위치 정상.
- `rhwp dump-pages -p 14` → 페이지 15 첫 항목 변화 확인.
- `rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 13` → `pi=300` 본문 렌더링 + 표 옆 첫 줄 들여쓰기 확인.
- `pdf/21_언어_기출_편집가능본-2022.pdf` 14페이지와 시각 대조.

### Stage 3 — 전체 회귀 + 마무리

- `cargo test`
- 어울림 표/그림이 있는 다른 샘플 몇 개 SVG 스팟체크 (회귀 없음 확인) — 예: 기존 `re_sample` 및 어울림 관련 테스트.
- `clippy` 통과 확인.
- 최종 보고서 `mydocs/report/task_m05x_855_report.md` 작성.

## 영향 범위

- 수정 파일: `src/renderer/pagination/engine.rs` (어울림 오버랩 판정 1개 블록).
- 레이아웃·typeset 변경 없음.

## 리스크

- "전부 일치" 로 좁힐 때, 기존에 `WrapAroundPara` 로 처리되던 정상 케이스(전부 표 옆 빈 문단)는 모든 seg 가 wrap zone 과 일치하므로 영향 없음. 다만 첫 줄만 표 옆이고 나머지가 본문 폭인 케이스가 다른 샘플에도 있을 수 있어 Stage 3 회귀 확인 필수.

---

승인해 주시면 Stage 1 구현 시작하겠습니다.
