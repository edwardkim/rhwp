# Task M100 #1382 최종 보고서 — HWPX autoNum 폭 축 일관화

- 이슈: #1382 "HWPX 파서: autoNum 폭 비일관 — char_shapes 축 1 유닛 vs offsets 축 8 유닛"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1382`
- 작성일: 2026-06-13

## 1. 결함과 해소 (근본 원인 2축 — 1단계에서 확장 확정)

| 축 | 종전 | 해소 |
|----|------|------|
| ① 파서 char_shapes 경계 | `calc_utf16_len_from_parts`가 `\u{0012}`(AUTO_NUMBER)를 1유닛 집계 — offsets 축(8유닛, #1050 한컴 정합)과 비일관 → 143E 경계 시프트 | 8유닛 분기에 추가 (동류 비일관 토큰 1:1 대조 — 단 1건) |
| ② serializer 슬롯 시스템 | autoNum 규약(placeholder가 슬롯 8유닛의 첫 유닛 점유 — HWP5/HWPX 공통)을 비인지 → 잉여 7 측정 → 슬롯 0 추정 → mismatch 경로 → **ctrl 끝 방출 + placeholder 텍스트 이중 방출** | 추론 보정(`(gap+autonum)/8`) + placeholder 위치 ctrl 방출·placeholder 미방출 (판별: 공백+위치+직후 offset 8-jump) |

영향 범위는 autoNum 보유 문단 전수 14건 (각주 11·캡션 1·본문 2) — #1387 한컴 판정의
"캡션 번호 문장 끝 밀림"이 ②의 가시 발현이었다.

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 메커니즘 추적 (근본 원인 2축 확정) + 스펙·HWP5 교차 + 소비처 축 조사(1유닛 전제 없음) + 구현계획 보정 승인 | `7051ace0` |
| 2 | 파서 calc 정정(2a) + serializer placeholder 슬롯(2b) + 테스트 5종 | `1bcdcd9b` |
| 3 | 143E xfail 승격 + #1387 캡션 테스트 완전 일치 승격 + 슬롯 위치 회귀 테스트 | `97a6dbc3` |
| 4 | 전수 검증 + SVG 귀속 + 매뉴얼·최종 보고서 | (본 커밋) |

수정 파일: `src/parser/hwpx/section.rs`, `src/serializer/hwpx/section.rs`,
`src/serializer/hwpx/table.rs`(테스트), `tests/hwpx_roundtrip_baseline.rs` —
렌더러·레이아웃 무변경.

## 3. 검증

### 3.1 전수 배치 (`output/poc/task1382/`)

- **PASS 49 / IR_DIFF 0** / SERIALIZE_FAIL 4(#1384) / PARSE_FAIL 1(제외) —
  종전 PASS 48에서 143E 승격, ROUND2_DIFF 0 (2-round 안정)
- baseline 4 passed — **#1382 귀속 xfail 0** (잔존 4건 전부 #1384)

### 3.2 왕복 대칭 (발현 2샘플)

- 143E 각주[0]·ta-pic 캡션[0]: text/char_count/char_offsets/char_shapes **4축 완전
  왕복 대칭** (143E char_shapes `[(0,10),(9,11)]`, offsets 8-jump 보존)
- RT XML이 한컴 원본 동형: `<hp:t>&lt;그림 </hp:t><hp:ctrl><hp:autoNum…`

### 3.3 SVG 대조 (`output/poc/task1382/svg/`)

- **ta-pic-001-r: 원본 vs RT SVG md5 완전 동일** — #1387 4단계 잔존이던 캡션 행
  -3.5px 시프트 해소. ta-pic은 캡션 포함 완전 일치 레퍼런스 샘플이 됨.
- 143E: RT 페이지 수 1→2 잔존 — **#1382 무관 실증** (#1379/#1380/#1387 시대 RT
  전부 동일 2페이지 + tbl pageBreak 패치 실험으로도 불변). 4절 신규 발견으로 분리.

### 3.4 CI급 검증 (release-test 프로필) + 통합 테스트 보완 1건

- 1차 전체 실행에서 `issue_1100_hwpx_even_header_page_auto_number_…` 1건 실패 검출
  (2·3단계는 lib+baseline만 실행해 통합 테스트 사각) — **회귀가 아니라 정합화에
  따른 앵커 갱신 건으로 판정**: exam_social 짝수 머리말의 fwSpace(\u{2007})가
  한컴 원본 run 구조대로 자동번호와 같은 run(charPr 63) 스타일로 귀속되며
  x 100.47→103.83 이동 (종전엔 1유닛 축 경계 탓에 후속 run 74로 잘못 귀속).
  쪽번호 "2"의 위치(77.47)·1회 치환·fwSpace 보존 의도는 전부 불변 — 앵커 좌표만
  갱신 (#1382 주석 명기).
- 재실행: `cargo test --profile release-test --tests` — **2252 passed, 0 failed**
  (기존 2243 + 신규 9: parser 2 + serializer 3 + 게이트·캡션 승격 4), fmt 통과, clippy 0
- 임시 추적 테스트(`tests/tmp_1382_trace.rs`) 삭제 완료

## 4. 신규 발견 — 별도 이슈 등록 완료 (#1407)

143E 잔존 차이 2건 (모두 #1382 이전부터 존재 — 시대별 RT 대조 실증):

| 발견 | 증상 | 제안 |
|------|------|------|
| ① newNum 슬롯 위치 변위 | 문단 0.14(머리말+하이퍼링크 필드 복합 문단)에서 newNum(PAGE 새번호) ctrl이 fieldEnd 뒤로 이동 — char_offsets[3] 27→35 (8유닛). placeholder 없는 8-슬롯 타입의 위치 결함 (autoNum과 별개 계열) | **#1407** (묶음 등록) |
| ② 143E RT 페이지 수 1→2 | 모든 시대 RT 동일, tbl pageBreak(#1393) 패치로도 불변 — 원인 미규명 (머리말 컨트롤 내용 등 후보) | **#1407** (묶음 등록) |

(tbl pageBreak RowBreak→CellBreak 자체는 #1393 기지)

## 5. 한컴 판정

- **`output/poc/task1382/ta-pic-001-r.rt.hwpx`** — 캡션 번호 "&lt;그림 1&gt;"
  원위치 표시 **판정 통과** (#1387 재판정의 "문장 끝 밀림" 해소 확인 완료).
  셀 안 그림 크기는 #1389 기지 — 범위 밖.

## 6. 산출물

- 계획서: `mydocs/plans/task_m100_1382{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1382_stage{1..3}.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (#1382 해소 2행 + 등급 현황 A=49/B=4)
- 검증 산출물: `output/poc/task1382/` (전수 rt + svg/)
