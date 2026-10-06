# Task M100 #1387 최종 보고서 — HWPX serializer 표 캡션(hp:caption) 직렬화

- 이슈: #1387 "HWPX serializer: 표 캡션(hp:caption) 미직렬화 — 캡션 subList 소실"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1387`
- 작성일: 2026-06-12

## 1. 결함과 해소

| 축 | 종전 | 해소 |
|----|------|------|
| serializer | `write_table`이 caption 미방출 — 전수 표 캡션 6건 소실 | `write_caption` 신설 (속성 5종 역매핑 + subList 공유 경로) — outMargin↔inMargin 사이, 한컴 실물 순서 |
| 게이트 | Table 재귀가 `cells`만 순회 — caption 사각 | `diff_table_caption` → `TableCaption` 동승 (존재/속성 5종/문단 수) + 캡션 문단 char_shapes·controls·linesegs 재귀 (`tbl.caption.p[k]`) |
| 파서 | 정상 (수정 없음) | `vert_align` 미파싱은 공백이 아니라 포맷 차이 — HWPX `hp:caption`에 대응 속성 부재 (전수 17건 실측) |

공유 헬퍼 `write_sub_list_paragraphs` 추출로 셀(#1379)·캡션이 같은 문단 직렬화
경로를 사용 — 셀 경로 무변경.

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 전수 측정 (표 6 + 그림/도형 11, RT 전량 소실) + vert_align 조사 종결 | `717e53ea` |
| 2 | `write_caption` + 공유 헬퍼 + 테스트 6종 | `8fcfe7f9` |
| 3 | `TableCaption` 게이트 동승 + 테스트 4종 | `697ce7bd` |
| 4 | 전수 검증 + SVG 귀속 정량화 + 매뉴얼·최종 보고서 | (본 커밋) |

수정 파일: `src/serializer/hwpx/table.rs`, `src/serializer/hwpx/roundtrip.rs` —
serializer/게이트 한정, 렌더러·레이아웃·파서 무변경.

## 3. 검증

### 3.1 전수 배치 (`output/poc/task1387/`)

- 배치 요약: PASS 48 / IR_DIFF 1(#1382) / SERIALIZE_FAIL 4(#1384) / PARSE_FAIL 1(제외)
  — 캡션 게이트 동승 후에도 **동일, 신규 실패 0**, ROUND2_DIFF 0 (2-round 안정)
- RT 표 캡션: 0건 → **5/5건 복원** (143E 1, aift 2, mel-001 1, ta-pic 1;
  exam_social은 #1384 xfail로 RT 부재)
- `cargo test --test hwpx_roundtrip_baseline`: 4 passed — **신규 xfail 0** (1단계 사전 판정 적중)

### 3.2 ta-pic-001-r SVG 대조 (`output/poc/task1387/svg/`)

- 좌표 수 **40 = 40** (종전 40 vs 22 — 캡션 18좌표 복원)
- 잔존 차이: 캡션 행(y=422.85) 18글자의 **균일 -3.5px 수평 시프트뿐** — autoNum
  슬롯 변위(#1382)로 줄 폭이 7px 달라져 가운데 정렬이 재배치된 것. 본문/표 22좌표는
  완전 동일. **잔존 전량 #1382 귀속** (2단계 보고서 3.1의 목표 보정대로 정량 입증)

### 3.3 CI급 검증 (release-test 프로필)

- `cargo test --profile release-test --tests` 전체 그린 — **2243 passed, 0 failed**
  (기존 2233 + 신규 10: serializer 6 + 게이트 4)
- `cargo fmt --check` 통과, clippy 경고 0

## 4. 관찰·귀속 (신규 이슈 아님)

| 관찰 | 귀속·처리 |
|------|------|
| 캡션 autoNum 슬롯 끝 방출 + 재파싱 끝 공백 1자 (ta-pic) | **#1382 발현** — 파서가 placeholder를 1유닛 적재 → `inferred_control_slot_count` mismatch. 본문 143E xfail과 동일 계열 |
| autoNum `numType` FIGURE 방출 → 한컴 캡션 번호 미출력 | **본 타스크에서 정정** — 1차 한컴 판정(ta-pic 번호 미출력)으로 실증. FIGURE는 한컴 생산 파일 비실재(전수 sweep 0건, 실물은 PICTURE 표기). `auto_number_type_to_str` Picture→"PICTURE" 정정 + 테스트 단정 추가. 파서는 FIGURE/PICTURE 양쪽 수용 유지 |

## 5. 별도 이슈 등록 (승인 완료)

| 이슈 | 내용 |
|------|------|
| **#1403** | 그림/도형 캡션(hp:caption) 미직렬화 — aift 실측 pic 8 + line 3 = 11건 소실. `ShapeComponent.caption` 경로로 표 캡션과 별개 |
| **#1402** | 열거 속성 표면 표기 정합 검사 — numType FIGURE 선례의 일반화. serializer 열거 토큰 방출 약 40여 함수가 동일 패턴 후보 (파서 관용 수용 → 게이트 비가시) |

## 6. 잔존 한계 (기지 이슈 귀속)

| 한계 | 이슈 |
|------|------|
| 캡션 행 3.5px 시프트 (autoNum 변위) | #1382 |
| hp:pic 크기 요소 IR 미반영 | #1389 |
| 표 pageBreak 일괄 TABLE 방출 | #1393 |
| MEMO subList / shapeComment 소실 | #1391 / #1392 |
| 그림/도형 캡션 소실 | #1403 (5절) |
| 열거 토큰 표기 정합 사각 | #1402 (5절) |

## 7. 한컴 판정

`output/poc/task1387/`의 RT 2건 (한컴에디터):

1. **mel-001.rt.hwpx** — 표 위 캡션("(단위 : 억원, %)", side=TOP) **복원 성공** (1차 판정)
2. **ta-pic-001-r.rt.hwpx** — 1차 판정: 캡션 본문 복원, 번호 미출력 → numType
   FIGURE→PICTURE 정정 후 재판정: **번호 출력 확인** ("…예시1" — 문장 끝 위치).
   위치 변위는 #1382 귀속 (한컴은 ctrl XML 위치(끝)에, rhwp-studio는 placeholder
   문자 위치(mid-text)에 렌더 — RT 내부 모순을 렌더러가 다르게 해석). 후속 #1382
   타스크에서 슬롯 위치 정정 예정. 셀 안 그림 크기는 #1389 기지.

## 8. 산출물

- 계획서: `mydocs/plans/task_m100_1387{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1387_stage{1..3}.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (게이트 항목 + #1387 해소 + #1382 발현 행)
- 검증 산출물: `output/poc/task1387/` (전수 rt + caption_inventory.tsv + svg/)
