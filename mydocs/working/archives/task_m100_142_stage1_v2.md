# 1단계 완료보고서 (v2): 원문자 전각 폭 + 탭 inline_tabs 동기화

- **타스크**: [#142](https://github.com/edwardkim/rhwp/issues/142), [#159](https://github.com/edwardkim/rhwp/issues/159)
- **마일스톤**: M100
- **브랜치**: `local/task142`
- **작성일**: 2026-04-16

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/layout/text_measurement.rs` | `estimate_text_width`에 `inline_tabs` 분기 추가, `is_fullwidth_symbol` 원문자 범위 추가 |
| `src/renderer/layout/paragraph_layout.rs` | 정렬 계산(est_x) `ts`에 `inline_tabs` 설정 추가 |
| `src/renderer/style_resolver.rs` | 탭 `/2.0` FIXME → 정상 주석 |

## 해결된 문제

- ③ 위에 수식 `0` 겹침 → **해소**
- 근본 원인: `estimate_text_width`와 `compute_char_positions`의 탭 계산 불일치 (inline_tabs 누락)
- ③→④ 간격: 13.8mm → 18.3mm (한컴 17mm 대비 +1.3mm)

## 검증 결과

- `cargo test` 788개 전체 통과
- exam_math.hwp 2페이지 격자 SVG 시각 비교 완료

## 다음 단계

2단계: exam_math.hwp 전체 20페이지 시각 검증
