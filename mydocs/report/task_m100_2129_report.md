# task_m100_2129 최종 결과보고서 — C1d 라인 누적(stacked/percentStacked) + 표식 렌더

- 이슈: #2129 "C1d: 라인 누적(stacked/percentStacked) + 표식 렌더 (#1431 Track C)"
- 브랜치: `local/task2129` (local/devel 분기)
- 기간: 2026-07-09 ~ 2026-07-10
- 수행/구현 계획서: `mydocs/plans/task_m100_2129.md` / `task_m100_2129_impl.md`

## 1. 목표 / 결과

`render_line`이 `c:grouping`을 인식하지 않아 누적꺽은선형·표식이있는누적꺽은선형·
백프로기준누적꺽은선형(3종)이 독립 선으로 오렌더되던 결함(C1a 막대 누적의 라인 대칭)을
해소하고, 작업지시자 승인으로 편입된 표식(마커) 렌더까지 구현했다.

**결과: 라인 5종 전부 한컴 2022 정답지 정합 — 작업지시자 시각판정 통과 (2026-07-10).**

| 샘플 | 종전 | 이후 |
|------|------|------|
| 누적꺽은선형 | 독립 선, 개별값 축 0~6 | 누적합 위치, 축 0~15 step 5 (정답지 일치) |
| 백프로기준누적꺽은선형 | 독립 선 | 0%~100% step 20%, 최상위 계열 100% 수평선 |
| 표식이있는누적꺽은선형 | 독립 선, 무마커 | 누적 + 계열별 마커 ◆■▲ |
| 표식이있는꺽은선형 | 무마커 | 마커 렌더 (같은 플래그로 함께 정합) |
| 꺽은선형 | — | 축·형상 불변 + x 슬롯 중앙 정합 개선 |

## 2. 설계 결정

1. **`line_grouping`·`line_markers` 별도 필드** (`OoxmlChart`) — 콤보(bar+line 공존)에서
   단일 `grouping` 공유 시 XML 문서 순서에 따른 상호 오염 차단. 파서 단위 테스트로 고정.
2. **plot 레벨 `<c:marker val>` 파싱** — 게이트 `cur_plot_type==Line && cur_series.is_none()
   && val 존재`. 계열 내부 `<c:marker>` 래퍼(val 없음)는 자연 배제, 신규 ParseState 불요.
3. **축·누적 정책은 `render_bars` 미러** — stacked=`nice_axis(0, max category_positive_sum,
   3칸)` / percent=(0,100,20) 고정 + `render_value_grid` percent 라벨. 새 축 기계장치 없음.
   `nice_axis(0,12.3,3.0)`=0~15 step 5로 정답지 실측과 정확 일치.
4. **값공간 누적** `cum[i] += v.max(0.0)` (음수 clamp 막대 동일), percent 합≤0→0% 가드.
5. **마커 사이클 ◆■▲+원 폴백** (`push_line_marker`, `hwp-chart-marker` 클래스) —
   정답지 실측 형상. 크기 근사(r=3.5/3.0)는 시각판정 통과.
6. **x 슬롯 중앙 배치** (시각판정 피드백 반영) — 아래 4.1.

## 3. 단계별 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 모델 필드 2개 + 파서 grouping 분기·marker arm + 단위 6건 (기존 고정 테스트 반전) | 80ed7b15 |
| 2 | `render_line` 누적/백프로 기하+축 + 기하 단위 6건 | 441fadd2 |
| (fmt) | stage3 테스트 코드 rustfmt 정리 (포맷 전용 분리 커밋) | 6ccf294e |
| 3 | `push_line_marker` + 배선 + 마커 단위 4건 | 4d2a03b7 |
| 4 | 통합 가드 10파일 + 시각판정 피드백(x 슬롯 중앙) 반영 | ba0a2d62 |

전 단계 TDD(RED 확인 → GREEN) 준수. 변경 면적: `src/ooxml_chart/{mod,parser,renderer}.rs`
+ `tests/issue_2129_line_stacked.rs` (공통 모듈·`render_combo`·축 헬퍼 무변경).

## 4. 검증

- 단위: `cargo test --lib ooxml_chart` **72/72** (파서 grouping/marker/콤보 무오염,
  누적 기하/축/NaN 가드/무회귀 핀, 마커 수/형상 사이클/기본값, 슬롯 중앙 배치).
- 통합: `tests/issue_2129_line_stacked.rs` **6/6** — 라인 5종 × hwp/hwpx **10파일**
  (placeholder 부재, 축 라벨, 마커 수 12, 무회귀 핀). hwp=hwpx 동일 결과.
- 전수: `cargo test` **exit 0, 231개 스위트 전부 ok, 2,995 passed / 0 failed**
  (수정본 기준 클린 로그). `cargo clippy --all-targets -- -D warnings` 무경고.
  fmt는 수정 파일 한정(무관 diff 없음).
- 시각: `output/poc/chart_c1d/` SVG·PNG 10개 ↔ `pdf/chart/라인/{stem}-2022.pdf` 대조
  (stage4 보고서 대조표) + 스튜디오(:7700, WASM 재빌드) — **작업지시자 시각판정 통과**.

## 4.1 시각판정 피드백 반영 (2026-07-10)

**피드백 #1 — 라인 x 배치가 한컴과 다름 (가로로 꽉 참).** 한컴은 라인 점을 카테고리
슬롯 중앙에 배치(첫/끝 점이 반 슬롯 안쪽)하는데 종전 구현은 양끝을 플롯 가장자리에
고정. 샘플 XML의 `c:crossBetween val="between"` 신호로 근거 확인 후, 카테고리 라벨이
쓰던 슬롯 중앙 공식(`px + cat_span×(i+0.5)`)으로 통일. 수행계획서 Non-goals("x 배치
불변")는 작업지시자 지시로 범위 보정. 비누적 기존 2종도 함께 정합 개선. TDD
(`test_line_points_at_category_slot_centers`) + 재산출 + 재시각판정 통과.

## 5. 알려진 편차 / 잔여 (범위 밖)

- **범례 순서 역전**(누적류: 한컴은 계열3→1): C1c에서 관찰 상충으로 **C2 기이관** — 불변.
- **범례 스와치에 마커 글리프 없음**(한컴은 선+마커): 경미, C2 fidelity 후보.
- **`line3DChart` 미라우팅**: 코퍼스 27종에 없음. Unknown→placeholder 폴백 유지, 갭 기록.
- `c:smooth`(코퍼스 전부 0)·계열별 `c:symbol`/`c:size` 세부, 음수값 실누적(Excel식),
  콤보 라인 누적/마커: 범위 외 유지.

## 6. 후속

- upstream devel PR 제출 (`Refs #2129`; 이슈 close는 메인테이너 판단).
- #1431 트래킹 갱신 제안: C1d 체크 + 잔여 C2(stock HLC + fidelity: 범례 순서/스와치
  마커 글리프/3D 입체/특이케이스 0.5 축 간격/scatter 마커 모양 + line3D 라우팅).
