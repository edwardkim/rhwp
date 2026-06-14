# 최종 결과보고서 — Task #1343

## 이슈
수식 조건부 막대 `U+E04D`(PUA)가 두부(▦) 박스로 렌더 (#1343, M100)

`P(A|B)`의 조건부 막대 `|`가 HWP 수식 스크립트에서 PUA 문자 `U+E04D`로 인코딩되어
있어 글리프 미존재로 두부(▦)로 표시되던 문제.

## 원인
수식 토크나이저가 non-ASCII 문자를 `Text` 토큰으로 그대로 보존하는데, 한컴 수식
편집기가 PUA(U+E000~F8FF)에 저장한 조건부 막대 `U+E04D`에 대한 표준 기호 매핑이
부재했다. PUA 코드포인트가 그대로 렌더 단계로 흘러 글리프 미존재 → 두부.

## 해결
수식 도메인 한정으로 PUA→표준기호 매핑을 추가:

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/symbols.rs` | `EQUATION_PUA` 매핑(`U+E04D → "|"`) + `lookup_equation_pua()` |
| `src/renderer/equation/tokenizer.rs` | non-ASCII Text 분기 직전 PUA 조회 → `Symbol` 토큰 + 단위테스트 |

생성된 `Symbol "|"` 토큰은 기존 단일 `|`(442행)와 동일하여 파서/레이아웃 경로를
공유한다. 매핑이 없는 PUA 문자는 기존 `Text` 폴백을 유지하여 회귀를 차단한다.

샘플 전수 조사 결과 수식 스크립트에서 사용되는 PUA는 `U+E04D` 1종뿐이었다.

## 검증
- `cargo test --lib`: 1617 passed, 0 failed (단위테스트 `test_pua_conditional_bar` 포함)
- `cargo clippy`: 경고 없음
- 시각: 16페이지 문24 `P(A|B)`, `P(B|A)` 조건부 막대 정상 `|` 렌더, 두부 소거
  (SVG 내 `U+E04D` 잔존 0건). 한글 2022 PDF 권위 자료와 시각 정합 확인.

## 범위/영향
- 수식 렌더링 전용 변경. 공통 렌더러/레이아웃/문서 코어 무수정.
- `EQUATION_PUA` 테이블은 향후 다른 HWP 수식 PUA 기호 발견 시 확장 가능.

## 후속
- 추가 PUA 수식 기호 발견 시 `EQUATION_PUA`에 항목 추가만으로 대응.
