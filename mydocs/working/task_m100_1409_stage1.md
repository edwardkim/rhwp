# Task M100 #1409 — 1단계(완료) 보고서 (numbering 등록 축 정정)

- 브랜치: `local/task1409`
- 작성일: 2026-06-14
- 수정 파일: `src/serializer/hwpx/context.rs`

## 1. 결함과 해소 (#1384 동형)

`write_numbering`이 numbering id 를 `idx+1`(1-based) 방출하는데 등록만 `idx`
(0-based)였다 — borderFill(#1384)과 동형 off-by-one.

- 수정: `context.rs:128` `register(idx)` → `register((idx + 1))`. 방출(1-based)·
  실물(온새미로 id 1~)과 통일.
- 전 ID 풀 방출축 vs 등록축 대조: `id+1` 방출은 border_fill·numbering 둘뿐이고
  border_fill 은 #1384 정정 완료 → **numbering 단 1건 잔존**. **동형 결함 추가 없음 확인**.
- numbering 은 `reference()` 호출 0건이라 현재 미표면화이나, 등록 축 일관성 +
  HWP5 변환·미래 검사 활성화 대비.

## 2. 검증

- 단위 테스트 `task1409_numbering_registered_one_based`: numbering 8개 적재 →
  id=8 등록 + 0 미등록 (회귀 가드). context 7 passed.
- `cargo test --test hwpx_roundtrip_baseline` 4 passed — **B=0 유지**.
- `hwpx-roundtrip --batch`: PASS 53 / IR_DIFF 0 / SERIALIZE_FAIL 0 (회귀 없음).
- CI급: `cargo test --profile release-test --tests` 전체 그린 (수치 커밋 시점),
  fmt 통과, clippy 0.

## 3. 매뉴얼

`hwpx_roundtrip_baseline.md` known limitations 에 #1409 해소 행 추가.

## 4. 잔존 한계

| 한계 | 이슈 |
|------|------|
| newNum 슬롯 위치 + 143E RT 페이지 수 | #1407 |
| circleType TIRANGLE owpml 철자 미확인 | #1402 보고서 4절 (roundtrip 정합) |

신규 발견 없음. **ID 등록 축 off-by-one 계열은 #1384(borderFill)·#1409(numbering)로
전수 해소.**

## 5. 산출물

- 수행계획서: `mydocs/plans/task_m100_1409.md`
- 본 보고서(최종): `mydocs/working/task_m100_1409_stage1.md` + `mydocs/report/task_m100_1409_report.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md`
