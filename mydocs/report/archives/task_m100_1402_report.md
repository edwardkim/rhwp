# Task M100 #1402 최종 보고서 — 열거 속성 표면 표기 정합 검사

- 이슈: #1402 "HWPX serializer: 열거 속성 표면 표기 정합 검사 — 파서 관용 수용에 가려진 비실재 토큰 검출"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1402`
- 작성일: 2026-06-14

## 1. 본질 — 방어 검사 인프라 구축

#1387에서 numType "FIGURE"(한컴 비실재 표기 → 캡션 번호 미출력)를 정정. 근본
구조는 "파서 관용 수용 + 게이트가 표면 표기 미비교"라 비실재 토큰이 가려진다.
본 타스크는 **재발 방지 검사 인프라**를 구축한다 (단일 결함 수정 아님 — 전수 측정상
명백한 신규 비실재 토큰 0건).

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 화이트리스트 검사 인프라 + numType FIGURE 가드 + 측정 | `23cf3a86` |
| 2 | baseline·CI + 매뉴얼 + 최종 보고서 | (본 커밋) |

수정 파일: `tests/issue_1402_enum_token_whitelist.rs`(신규) — 소스 무변경(검사만).

## 3. 검사 인프라

`tests/issue_1402_enum_token_whitelist.rs`:

| 테스트 | 역할 |
|--------|------|
| `serializer_enum_tokens_within_whitelist` | samples/hwpx 전수 parse→serialize 후 방출 XML 의 열거 토큰이 화이트리스트(17속성) 안인지 검증. **밖 토큰 0건** |
| `numtype_figure_regression_guard` | numType="FIGURE" 방출 금지 (#1387 봉인). **방출 0건** |

화이트리스트 17속성: numType/numberingType/pageBreak/textWrap/textFlow/vertRelTo/
horzRelTo/vertAlign/horzAlign/applyPageType/lineWrap/familyType/gutterType/side/
circleType/composeType. 근거: 실물 관측(`output/poc/task1402/observed_tokens.txt`)
∪ owpml/한컴 스펙.

## 4. circleType TIRANGLE (현행 유지 + 기록)

`SHAPE_REVERSAL_TIRANGLE`(오타 의심) — 파서·serializer 양쪽 동일 철자라 roundtrip
정합, 실물 corpus 에 사례 없어 한컴 실제 철자 미확인. owpml 권위 자료 로컬 미접근 →
**추정 금지 룰**에 따라 현행 유지 + 화이트리스트 명시 + 미확인 기록. owpml 확인
가능 시 정정 후보 (정정 시 파서·serializer 동시 — 동일 철자 보장).

## 5. 검증

- 두 검사 테스트 **통과** — 방출 토큰 화이트리스트 밖 0건, numType FIGURE 0건.
- `cargo test --test hwpx_roundtrip_baseline` — B=0 유지.
- CI급: `cargo test --profile release-test --tests` 전체 그린 (수치 커밋 시점 기재),
  fmt 통과, clippy 0.

## 6. 매뉴얼 — 화이트리스트 갱신 절차 문서화

`hwpx_roundtrip_baseline.md` 6.5절: 새 열거 속성/토큰 방출 시 한컴 실물 또는 owpml
스펙으로 확인(추정 금지) 후 `whitelist()` 추가. circleType TIRANGLE 미확인 기록.

## 7. 잔존 한계 (기지 이슈)

| 한계 | 이슈 |
|------|------|
| newNum 슬롯 위치 + 143E RT 페이지 수 | #1407 |
| numbering 등록 축 잠재 불일치 | #1409 |
| circleType TIRANGLE owpml 철자 미확인 | 본 보고서 4절 (잠재, roundtrip 정합) |

## 8. 산출물

- 계획서: `mydocs/plans/task_m100_1402{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1402_stage1.md`
- 검사 인프라: `tests/issue_1402_enum_token_whitelist.rs`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (6.5절)
- 검증 산출물: `output/poc/task1402/observed_tokens.txt`
