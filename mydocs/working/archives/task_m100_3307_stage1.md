---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3307.md
last_verified: 2026-08-01
---

# Task #3307 Stage 1 보고 — 기본 개요 모양 권위 확정

## 결론 — 한컴 2020 직접 실측으로 확정

정의 없는 개요(idRef=0, `<hh:numbering>` 부재)에 한컴 2020 이 적용하는 기본 모양은
**전 수준 `^N`(레벨 경로 + 후행 마침표, 아라비아 숫자, 시작 1)** 이다.

| 수준(0-based) | 한컴 실측 | 형식 해석 |
|---|---|---|
| 0 | `1.` `2.` `3.` `4.` | `^N` (경로 = 자기 카운터) |
| 1 | `2.1.` | `^N` |
| 2 | `2.2.1.` | `^N` |
| 3 | `2.2.2.…` 계열 | `^N` |
| 4 | `2.5.1.1.1.` | `^N` (5성분) |
| 5 | `2.5.1.1.1.1.` / `…2.` | `^N` (6성분) |
| 6 | `2.5.1.1.1.2.1.` | `^N` (7성분) |

**rhwp 는 `^N` 확장을 이미 구현하고 있다** (`layout/utils.rs`
`expand_numbering_format` — "^N: 레벨 경로 + 후행 마침표"). 따라서 수정은 형식
엔진 추가가 아니라 **합성 기본 Numbering 의 fallback 배선**만 필요하다.

## 실측 방법 (권위 확보 경로)

1. **한컴 HWP5 재저장 실험** — fixture 를 MCP 로 HWP5 재저장 → rhwp 파싱 덤프:
   `numberings 0 / outline_numbering_id 0 / OUTLINE shape 3 보존`.
   **기본 모양은 파일에 실체화되지 않는 편집기 내장 동작**이며, HWP5 도 동일 결함
   대상임이 확정됐다.
2. **수준 스윕 문서 실측** — fixture 의 기존 paraPr 정의 7종에
   `heading type="OUTLINE" idRef="0" level="0..6"` 을 주입한 검증 문서를 만들어
   한컴 2020 MCP 로 PDF 화 → 각 수준의 실렌더 번호를 직접 판독(위 표).
   교차 검증: 원본 fixture 정답지 p7 의 1.~4.(level 0) 와 정합.
3. 시행착오 기록 — ①신규 문단 주입은 한컴이 빈 출력(침묵 실패) ②신규 paraPr
   id(200~) 추가는 미해결(한컴이 **paraPrIDRef 를 인덱스로 해석**하는 정황) →
   기존 정의 수정 방식으로 성공. 이 정황은 HWPX 직렬화 시 id 연속성 유지가 필요할
   수 있다는 별도 관찰(후속 기록 대상).

## HWP5 경로 확인

렌더러 공유(`compose_heading` → `resolve_numbering_id`)이므로 수정 지점은 양 포맷
공통이다. 한컴 재저장 HWP5 실험이 이를 실물로 확증했다.

## Stage 2 구현 설계 (확정)

`resolve` 실패 지점에서 합성 기본 Numbering 으로 fallback:

- 발동 조건(좁게): `HeadType::Outline` **그리고** 유효 numbering 정의 부재
  (id=0 또는 정의 조회 실패). NUMBER/BULLET/None 은 불변.
- 합성 내용: 7수준 전부 `level_formats = "^N"`, `number_format = 아라비아`,
  `level_start_numbers = 1`.
- 카운터는 기존 `numbering_state.advance` 재사용(합성 정의에 가상 id 부여).

## 기존 개요 테스트 현황

`tests/` 에 outline/numbering 렌더 회귀 테스트 부재(doclang adapter 단위 테스트만
존재). 신설 테스트가 첫 렌더 계약이 된다.
