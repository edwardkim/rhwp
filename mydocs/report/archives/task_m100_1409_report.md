# Task M100 #1409 최종 보고서 — numbering 등록 축 정정 (#1384 동형)

- 이슈: #1409 "HWPX serializer: numbering 등록 축 0/1-based 불일치 — borderFill(#1384) 동형 잠재 결함"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1409`
- 작성일: 2026-06-14

## 1. 결함과 해소

`write_numbering`(header.rs:788)이 numbering id 를 `idx+1`(1-based) 방출하는데 등록만
`idx`(0-based) — borderFill(#1384) 동형 off-by-one.

- 수정: `context.rs:128` `register(idx)` → `register((idx + 1))` (1줄). 방출(1-based)·
  실물(온새미로 id 1~)과 통일.
- 전 ID 풀 방출축 vs 등록축 전수 대조: `id+1` 방출은 border_fill·numbering 둘뿐,
  border_fill 은 #1384 정정 완료 → **numbering 단 1건 잔존, 동형 결함 추가 없음 확인**.
- numbering 은 `reference()` 호출 0건이라 현재 미표면화(잠재). 등록 축 일관성 +
  HWP5 변환·미래 검사 활성화 대비 정정.

## 2. 검증

- 단위 테스트 `task1409_numbering_registered_one_based`: numbering 8개 → id=8 등록 +
  0 미등록 (회귀 가드). context 7 passed.
- baseline 4 passed — **B=0 유지**.
- `hwpx-roundtrip --batch`: PASS 53 / IR_DIFF 0 / SERIALIZE_FAIL 0 (회귀 없음).
- CI급: `cargo test --profile release-test --tests` 전체 그린 (수치 커밋 시점 기재),
  fmt 통과, clippy 0.

## 3. 의의 — ID 등록 축 계열 전수 해소

ID 등록 축 0/1-based off-by-one 결함은 **#1384(borderFill)·#1409(numbering)로 전수
해소**. 나머지 풀(char_shape/para_shape/tab/style)은 방출·등록 모두 0-based 정합.

## 4. 잔존 한계 (기지 이슈)

| 한계 | 이슈 |
|------|------|
| newNum 슬롯 위치 + 143E RT 페이지 수 | #1407 |
| circleType TIRANGLE owpml 철자 미확인 | #1402 보고서 4절 (roundtrip 정합) |

신규 발견 없음.

## 5. 산출물

- 계획서: `mydocs/plans/task_m100_1409.md`
- 단계별 보고서: `mydocs/working/task_m100_1409_stage1.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md`
