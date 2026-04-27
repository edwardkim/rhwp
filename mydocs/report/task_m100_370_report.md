# Task #370 최종 결과 보고서

## 제목
한글 세리프 폴백 체인에 Noto Serif CJK KR 추가

## 배경
`form-002.hwpx` 10쪽의 (기술적 측면), 연구개발기간, 정부지원연구개발비, 주관연구개발기관, 정부납부기술료 등 볼드 지정 문구가 SVG 렌더링 시 볼드로 표시되지 않는 현상 보고.

## 분석 (요약)
- SVG 출력은 정상 — `font-weight="bold"`가 정확히 부여.
- 원인: 한글 세리프 폴백 체인(`함초롬바탕,'Batang','바탕','AppleMyungjo','Noto Serif KR',serif`)의 모든 패밀리가 표준 리눅스 환경에 미설치. 시스템에 설치된 `Noto Serif CJK KR`(Bold 포함)이 체인에 누락되어 generic `serif` → 자동 한글 폴백 경로로 떨어지면서 일부 SVG/PDF 렌더러에서 bold variant 매칭 실패.

## 변경 내용

| 파일 | 변경 |
|------|------|
| `src/renderer/mod.rs` | 한글 세리프 폴백 체인 두 분기(`:558`, `:565`)에 `'Noto Serif CJK KR'` 추가 (`'Noto Serif KR'` 직후, `serif` 직전) |
| `src/renderer/mod.rs` 테스트 | `test_generic_fallback`의 기대 문자열 갱신 |
| `tests/golden_svg/{form-002/page-0, issue-157/page-1, issue-267/ktx-toc-page}.svg` | 폰트 체인 변경에 따른 골든 갱신 |

## 검증 결과

- `cargo test`: **1055 passed, 0 failed**
- `rhwp export-svg samples/hwpx/form-002.hwpx -p 9` 출력 SVG에 `Noto Serif CJK KR` 936회 적용 확인.
- 시스템 설치 패밀리 매칭으로 한글 볼드 표시 문제 해소.

## 영향 범위

- HTML 렌더러도 `generic_fallback` 공유 → 동일하게 개선됨.
- PDF 렌더러는 별도 경로(`pdf.rs:add_font_fallbacks`) 사용 → 본 변경 영향 없음.
- 기존 환경(Windows: Batang / macOS: AppleMyungjo)에서 우선순위 변경 없음 — `Noto Serif CJK KR`은 후순위 추가이므로 기존 매칭 결과 보존.

## 후속 과제 (선택)

- Sans-serif 체인(`mod.rs:542,568`)에도 `Noto Sans CJK KR` 추가 검토 — 동일한 누락 위험. 본 타스크 범위 외(이슈 #370 한정).
- `cargo clippy`의 기존 회귀 44건은 별도 타스크로 처리.
