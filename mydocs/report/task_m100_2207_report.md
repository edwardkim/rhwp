# Task M100 #2207 최종 보고 — 셀 내부 앵커 그림(글앞으로) vert=Para 기준점 오류

- 이슈: #2207 (#2189 처리 중 작업지시자 발견) / 브랜치: `local/task2207`
- 기간: 2026-07-11 / 시각 판정: **통과** (작업지시자, 한컴 2022 PDF 3-way 대조)
- 재현: `samples/basic/issue1994_behindtext_table_20200830.hwp` p1 헤더 표 제목 셀
  전화 금지 픽토그램 (wrap=글앞으로, vert=Para off=91)

## 결론

셀 내부 오버레이(글뒤로/글앞으로) 그림의 세로 앵커를 **앵커 문단 시작(첫 LINE_SEG
vpos) 기준**으로 정정. 픽토그램이 줄 높이(24px)만큼 하강 + 하단 클립 절단되던
증상이 해소되어 한컴과 1px 이내 정합 (잉크 top/bottom 66/117 vs 한컴 65/117).

## 원인

Task #577이 `TopAndBottom + vert=Para` 셀 내부 그림의 동일 증상을 정정했으나
가드가 TopAndBottom 한정 — 오버레이 wrap + Para 그림은 compose 후 전진된
`para_y`(앵커 문단 한 줄 아래)를 앵커로 사용. 산술 예측(87.56 − 24.0 = 63.56)이
가드 확장 후 렌더 y와 정확히 일치해 확정. 같은 함수의 Shape 분기는 이미 wrap
무관 앵커-시점 기준이었다 (그림 분기만 누락).

## 정정

[table_layout.rs](../../src/renderer/layout/table_layout.rs) `overlay_para`
(BehindText|InFrontOfText + vert=Para) 조건 신설, #577 앵커-시점 기준 공유.
+13줄 단일 지점, Square 등 다른 wrap·후속 가드 비접촉. 하드코딩 없음.

## 게이트 + 검증

| 항목 | 결과 |
|------|------|
| fmt / clippy (release-test, all-targets) | 통과 / 0 |
| `cargo test --profile release-test --tests` | 3,044 / 실패 0 |
| 표적 테스트 신설 `tests/issue_2207_cell_overlay_picture_anchor.rs` | 수정 전 FAILED(y=87.6) → 수정 후 ok |
| golden svg_snapshot | 8/8 무변동 |
| OVR baseline 5샘플 (±2px, 분리 폴더) | 회귀 0건 — exam_science(#577 원 재현 파일) 포함 |
| WASM 빌드 (Docker) | 통과 — pkg/ 산출 (#2189+#2207 포함) |
| 시각 판정 | **통과** — `output/poc/issue2207/compare_3way_pictogram.png` |

## 산출물

- 커밋: `7abb104f`(계획), `ddcf3103`(2단계 진단), `36dea20f`(정정+테스트+3단계)
- 문서: `working/task_m100_2207_stage{2,3}.md`, 본 보고서
- 연계: #2189(발견 경위, 동일 재현 파일), #577(전례), #2206(폰트 메트릭 보완축)
