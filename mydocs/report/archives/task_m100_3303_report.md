# Task #3303 — HWP3 문단 테두리 '없음' 오렌더 정정 (최종 보고서)

- Issue: [#3303](https://github.com/edwardkim/rhwp/issues/3303)
- Branch: `task/3303-para-border-none-render`
- 계획서: `mydocs/working/task_m100_3303_stage1.md` / `_stage2.md`

## 결론

SO-SUEOP.hwp 42쪽 지문 문단에 그려지던 테두리 상자(v0.8.0 시각 회귀)와
hwp3-sample4 음영 문단 좌우의 검은 세로선은 **같은 뿌리** — #2995(bde926e4b)가
HWP3 `border=1`을 4방향 Solid로 합성했고, `BorderLineType`의 Rust 기본값이 Solid라
음영 경로의 BorderFill까지 Solid 선을 갖게 된 것이다. 파서에서 4방향 선을 명시적
`None`으로 배선해 정정했다(렌더러 무수정 — `para_border_is_visible`이 None을 걸러줌).

## 권위 근거 (3중 교차검증)

| 층위 | 실측 |
|---|---|
| 원시 바이트 | 42쪽 지문 셰이프 `shade_ratio=0, border=1, border_connection=1` |
| 스펙 정독 | 표 13(187B)에 테두리 선 종류/굵기/색 필드 부재 — offset 181은 on/off 뿐 |
| 한컴 UI | 문단 모양 대화상자: 종류 "선 없음"·0.1mm·검정, 문단 테두리 연결 ON |
| 한컴 변환 HWPX | `samples/SO-SUEOP.hwpx` `paraPr 38 → border borderFillIDRef="1" connect="1"`, `borderFill 1` 4방향 `type="NONE"` |
| 코퍼스 스윕 | HWP3 271개 전수에서 `border=1`은 SO-SUEOP 유일 — Solid 합성의 양성 증거 전무 |

부수 확정: 스펙 표 13 offset 182 "1=선 연결 안 함"은 실측과 **극성 반대**
(`border_connection=1 → connect="1"` 연결 ON). rhwp 배선(#2976, attr1 bit28)은 실측과
일치해 무변경. 두 사항 모두 `mydocs/tech/한글문서파일구조3.0.md` 표 13 아래 보완
주석으로 기록(#1129 보완 주석 관례를 따름 — hwp_spec_errata.md는 HWP5 전용 정오표라
위치 변경).

## 수정

- `src/parser/hwp3/mod.rs` `hwp3_para_shape_border_fill()`: 4방향 선을 명시적
  `BorderLineType::None`으로 채우고 has_border Solid 합성 블록 삭제. bf 생성
  조건(`shade>0 || border=1`)·참조·연결 배선 무변경 — 한컴 변환 HWPX와 동형 구조.
- 테스트 3케이스 교체/추가: border만 → None 선 / 음영만 → None 선+Solid fill /
  둘 다 없음 → None 반환.

## 검증 결과

| 항목 | 결과 |
|---|---|
| 단위(hwp3 파서 36건, 신규 3건 포함) | 통과 |
| 전체 `cargo test --tests --profile release-test` | 323 스위트 전부 ok, 실패 0 |
| `cargo fmt --check` | 통과 |
| IR 덤프 | ps 947 `border_fill_id=5` 참조 구조 보존 |
| SVG | 42쪽 stroke 6→2(소멸 4 = 지문·222번 좌우 세로선, 잔여 2 = 머리말 구분선·본문 밑줄), sample4 stroke 4→0 + 음영 rect 2 보존 |
| render-tree JSON | 테두리 Line 노드 소멸(잔여 1 = 머리말 구분선) |
| skia (`--features native-skia` export-png) | 42쪽에서 지문 높이 80%+ 관통 세로 열 0개 (수정 전 우측 세로선 실측 지점) |
| canvas2d (wasm 재빌드, studio) | **작업지시자 시각 판정 통과** (2026-07-26) |

수정 영향 파일은 코퍼스 스윕상 SO-SUEOP·hwp3-sample4 2개뿐 — 그 외 HWP3 문서
무영향은 구조적으로 보장된다.

## 후속 (범위 밖)

- **ir-diff 확장 제안**: ParaShape 비교 항목에 `border_fill_id` + 참조 BorderFill
  4방향 선 종류 요약 추가. 이번 발산(HWPX NONE vs HWP3 Solid)은 현행 ir-diff
  비교 항목(ml/mr/indent/tab_def/sb/sa/ls) 밖이라 자동 검출되지 않았다.
- 한컴이 문단 테두리를 실제로 그리는 HWP3 양성 샘플(한글 97 형식 저장 대비 실험)이
  확보되면 선 합성 재도입을 그 실측 근거로 판단한다.

## 릴리즈

0.8.1 PATCH 대상 (#3348과 함께).
