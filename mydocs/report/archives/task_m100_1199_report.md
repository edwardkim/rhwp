# 최종 결과보고서 — Task M100 #1199

**이슈**: [#1199](https://github.com/edwardkim/rhwp/issues/1199) HWPX 미주/각주 마커 접두문자(prefixChar) 미파싱 — '문N)'가 'N)'로 렌더링
**마일스톤**: v1.0.0 (M100)
**브랜치**: `local/task1199` (← `local/devel` ← `stream/devel`)
**완료일**: 2026-06-01

---

## 1. 문제

`samples/3-09월_교육_통합_2022.hwpx` 9~10쪽 미주 마커 "문N)" 가 "N)" 로 렌더링(접두 '문' 탈락).

## 2. 근본 원인

HWPX `<hp:endNote>` / `<hp:footNote>` 요소는 접두/접미문자를 코드포인트 숫자 속성으로 인코딩:
- `prefixChar="47928"`(0xBB38 '문'), `suffixChar="65289"`(0xFF09 '）').

파서 `parse_ctrl_endnote()` / `parse_ctrl_footnote()` (`src/parser/hwpx/section.rs`)가 **`suffixChar`만 읽고 `prefixChar` 분기가 없어** `before_decoration_letter`가 0으로 남음 → `note_decoration_char(0)` = `None` → 접두 탈락. 렌더 경로(`format_endnote_marker_text`, typeset.rs:60)는 이미 접두를 지원하므로 **파서 단일 원인**.

## 3. 수정

`src/parser/hwpx/section.rs` 두 함수에 `suffixChar`와 대칭으로 `b"prefixChar"` 분기 추가 → u16 파싱하여 `before_decoration_letter` 설정. (118줄 추가, 삭제 0 — 분기 2개 + 테스트 2개)

- 모델/렌더러/HWP3/공통 모듈 무변경.

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 성공 |
| `cargo test --release` (전체) | 통과, 회귀 0 |
| 신규 회귀 테스트 2건 | 통과 |
| SVG 렌더 (9·10쪽) | `문1）`~`문13）` 접두 복원 |
| 한글 2022 PDF 9쪽 대조 | `문1)`~`문7)` 정합 (1차 정답지) |
| rustfmt (변경 파일) | clean |

## 5. 범위 외 / 후속

- **#1200** "다른 풀이" 미표시 — 미주 2단(EACH_COLUMN) 배치/오버플로 추정, #1184 영역. 별도 진단.

## 6. 산출물

- 소스: `src/parser/hwpx/section.rs`
- 계획: `mydocs/plans/task_m100_1199.md`, `mydocs/plans/task_m100_1199_impl.md`
- 단계 보고: `mydocs/working/task_m100_1199_stage{1,2,3}.md`
- 최종 보고: 본 문서
