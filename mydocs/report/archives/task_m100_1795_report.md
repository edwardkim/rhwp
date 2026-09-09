# 최종 결과 보고서 — Task M100 #1795

## 이슈

HWPX→HWP 라운드트립: 글상자 내부 줄바꿈 지점 차이 (seoul_0043, OVER 101px)

## 결론

이슈 등록 시 가설(텍스트 폭 계측 불일치)은 기각. 실제 원인은 **HWP5 직렬화기의
필드 컨트롤 배치 버그**다. `serialize_para_text` 의 갭 채우기 루프가 각 문자
인덱스에 삽입될 FIELD_END(8 code unit)의 공간을 예약하지 않아, FIELD_END 전용
갭을 다음 컨트롤(다음 필드의 FIELD_BEGIN)이 선점했다. 그 결과 재파싱 시
char_offsets 가 시프트되고, 보존된 lineseg text_start 의 utf16→문자 매핑이
어긋나 줄바꿈 경계가 이동했다 (3번째 줄이 글상자 우변을 100px 초과하는 렌더 포함).

## 원인 상세 (seoul_0043 글상자 p[1], 필드 2개 + 컨트롤 1개)

| | A (HWPX 직파스) | B (변환본 재파스) |
|---|---|---|
| char_offsets idx35 점프 | +9 (FIELD_END 1블록) | **+17 (2블록 — FIELD_BEGIN이 선점)** |
| field_ranges[1] | 101..105 | **61..105** |
| lineseg text_start | [0,60,112,173,217] (동일) | 동일 — 매핑만 어긋남 |

char_shapes/CharShape 정의/폰트 정의는 완전 동일 (프로브 대조). 렌더 트리에서
줄 1~3만 경계 이동, 4번째 줄부터 재일치 — offsets 시프트 패턴과 정합.

ir-diff 가 Shape 글상자 문단을 재귀 비교하지 않아(controls 는 common 속성만)
이전 소거에서 빠져나갔다 — **도구 개선 후보**로 기록.

## 수정 내역

| 파일 | 수정 |
|------|------|
| `src/serializer/body_text.rs` | 갭 채우기 전 해당 인덱스 FIELD_END 공간(개수×8cu) 예약: `while prev_end + 8 + pending_field_end_cus <= offset` |

회귀 테스트: `test_field_end_gap_not_stolen_by_next_control` — 필드 2개 문단
직렬화→재파싱 char_offsets/field_ranges 보존.

## 검증 결과

1. **seoul_0043**: render-diff --via hwp OVER 101px → **PASS 0.00px**
2. **big_hwpx 2,500 배치** (--via hwp, #1794 시점 대비): 변화 7건 전부 개선, **회귀 0** —
   seoul_0043(101→0), **seoul_0978(150→0)**, STRUCT_MISMATCH 해소 3건
   (admrul_0262, admrul_1077, seoul_0950), STRUCT 완화 2건 (admrul_1078 293→118,
   admrul_1080 631→527). 누적 분포: PASS 2453 / OVER 10 / STRUCT 29 / PAGE 4 / LOAD 4.
3. **big_hwp 2,500 배치** (identity): 변화 0건 (HWP5 raw 보존 경로 무영향).
4. **cargo test --release** 전수: 통과 (유일 실패 = #1775 Windows 경로 구분자 기대 실패).

## 세션 누적 효과 (#1793 + #1794 + #1795)

big_hwpx 라운드트립 게이트: 기준선(v3fix) OVER 24 / STRUCT 32 → **OVER 10 / STRUCT 29**
(OVER 19건 해소·신규 0, seoul_0765/0505/0043/0978 포함), PASS 2429 → 2453.

## 후속

- ir-diff 에 Shape 글상자 문단 재귀 비교 추가 (도구 개선, 별도 이슈 후보)
- HWPX→HWP 변환 셀 field_name 소실 (#1794 조사 중 발견, 별도 이슈 후보)
