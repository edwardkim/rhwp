# Task M100 #1387 — 2단계 완료 보고서 (serializer 수정)

- 브랜치: `local/task1387`
- 작성일: 2026-06-12
- 수정 파일: `src/serializer/hwpx/table.rs` (공유 헬퍼 추출 + `write_caption` + 테스트 6종)

## 1. 구현 내용

### 1.1 공유 헬퍼 추출 (2.1)

`write_sub_list`의 문단 방출 본체를 `write_sub_list_paragraphs(w, paragraphs, ctx)`로
추출 — 셀(#1379)과 캡션(#1387)이 공유. 셀 경로 동작 무변경 (기존 테스트 + baseline 보증).

### 1.2 `write_caption` 신설 (2.2)

- 호출 위치: `write_out_margin` 직후 (모듈 doc 자식 순서 `… outMargin, (caption …), inMargin`)
- 속성 5종 역매핑, 한컴 실물 순서: side(direction 4종) / fullSz(include_margin) /
  width / gap(spacing) / lastWidth(max_width)
- subList 래퍼: 전수 17건 동일 실물 고정값 (vertAlign=TOP lineWrap=BREAK
  textDirection=HORIZONTAL — 1단계 측정 근거)

## 2. 단위 테스트 (2.3)

| 테스트 | 검증 |
|--------|------|
| `task1387_caption_attrs_reflect_ir` | 속성 5종 + 실물 순서 + outMargin↔inMargin 사이 배치 |
| `task1387_caption_side_reflects_direction` | direction 4종 역매핑 |
| `task1387_caption_paragraph_controls_emitted` | 캡션 문단 내 autoNum 방출 (공유 경로) |
| `task1387_no_caption_no_emission` | 캡션 없는 표 무변화 |
| `task1387_ta_pic_001_r_roundtrip_preserves_caption` | 실샘플 — 속성 5종 + 문단 수 + 텍스트 본문 + 컨트롤 수 보존 |
| `task1387_mel_001_roundtrip_caption_text_exact` | autoNum 없는 캡션(side=TOP) 텍스트 완전 일치 |

`cargo test --lib serializer::hwpx` — **168 passed, 0 failed**. `cargo fmt --check` 통과.

## 3. 신규 관찰 2건 (귀속 판정)

### 3.1 캡션 autoNum 슬롯 변위 — #1382 귀속

ta-pic-001-r 캡션(autoNum 보유 유일 샘플)의 roundtrip에서 autoNum 슬롯이 문단 끝으로
밀리고, 재파싱 시 끝 placeholder 공백 1자가 추가된다.

- 원인: HWPX 파서가 autoNum placeholder를 char_offsets 축에서 1유닛으로 적재 →
  serializer `inferred_control_slot_count` = 0 ≠ controls 1 → mismatch 경로(텍스트 후
  슬롯 일괄 방출) 진입. **#1382(autoNum 폭 비일관) 파서 결함의 발현**이며 본문(143E
  IR_DIFF xfail)과 동일 계열 — 캡션 경로 신규 결함 아님.
- 영향: 4단계 ta-pic SVG 비교에서 캡션 행의 autoNum 숫자 위치/끝 공백 차이가 잔존할
  것으로 예상 — **"완전 일치" 목표는 "캡션 텍스트 복원 + 잔존 차이 #1382 전량 귀속"으로
  보정 필요** (4단계 보고서에서 정량 입증 예정).
- 실샘플 테스트는 #1382 변위를 주석으로 명시하고 텍스트 본문 보존을 검증.

### 3.2 autoNum numType 역매핑 — 기존 동작 기록

원본 `numType="PICTURE"` → RT `numType="FIGURE"` (serializer 기존 전역 역매핑,
section.rs `Picture→FIGURE`). 파서가 FIGURE/PICTURE 모두 수용해 IR 대칭(게이트 비가시).
본 타스크 범위 밖 — 한컴 수용성은 4단계 한컴 판정에서 겸사 확인.

## 4. 전수 배치 재실행 (2.4)

`hwpx-roundtrip --batch samples/hwpx -o output/poc/task1387`

| 항목 | 수정 전 | 수정 후 |
|------|--------|--------|
| RT 표 캡션 | 0건 (전량 소실) | **5/5건 복원** (143E 1, aift 2, mel-001 1, ta-pic 1 — exam_social은 #1384 xfail로 RT 부재) |
| 그림/도형 캡션 | 소실 | 소실 유지 (범위 밖 — 별도 이슈 제안 예정) |
| 배치 요약 | PASS 48 / xfail 5 / 제외 1 | **동일** (신규 실패 0, ROUND2_DIFF 0 — 2-round 안정) |

## 5. 다음 단계

3단계 — `TableCaption` 게이트 동승 + Table arm 3곳 caption 문단 재귀.

승인 요청드립니다.
