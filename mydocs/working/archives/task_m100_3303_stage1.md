# Task #3303 Stage 1 — 문단 테두리 '없음' 오렌더 (수행계획서)

## 증상 (이슈 + 오늘 실측)

`samples/SO-SUEOP.hwp` 42쪽 지문 문단들에 rhwp가 테두리 상자를 그린다. 한컴 편집기
문단 모양 대화상자의 테두리 종류는 **"없음"**(작업지시자 실측, 2026-07-25). v0.7.x에는
없다가 v0.8.0에서 나타난 시각 회귀로, 용의 커밋은 bde926e4b(#2995)다.

## 조사 결과 (2026-07-26)

1. **원시 바이트 실측**: 42쪽 지문 문단 셰이프(ps_id=947 계열)는
   `shade_ratio=0, border=1, border_connection=1`. 파일에는 스펙(표 13, offset 181
   "문단 테두리 0=없음, 1=있음")상 "있음"이 기록되어 있다.
2. **원인 확정**: bde926e4b가 `has_border()`(=`border==1`)만으로 4방향
   `BorderLineType::Solid` 테두리를 IR에 배선 → 렌더러가 그대로 그림. 즉 이슈의
   가설 (a) "파서가 '없음' 테두리를 border_fill 참조로 잘못 배선"이 맞다. 렌더러는
   선이 있으니 그리는 것뿐이므로 수정은 **파서 안**에서 끝낸다(CLAUDE.md: HWP3 전용
   해석은 `src/parser/hwp3/` 안에서).
3. **스펙 한계**: HWP3 문단 모양에는 테두리 **선 종류/굵기/색 필드가 없다**(on/off
   1바이트뿐). #2995는 스펙 문구("1=있음")만 근거로 Solid 4방향을 합성했고, red→green
   테스트도 합성 struct 입력뿐 — **한컴 확인 양성 샘플이 없었다**.
4. **코퍼스 스윕(271개 HWP3 전수)**: `border=1`인 파일은 SO-SUEOP.hwp **단 1개**
   (셰이프 2건)이고, 그 파일에 대한 한컴 판정이 "없음"이다. 나머지 border_fill 생성은
   전부 `shade_ratio>0, border=0`(hwp3-sample4, 음영 경로 77건). 즉 **"border=1 →
   한컴이 테두리를 그린다"는 양성 증거가 코퍼스에 전무**하고, 유일한 실측 증거는
   그 반대다.
5. **한컴 자체 변환 HWPX 대조(권위 확정)**: `samples/SO-SUEOP.hwpx`(한컴 편집기가
   같은 V3 문서를 HWPX로 변환한 것)에서 해당 지문 문단 `paraPr id="38"`은
   `<hh:border borderFillIDRef="1" ... connect="1"/>`를 갖고, `borderFill id="1"`은
   **4방향 전부 `type="NONE"` `width="0.1 mm"` `color="#000000"`, 채움 없음**이다.
   한컴 UI 대화상자 실측(종류: 선 없음, 굵기 0.1mm, 색 검정, 연결 ON)과 XML이
   완전 일치 — V3 `border=1`의 목표 매핑이 구조 레벨로 확정됐다.

## 방침 (한컴 UI 실측 반영 — 2026-07-26 작업지시자 제공)

한컴 2022 문단 모양 대화상자 실측으로 V3 → 한컴 매핑이 확정됐다:

| V3 바이트 | 한컴 2022 매핑 |
|---|---|
| `border=1` | 테두리 **종류: 선 없음**(굵기 0.1mm·색 검정 기본값) → 선을 그리지 않음 |
| `border_connection=1` | 문단 테두리 **연결: ON** |
| `shade_ratio=0` | 배경 면 색: 색 없음 |

이에 따라 `hwp3_para_shape_border_fill()`의 **has_border 기반 `Solid` 4방향 합성을
제거하고, 한컴과 동일하게 "선 없음" 매핑으로 정정**한다(#2995의 Solid 합성 철회).
BorderFill 생성·참조 자체는 유지(한컴도 테두리 구조는 유지한 채 선 종류만 없음) —
시각 결과는 "그리지 않음", 왕복 보존 구조는 한컴과 동형. 음영(`shade_ratio>0`)
경로는 무변경.

- 근거: 시각 판정 권위(한컴 2022) UI 실측 + 스펙 정독(표 13에 선 종류 필드 부재)
  교차검증. 스펙 문구("1=있음")는 "테두리 구조 존재"이지 "선을 그린다"가 아님이
  실측으로 확정.
- `hwp_spec_errata.md`에 2건 기록해 재발 방지:
  (1) 표 13 offset 181 — border=1이어도 선 종류 필드가 없어 한컴은 "선 없음"으로
  매핑(Solid 합성 금지). (2) offset 182 — 스펙 "1=연결 안 함"은 실측과 **극성 반대**
  (한컴: border_connection=1 → 연결 ON). rhwp 현행 배선(#2976, attr1 bit28)은 실측과
  일치하므로 무변경.

## 수정 범위

- `src/parser/hwp3/mod.rs` — `hwp3_para_shape_border_fill()`: has_border 시
  `BorderLineType::Solid` 합성을 제거하고 선 종류를 기본값(없음)으로 유지(BorderFill
  생성·참조는 유지). 함수 주석을 #3303 한컴 실측 근거로 갱신. #2995 합성 테스트
  (`test_hwp3_para_shape_border_fill_wires_has_border_flag`)를 "border=1 → BorderFill은
  생성되되 4방향 선 종류는 없음(Solid 합성 금지)" 기대로 교체.
- `mydocs/tech/hwp_spec_errata.md` — 표 13 offset 181(선 없음 매핑)·182(연결 극성
  반대) 실측 불일치 2건 추가.
- 렌더러 무수정 — 단, 검증에서 "선 없음 BorderFill 참조(border_fill_id≠0)일 때
  아무 백엔드도 선을 그리지 않음"을 실측으로 확인(이슈 가설 (b) 기각 확정).

## 검증

1. 단위: 갱신 테스트 (border=1·shade=0 → None / shade>0 → Some+선 없음+음영 유지).
2. IR: `dump samples/SO-SUEOP.hwp`에서 ps_id=947 `border_fill_id=0` 확인.
3. 시각: `export-png -p 41`(42쪽) before/after + **4-backend 대조**(svg/canvas/paint/json
   — 오늘할일 이월 기록의 "수정 시 4-backend 대조 필수") + skia(p42 우측 세로선 소멸).
   after 자산은 작업지시자 시각 판정 게이트.
4. 회귀: hwp3-sample4(음영 77건) before/after 픽셀 diff 0 확인(음영 경로 무영향 증명).
5. push 전: `cargo test --tests --profile release-test` + `fmt --check`.

## PR

- 브랜치 `task/3303-para-border-none-render`, base devel, `Closes #3303`.
- 0.8.1 PATCH 대상. PR 생성은 별도 승인 후.

## 다음 단계

승인 시 Stage 2(구현계획서 — 코드/테스트/errata 문안 확정) 후 구현.
