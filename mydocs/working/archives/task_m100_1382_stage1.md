# Task M100 #1382 — 1단계 완료 보고서 (메커니즘 추적 + 소비처 축 조사)

- 브랜치: `local/task1382`
- 작성일: 2026-06-12
- 코드 수정: 없음 (조사 전용 — 임시 추적 테스트 `tests/tmp_1382_trace.rs`는 미커밋,
  4단계 종료 시 삭제)

## 1. 발현 2패턴 정밀 추적 (1.1) — 메커니즘 정정

IR 실측 (143E 각주[0] / ta-pic 캡션[0], 원본 vs 재파싱):

| 항목 | 143E 원본 | 143E 재파싱 | ta-pic 원본 | ta-pic 재파싱 |
|------|----------|------------|------------|--------------|
| char_offsets | `[0, 8, 9, …]` (jump 8) | `[0, 1, 2, …]` (**jump 소실**) | `[…4, 12, …]` (jump 8) | 연속 (**jump 소실**) |
| char_shapes | `[(0,10),(2,11)]` | `[(0,10),(1,11)]` | `[(0,6)]` | `[(0,6)]` |
| text | placeholder 공백 포함 | 끝 공백 1자 추가 | 〃 | 〃 |

**두 발현 모두 동일 메커니즘** — 슬롯이 문단 끝으로 방출되어 재파싱 offsets에서
8유닛 jump 자체가 소실된다. 경계 시프트(143E)는 그 위에 char_shapes 축 결함이
얹힌 것.

### 근본 원인 2축 (계획 대비 1축 추가 확정)

**① char_shapes 경계 축 (계획대로)**: `calc_utf16_len_from_parts`가 `\u{0012}`를
1유닛으로 집계 → 143E 경계 2 (offsets 축 정답 9 = autoNum 8 + 공백 1).

**② serializer 슬롯 시스템의 placeholder 비인지 (신규 확정)**: autoNum IR 규약
(HWP5 `body_text.rs:326`·HWPX offsets 루프 공통)은 **placeholder 문자가 슬롯
8유닛의 첫 유닛을 점유** (placeholder at P, 다음 문자 P+8). 그런데 슬롯 추론
(`inferred_control_slot_count`)과 방출 루프는 placeholder 없는 순수 8-gap
(Table/0x02류 — 텍스트 미점유)을 전제한다:

- 추론: placeholder가 expected를 1 전진시켜 잔여 gap = **7** → `7/8 = 0` 슬롯
- char_count 축도 동일: `(char_count-1-text_units)/8 = 7/8 = 0`
- → `slot_count(0) ≠ slots.len()(1)` → **mismatch 경로** → ctrl 일괄 끝 방출
- 방출 시 placeholder가 텍스트로 이중 방출 (한컴 원본 XML에는 placeholder 텍스트가
  없음 — `<hp:ctrl><hp:autoNum/></hp:ctrl><hp:t> </hp:t>` — placeholder는 파서가
  IR에 합성하는 것)

**따라서 ① 정정만으로 슬롯 위치는 해소되지 않는다** (mismatch 경로 진입 원인은 ②).

## 2. 스펙 교차 확정 (1.2)

- HWP 5.0 제어 문자 분류 (표 6, `body_text.rs` doc 주석): 0x12는 extended —
  8 code unit 점유. ✓
- HWP5 파서(`body_text.rs:326~330`): `char_offsets.push(pos)` + `text.push(' ')`
  (placeholder) + `pos += 16` — **HWPX offsets 루프(#1050)와 동일 규약** 재확인.
- HWP5 char_shapes는 CHAR_SHAPE 레코드 위치(8유닛 축) 그대로 적재 → ① 정정은
  HWP5 정합 방향.

## 3. 소비처 축 의존성 조사 (1.3)

| 소비처 | 축 | 판정 |
|--------|-----|------|
| renderer `composer.rs` `split_by_char_shapes` | char_offsets 매핑 경유 (8유닛 축) | ① 정정이 정합 방향. 현재 143E는 실공백 1자에 스타일이 잘못 적용 중 (육안 비가시 수준) |
| renderer `composer.rs` 마커 합성 (`:117`, `:215`) | char_offsets gap (8 wchar) 분석 | 8유닛 축 — 영향 없음 |
| serializer `RunSplitter`/슬롯 | offsets 축 전제 + 순수 8-gap 전제 | ② 보정 대상 |
| `style_resolver.rs` | char_shapes **id** 참조만 (위치 비소비) | 무관 |

**1유닛 축을 전제한 소비처 없음** — ① 정정의 회귀 위험 낮음.

## 4. 전수 분포 (1.4)

autoNum 보유 문단 **14건** (전수 파싱 sweep): footnote-01 각주 9 + 143E 각주 1(경계
동반 유일) + footnote-tbox-01 1 + eq-002 본문 1 + aift 각주 1 + ta-pic 캡션 1.

- **14건 전부 ② 슬롯 끝 변위 영향권** (boundary 동반 여부 무관) — 각주 번호도 ctrl이
  문단 끝으로 가 있어, 한컴이 ctrl 위치에 번호를 렌더하면 각주 전반에서 위치 오류.
- ① char_shapes 경계 영향은 143E 1건 (baseline IR_DIFF xfail과 1:1).
- 게이트 영향 예측: ① 정정으로 143E xfail 해소. ② 정정은 텍스트/offsets 축 — 현행
  게이트 비교 항목 밖이라 xfail 변동 없음. 신규 xfail 0 예상.

## 5. 구현계획 보정 (승인 요청)

2단계를 파서·serializer 2축으로 확대한다 (3·4단계 구성 불변):

- **2a. 파서** (계획대로): `calc_utf16_len_from_parts`에 `\u{0012}` 8유닛 + 동류
  토큰 1:1 대조
- **2b. serializer** (신규): `render_paragraph_parts` 슬롯 시스템의 autoNum
  placeholder 인지 —
  1. 슬롯 추론: AutoNumber 슬롯의 placeholder 점유(7-gap)를 슬롯 1개로 집계
  2. 방출 루프: AutoNumber 슬롯 위치에서 ctrl 방출 + 대응 placeholder 문자 1자를
     텍스트에서 삼킴 (한컴 원본 XML 동형: `<hp:ctrl><hp:autoNum/></hp:ctrl>` + 후속 텍스트)
- 검증 기대치: RT XML 한컴 원본 패턴 동형, 재파싱 offsets 8-jump 복원, 143E
  char_shapes `[(0,10),(9,11)]` 왕복 대칭, 2-round 안정, 각주 14건 전수 위치 복원

승인 요청드립니다.
