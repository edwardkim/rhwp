# Task M100 #2226 최종 보고 — 셀 내 겹침 배치 다중 그림 flow 회계

- 이슈: #2226 (작업지시자 발견 + 편집기 조판부호 스크린샷 제공) / 브랜치: `local/task2226`
- 기간: 2026-07-12 / 시각 판정: **통과**
- 재현: `samples/basic/issue1994_behindtext_table_20200830.hwp` p2 우측 하단
  로고 표 (2×3, tac 어울림 — rowspan 로고 + 붓글씨 그림 3개 + 주소 블록)

## 결론

행높이 1.9× 팽창(표 하단 페이지 밖 865px → 2행 주소 블록 소실, 겉보기 rowspan
병합 실패)을 세 결함 축으로 분해·정정. 행높이 선언 정합(69.7/80.8px), 표 하단
743px, 주소 블록 3줄·붓글씨 위치까지 한컴 편집기 스크린샷(1차 정답지)과 일치.

## 원인 3축 (계측 귀속)

1. r0 +62.4px: 측정 `measure_non_inline_controls_height`가 같은 문단의
   TopAndBottom 그림 2개 flow를 직렬 합산 — 저장 ls 사다리(줄이 그림 아래로
   밀림)가 이미 배치를 반영.
2. r1 +60.7px: wrap bottom이 para_top(밀린 줄 vpos) + 그림 bottom을 이중 계상
   (rowspan 로고 셀) → 2-b deficit 배분.
3. 붓글씨 소실(정정 중 노출): 그림 앵커(#577 공식)가 밀린 줄 vpos 기점 —
   축소된 셀 클립 밖 이탈.

## 정정 (`05272fcf`)

1. wrap bottom para 기점 — 기하 판정(prev_extent+obj_bottom ≤ first_vpos)으로
   사다리 기반 문단 시작 사용 (layout/measurer 대칭).
2. 측정 저장 흐름 신뢰 — 3중 증거 가드(빈 문단 + vpos>0 + 비인라인 개체)
   보유 셀만 stored extent. **반례 캘리브 게이트 실측 2건으로 수렴**:
   KTX TOC(개체 없음 — additive가 한컴 쪽, golden 실측 884px) / #1282
   쪽영역제한 ON(텍스트 문단 vpos 0 — 한컴이 그림만큼 행 성장).
3. 밀림-빈문단 그림 앵커 = 문단 시작 (top_and_bottom/overlay/Square 공통).

하드코딩 없음 — 판정 근거는 저장 LINE_SEG/텍스트 유무/wrap·voff 스펙 필드.

## 게이트 + 검증

| 항목 | 결과 |
|------|------|
| fmt / clippy | 통과 / 0 |
| `--tests --no-fail-fast` | **3,049 / 실패 0** (golden 8/8 무변동, KTX·exam_kor·복학원서·#1282·#1486·#1748·#1858·주보 아크 보존) |
| 표적 테스트 신설 `tests/issue_2226_cell_flow_pictures_overlap.rs` | 수정 전 FAILED(272.7px) → ok |
| OVR 5샘플 (분리 폴더) | 회귀 0건 |
| 시각 판정 | **통과** — `output/poc/issue2226/compare_footer_3way.png` |

## 산출물

- 커밋: `016f5b25`(계획), `05272fcf`(정정 3건+테스트+2·3단계 보고)
- 문서: `working/task_m100_2226_stage2.md`, 본 보고서
- 연계: #577/#2207(앵커 계보), #2211/#2221(저장 흐름 신뢰 계보), 주보 아크 5건째
