# task_m100_2277 최종 결과보고서 — C2a stock HLC 렌더 + 2D fidelity 정합

- 이슈: #2277 "C2a: stock HLC 렌더 + 2D fidelity 정합 (#1431 Track C)"
- 브랜치: `local/task2277` (local/devel 분기)
- 기간: 2026-07-15
- 수행/구현 계획서: `mydocs/plans/task_m100_2277.md` / `task_m100_2277_impl.md`
- 후속 이슈: C2b(#2278) — 3D 입체·ofPie 보조플롯 (본 작업 완료 후 착수)

## 1. 목표 / 결과

코퍼스 마지막 미지원 종류였던 stock 2종(고가저가종가·시가고가저가종가)의 placeholder를
해소하고, C1c/C1d에서 이관된 2D fidelity 갭 4건(범례 순서·범례 스와치 글리프·scatter
마커 모양·특이케이스 0.5축)을 정합했다.

**결과: 코퍼스 28종 기준 "차트 (미지원)" placeholder 0건 (#1431 Track C 완료기준의
렌더 커버리지 축 달성) — 작업지시자 시각판정 통과 (2026-07-15).**

| 영역 | 종전 | 이후 |
|------|------|------|
| stock 2종 | "차트 (미지원)" 점선 박스 | HLC: 고저선+종가 ▲ / OHLC: 캔들(하락 채움·상승 테두리)+종가 ×, 축 0~80 (정답지 일치) |
| scatter 마커 | 전부 원(r=3) | 계열 사이클 ◆■▲× (정답지: 계열1 ◆/계열2 ■) |
| 범례 순서 | 전부 정순 | 실측 규칙: 역순 = (세로 값축 && 누적/백프로) \|\| (가로막대 && 묶음) — 28종 예외 0 |
| 묶은가로 슬롯 | 계열1이 슬롯 맨 위 | 계열1 맨 아래 (정답지 정합, 범례 역순과 시각 일치) |
| 범례 스와치 | 색 사각형/선만 | 표식 라인=선+글리프(—◆—), 표식만 분산형=글리프, stock 시/고/저=빈 스와치·종가만 글리프 |
| 특이케이스 축 | 0~5 step 1 | 0~5 **step 0.5** (1카테고리 가로막대 게이트) |
| line3DChart | Unknown→placeholder | Line+is_3d 방어 라우팅 (입체는 C2b) |

## 2. 설계 결정

1. **마커 경로 헬퍼 `marker_path`/`push_marker` 추출** (1단계) — 반경 파라미터화 +
   class 분리(`hwp-chart-marker`/`hwp-legend-glyph`)로 scatter·stock·범례가 같은
   사이클 인프라 공유. ◆■▲ 출력 바이트 보존, 4번째 원 폴백→**×**(OHLC 종가 실측,
   stroke 기반).
2. **stock 계열 역할 = XML 순서 규약** (3계열=고/저/종, 4계열=시/고/저/종 — 코퍼스
   `c:order` 실측). C1b가 예약한 `SeriesData` 합타입은 **도입하지 않음** — 파서·콤보
   전반 파급 대비 코퍼스 이득 없음. 그 외 계열 수는 `render_line` 폴백(placeholder
   재발 방지).
3. **stock 전용 무조건 +1 step 축 헤드룸** — 데이터 max 59 → 한컴 0~80 step 20.
   `nice_axis`(경계 조건부)로는 0~60이라 부족, 3D 누적세로 "+1 step" 패턴과 동형.
   새 축 기계장치 없음.
4. **`SeriesMarker`{NotSpecified/None/Auto/Named}** — 계열 내부 `<c:marker>`/`<c:symbol>`
   상태 보존. 종가 판별(Auto)·시/고/저 억제(None)가 **데이터 주도**, 색·형상은 기존
   팔레트/사이클 폴백이 자동 정합(신규 색 상수 0개).
5. **`legend_order_reversed` 단일 결정 함수** — 28종 전수 실측표(stage3 보고서) 기반
   규칙을 한 곳에 격리(우측 범례 게이트, 콤보/이중축/stock 정순, 하단 범례 미실측 →
   현행 유지). C1c "관찰 상충"은 상충이 아니라 시각 상→하 정렬 규칙이었음.
6. **`SwatchKind` 5형** — Square/LineOnly 출력 바이트 보존(issue_1882 필터 보호),
   글리프 인덱스는 원 계열 인덱스(역순 나열과 무관하게 플롯 마커·팔레트 정합),
   Blank는 텍스트 오프셋 유지(stock 라벨 좌정렬).
7. **0.5축 좁은 게이트** — `가로 && 1카테고리`(비누적·비3D 분기 내)만 step 절반.
   단일 샘플 근거 명기, 코퍼스 27종(전부 4카테고리) 회귀 반경 0.

## 3. 단계별 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 마커 인프라(× 교체·`marker_path`/`push_marker`) + scatter 마커 사이클 | 450dbd8c |
| 2 | stock 2종 (파서 stockChart 계열 arm + `render_stock`) + `issue_2277_stock.rs` | 106ce71d |
| 3 | 범례 순서 규칙(28종 전수 실측표) + 묶은가로 슬롯 반전 + `issue_2277_legend_order.rs` | b9dc9139 |
| 4 | 범례 스와치 글리프 (`SwatchKind`) + 통합 가드 보강 | 991c66da |
| 5 | 특이케이스 0.5축 게이트 + line3DChart 라우팅 + `issue_2277_mini_chart_axis.rs` + 시각판정 산출 | 5e5a3e5e |

전 단계 TDD(RED 확인 → GREEN) 준수. 변경 면적: `src/ooxml_chart/{mod,parser,renderer}.rs`
+ 신규 통합 테스트 3파일 (공통 레이아웃/문서 코어/저장 경로 무접점 — OLE blob 보존
계약 불변).

## 4. 검증

- 단위: `cargo test --lib ooxml_chart` **95/95** (파서 stock/marker_symbol/line3D·
  렌더러 stock 축/고저선/캔들/마커·범례 규칙 진리표 9케이스·스와치 6종·0.5축).
- 통합: `issue_2277_stock.rs` 3 / `issue_2277_legend_order.rs` 3 /
  `issue_2277_mini_chart_axis.rs` 1 — stock 4파일·범례 16파일·특이케이스 2파일.
  기존 차트 4스위트(2129/1431_scatter/1882/1453) 무회귀 (issue_2129 마커 12개·
  issue_1882 Square 10×10/3D 축 앵커/scatter 0.5 라벨 전부 불변).
- 전수: `cargo test` **exit 0, 266개 스위트 전부 ok, 3,189 passed / 0 failed**
  (최종본 클린 로그). `cargo clippy --all-targets -- -D warnings` 무경고.
  fmt는 수정 파일 한정(무관 diff 없음).
- 시각: `output/poc/chart_c2a/` SVG 56개(28종×hwp/hwpx) ↔ `pdf/chart/{종류}/*-2022.pdf`
  대조(stage5 보고서 대조표) — **작업지시자 시각판정 통과 (2026-07-15)**.

## 5. 알려진 편차 / 잔여 (범위 밖)

- **OHLC 하락 캔들 채움색 #404040 근사** — 시각판정 통과 수준, 픽셀 정밀값은 후속
  여지. scatter 마커 r=4.5·범례 글리프 r=3.0/3.5도 근사(판정 통과).
- **세로 1카테고리 미니차트** — 미실측, 0.5축 게이트 미적용(가로만).
- **하단 가로 범례의 역순 여부** — 코퍼스 미실측(전 샘플 legendPos=r), 정순 유지.
- **3D 입체감·ofPie 보조플롯·팔레트 5번 교정** — **C2b(#2278)**.
- 콤보 라인 누적/마커, `c:smooth`·`c:size` 세부, 음수값 실누적: 범위 외 유지.

## 6. 후속

- upstream devel PR 제출 (`Refs #2277`; 이슈 close는 메인테이너 판단).
- #1431 트래킹 갱신 제안: C2a 체크 + 잔여 C2b(#2278) — 3D 입체(막대 압출·원형
  타원+측벽) + ofPie 보조플롯(팔레트 5번 실측 교정 포함).
