# Task M100 #1402 — 1단계 완료 보고서 (검사 인프라 + 화이트리스트)

- 브랜치: `local/task1402`
- 작성일: 2026-06-14
- 산출물: `tests/issue_1402_enum_token_whitelist.rs`, `output/poc/task1402/observed_tokens.txt`

## 1. 검사 인프라

`tests/issue_1402_enum_token_whitelist.rs` — samples/hwpx 전수를 parse→serialize 한
뒤 방출 XML 에서 열거 속성 토큰을 추출, 화이트리스트와 대조.

| 테스트 | 역할 |
|--------|------|
| `serializer_enum_tokens_within_whitelist` | 전수 방출 토큰 ∈ 화이트리스트 검증. 밖 토큰 0건 |
| `numtype_figure_regression_guard` | numType="FIGURE" 방출 금지 (#1387 봉인) |

화이트리스트(`whitelist()`): 17개 속성 — numType/numberingType/pageBreak/textWrap/
textFlow/vertRelTo/horzRelTo/vertAlign/horzAlign/applyPageType/lineWrap/familyType/
gutterType/side/circleType/composeType. 근거: 실물 관측(observed_tokens.txt) ∪
owpml/한컴 스펙 표준값.

## 2. 검증 결과

- 두 테스트 **통과** — 현재 serializer 방출 토큰은 화이트리스트 밖 0건, numType
  FIGURE 방출 0건. #1387 정정 후 명백한 비실재 토큰 잔존 없음 재확인.
- 본 타스크는 **재발 방지 봉인** — 미래에 비실재 토큰을 방출하도록 바뀌면 즉시 실패.

## 3. circleType TIRANGLE 판정 (현행 유지 + 기록)

- `SHAPE_REVERSAL_TIRANGLE`(오타 TIRANGLE) — 같은 함수의 `SHAPE_TRIANGLE`(정상)과
  대비되어 오타 의심.
- **파서·serializer 양쪽 동일 철자**(parser/hwpx/section.rs:4734, serializer
  section.rs:798)라 roundtrip 정합. 실물 corpus 에 triangle circleType 사례 없어
  한컴 실제 철자 미확인.
- owpml(hancom-io/hwpx-owpml-model) 권위 자료 로컬 미접근 → **추정 금지 룰**에 따라
  현행 유지 + 화이트리스트에 TIRANGLE 명시 + 사유 주석. owpml 확인 가능 시 정정 후보.

## 4. 정정 (신규 0건)

명백한 비실재 토큰 신규 발견 0건. numType FIGURE는 #1387 정정 완료, 검사로 봉인.

## 5. 다음 단계

2단계 — baseline·CI(release-test) + 매뉴얼(검사 인프라·화이트리스트 갱신 절차) +
최종 보고서.

승인 요청드립니다.
