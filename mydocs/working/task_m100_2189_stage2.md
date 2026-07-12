# Task M100 #2189 — 2단계 완료 보고: 원인 이분 진단

- 이슈: #2189 고정폭 테두리 글상자 우측 텍스트 클리핑 / 브랜치: `local/task2189`
- 재현 파일: `samples/basic/issue1994_behindtext_table_20200830.hwp` p2 좌측 성명서 박스
- 진단일: 2026-07-11

## 판정: **가설 2 확정** (대체 폰트 advance 과대), 가설 1 기각

리포터가 "글상자"로 지칭한 성명서 박스의 실체는 **1×1 표 셀**이다
(문단 0.22, treat_as_char, RowBreak, aim=true).

## 3자 대조 결과

| 항목 | 값 | 근거 |
|------|-----|------|
| 셀 선언 폭 | 36282 HU = 483.8px | dump 셀[0] w=36282 |
| 셀 내부 폭 (pad 850×2 차감) | 34582 HU = **461.1px** | aim=true → cell.padding 사용 |
| SVG 클립 rect | x=61.8 w=483.8 (우측 545.6) | cell-clip-6 = 셀 전체 rect |
| 본문 시작 x | 73.16 (= 셀 x + pad_left 11.33) | **정확** — 가설 1의 시작점 시프트 없음 |
| **한컴 PDF 줄 폭** | 345.8pt = **461.0px** | pdf/issue1994/issue_1994.pdf p2, pdftotext -bbox |
| **우리 그린 줄 폭** | 최대 **483.5px** (+4.9%) | SVG text x 좌표 실측 |
| 클립 초과 | 최대 **+15.1px** (본문 28줄 중 14줄 초과) | 545.6 기준 |

- 한컴 PDF의 본문 줄 폭 461.0px = 우리가 계산한 셀 내부 폭 461.1px — **available
  width 계산은 정확**하다 (가설 1 기각).
- 줄바꿈은 저장된 LINE_SEG ts 를 그대로 사용하므로
  ([paragraph_layout.rs:1348-1360](../../src/renderer/layout/paragraph_layout.rs#L1348-L1360))
  줄당 글자 구성은 한컴과 동일. 같은 글자들을 우리 폴백 메트릭(원본 폰트
  "08서울한강체 M" 미보유)으로 그리면 전각 advance 12.60px/자 누적이 한컴 실폰트
  대비 +4.9% 넓어 우측 테두리를 넘는다 (가설 2).

## 기존 압축 메커니즘의 한계 (왜 지금 안 막히나)

- Justify 음수 슬랙 압축은 존재하나 **공백 폭 −50% 클램프**가 상한
  (`min_ews = -space_base_w*0.5`,
  [paragraph_layout.rs:899](../../src/renderer/layout/paragraph_layout.rs#L899)).
  실측: 공백이 이미 3.32px(최대 압축)까지 줄었는데도 잔여 초과 최대 +15px.
- 공백 클램프 도달 후 **잔여 음수 슬랙을 자간(extra_char_sp)으로 넘기는 경로가
  없다.** 공백-없는 줄 분기(3-tuple `(0.0, raw.max(min_sp), 0.0)`)에는 자간 압축이
  이미 있으나, 공백-있는 Justify 분기는 공백 압축에서 끝난다.
- `suppress_cell_overflow_spacing`(초과 15% 이상 압축 포기)은 이 케이스(4.9%)에서
  비발동 — 정상.

## 3단계 정정 방향 (승인 요청)

**Justify(공백 있음) 셀 오버플로우 스필오버**: 공백 압축이 클램프에 도달하고도
슬랙이 남으면, 잔여 음수 슬랙을 `extra_char_sp`로 분배한다 (기존 공백-없는 분기와
동일한 `-avg_char_w*0.5` 클램프 준용).

- 하드코딩 없음 — 사용 근거는 저장 줄바꿈(LINE_SEG ts) + 셀 내부 폭(선언 필드) +
  측정 자연 폭뿐. 샘플/이슈 분기 없음.
- 음수 자간의 narrow glyph 역진은 기존 per-char 50% 클램프(암묵지 2,
  `troubleshootings/hwp_letter_spacing_compensation.md`)가 이미 방어 — 측정/배치
  5곳 일관성 유지 확인 예정.
- 적용 범위는 `cell_ctx.is_some()` 프레임-제약 컨텍스트로 한정 (본문 일반 문단
  비영향).
- 게이트: 표적 회귀 테스트 신설 + `cargo test --tests` + golden/OVR +
  #1994 영향권(같은 파일) + form-002/table-text 비회귀(#229 가드).

예상 효과: 본문 줄 폭 483.5 → 461.1px 수렴, 클립 초과 0.
