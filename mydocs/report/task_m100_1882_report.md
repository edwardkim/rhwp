# task_m100_1882 최종 결과보고서 — C1c 차트 스타일 4갭 보정

- 이슈: **#1882** (C1c, #1431 Track C 하위)
- 브랜치: `local/task1882` (from `local/devel` = upstream/devel a433b0d8)
- 마일스톤: M100
- 상태: 구현·검증 완료, 작업지시자 시각판정 대기 (studio dev 서버 기동)

## 1. 목표 / 결과

렌더되는 전 차트 종류(25종, stock 2종 제외)에 공통이던 한컴 정답지 대비 **스타일 4갭**을
보정했다. 핵심 발견: 샘플 XML에 스타일 값이 사실상 없어(범례 제외) 본 작업은 XML 파싱이
아니라 **한컴 기본 스타일 재현**이었다.

| 갭 | 종전 | 이후 (= 한컴 2022 정답지 실측) |
|---|---|---|
| ① 제목 | 미렌더 | 자동 제목 **"차트 제목"** 상단 중앙, regular weight(400) |
| ② 팔레트 | 녹색-우선(#70AD47…) | **#6183D7 파랑→#FE813B 주황→#B0B0B0 회색→#FCD801 노랑** (PDF 픽셀 실측) |
| ③ 범례 | 하단 가로 고정 | `c:legendPos="r"` 파싱 → **우측 세로 스택** (플롯 세로 중앙) |
| ④ Y축 | max가 데이터에 붙음 | **경계 headroom + step 기반 눈금** — 막대 5.0→0~6 라벨 0,2,4,6 / scatter Y 4.0→0~5 / X 2.6→0.5 간격 유지 |

## 2. 설계 결정

- **④ `nice_range` → `nice_axis(min,max) → (min',max',step)`**: 데이터 max가 step 경계에
  정확히 걸리면 +1 step 확장 후 step 재계산(ceil-nice), 비경계면 step 유지. 이 조건부
  규칙이 실측 앵커 3점을 동시에 만족하는 유일한 단순 규칙. 격자 전용 함수 분리는 막대
  기하와의 범위 불일치(플롯 초과) 위험으로 기각 — 반환 통합 후 전 호출처 일괄 이관.
  percentStacked는 (0,100,20) 명시 전달로 종전 출력과 동일(무회귀).
- **① 모델 플래그 방식**: `has_title_elem`/`auto_title_deleted` 추가, `chart.title`은 명시
  텍스트 전용 유지 → 파서의 빈 차트 조기 반환 가드 무변경 (placeholder 회귀 원천 차단).
- **② `DEFAULT_PALETTE`만 교체**(앞 4색 실측, 이후 유추 — 주석 구분). `scheme_color`는
  의도적 무변경 (schemeClr=문서 테마 참조 의미, 코퍼스 미사용, 변경 시 기존 테스트 무근거 파손).
- **③ `LegendPos` default=Bottom**: legend 없는 XML·모델 직접 구성 테스트는 현행 하단 유지.
  Right만 우측 세로 신설(Left/Top 하단 폴백 — 코퍼스 전 샘플 r). 좁은 차트(w×0.30<50px)는
  하단 폴백 — clamp min>max 패닉 가드 (자동 보안 리뷰 지적 → 즉시 수정, 6d0829d7).

## 3. 단계별 요약

| 단계 | 내용 | 커밋 |
|---|---|---|
| Stage 1 | 갭② 팔레트: DEFAULT_PALETTE 한컴 실측값 교체 | `73dcb560` |
| Stage 2 | 갭④ 축: nice_axis + render_value_grid step화 + 앵커 3점 테스트 | `9caf475f` |
| Stage 3 | 갭① 자동 제목: 모델 플래그 + autoTitleDeleted 파싱 + weight 400 | `58ba5b80` |
| Stage 4 | 갭③ 범례: LegendPos + legendPos 파싱 + 우측 세로 배치 | `0bd80319` |
| — | Stage 4 보완: 좁은 차트 clamp 패닉 가드 | `6d0829d7` |
| Stage 5 | 통합 테스트 4건 + 전체 회귀 + 27종 시각 산출 | `d476a1dc` |

수정 파일: `src/ooxml_chart/{mod,parser,renderer}.rs` + 신규 `tests/issue_1882_chart_style_gaps.rs`.
shape_layout 등 배선 무수정. 전 단계 TDD(선실패 확인 후 구현). 무관 rustfmt churn 없음.

## 4. 검증

```
cargo test --lib ooxml_chart                     → 45 passed (기존 32 + C1c 13)
cargo test --test issue_1882_chart_style_gaps    → 4 passed  (4갭 회귀 가드)
cargo test --test issue_1431_scatter             → 1 passed  (placeholder 0건 무회귀)
cargo test --test issue_1453_chart_3d_ofpie_routing → 2 passed (percent 축 가드 포함)
cargo test (전체)                                 → 전부 통과 (exit 0)
cargo clippy --all-targets -- -D warnings         → 경고 0
```

- 기존 테스트 갱신 1건뿐: `test_render_scatter_decimal_axis_labels` `2.4`→`2.5`
  (X축 0.6→0.5 간격 — 의도된 스펙 변경, 구현계획서 명기).
- **시각검증**: 27종 hwpx 전수 `output/poc/chart_c1c/all/` SVG+PNG 산출. 막대(묶은/누적/백프로)·
  라인·원형·분산형 대표 확인 — 4갭 전부 정답지 정합. 작업지시자 판정용 **WASM 재빌드 +
  studio dev 서버 기동** (studio는 #1456 수정 포함 트리 — 차트 첫로드 공백 없음).

## 4.1 시각판정 피드백 반영 (2026-07-04)

- **가로막대 좌측 카테고리 라벨 잘림** (작업지시자 studio 판정 지적): `left_pad`가 항상 값축
  숫자 라벨 폭(2자≈32px) 기준이라, 좌측이 카테고리 라벨("항목 1"≈40px+)인 가로막대에서
  라벨이 차트 밖으로 잘리던 기존 결함. 가로막대는 `estimate_category_label_width`
  (최장 카테고리×10px, 상한 차트 폭 35%)로 계산하도록 수정 + 회귀 테스트
  (`test_horizontal_bar_category_labels_not_clipped`). 가로막대형 5종 해소.
- **축 눈금 밀도가 방향별로 다름** (작업지시자 "3차원 세로막대형 y축 범위 다름" 지적 →
  정답지 5종 추가 실측으로 정책 해독): 같은 데이터(합 12.3)가 **세로 누적 0~15 step 5 /
  가로 누적 0~14 step 2**, 묶은가로(5.0)는 **0~6 step 1 유지**(headroom 후 step 재계산
  없음). → `nice_axis`에 `target_ticks` 도입: **세로 값축 3칸 / 가로 값축·scatter 5칸**,
  경계 headroom은 step 유지(재계산 로직 제거). 실측 9앵커(묶은세로/누적세로/묶은가로/
  누적가로/백프로/꺽은선/scatter X·Y) 전부 정확 일치. 단위 테스트 3종 + 통합 앵커 추가.
- **3차원 계열의 고유 축은 잔여 편차로 기록**: 한컴 3D 엔진은 묶은 0~5(무헤드룸)/누적
  0~20(과헤드룸)으로 2D와 또 다름. 3D는 2D 근사 렌더(C1a 명시 범위)이므로 축도 2D 정책
  적용 — 3D 고유 축 정합은 **C2(3D 입체감)와 함께 이관**. (이번 수정으로 3D 누적은
  0~14→0~15로 정답지 20에 근접, step은 5로 일치.)
- **가로막대 카테고리 아래→위 배치** (작업지시자 승인 후 반영): 한컴은 항목 1을 맨
  아래에 배치. `render_bars`/`render_category_labels`의 가로 분기에 `cat_slot`(역순
  인덱스) 도입 + 회귀 테스트. 가로막대형 전 종(2D/3D, 묶은/누적/백프로) 반영.
- **3D 축 정책 반영** (작업지시자 "3차원묶은가로 0~5" 지적 → 3D 정답지 4종 전수 실측):
  한컴 3D 엔진은 **묶은 3D = 세로·가로 모두 0~5(촘촘 눈금 + 무헤드룸)**, **누적 3D
  세로 = 0~20(2D 15 + 1 step)**, 누적 3D 가로 = 2D와 동일(0~14). 파서가 3D 여부를
  버리고 있어 모델에 `is_3d` 플래그 추가(bar3DChart/pie3DChart) + `render_bars` 3D
  분기 + `nice_axis_no_headroom` 변형. 실측 4앵커 전부 일치, 단위 3 + 통합 앵커 추가.
- **누적 세로 막대 x 좌표 결함 수정** (실측 검증 중 발견): 누적/백프로 세로 막대의
  x가 `plot_x` 대신 `plot_y` 기반으로 계산돼 전체가 (plot_y−plot_x)만큼 수평 이동
  (코퍼스에서 ~17px 우측 — C1a Part B부터 잠복, 카테고리 라벨과 미세 불일치).
  `cell` 계산을 방향별 기준축으로 교정 + 라벨 중심 정렬 회귀 테스트.
- 관찰된 추가 편차(범위 밖, C2 후보로 기록): 한컴은 일부 차트(세로 누적·묶은가로 등)
  에서 범례 항목 순서를 역전 — 관찰이 상충해(누적가로는 정순) 규칙 미해독, 27종 전수
  실측 필요. **C2 이관** (작업지시자 합의 2026-07-04).

## 5. 알려진 편차 / 잔여 (범위 밖)

- 콤보 보조축(실측 앵커 없음): max 10 → 축 0~15 (50% headroom, 과확장 수용).
- stock 2종 placeholder 유지(**C2**), scatter 시리즈별 마커 모양(다이아몬드/사각형 — C2 fidelity),
  3D 입체감·ofPie 보조플롯(C2), 레거시 `src/ole_chart/` 스타일 갭(#1251 후속).
- 팔레트 5번째 이후 색은 유추(코퍼스 최대 4시리즈 — 실측 불가, 주석 명기).

## 6. 후속

- 본 보고서 승인 + 작업지시자 시각판정 후 origin(fork) push → upstream `devel` PR
  (`Refs #1431`, `#1882`).
- #1431 트래킹: C1c 체크 갱신은 PR merge 후. 잔여 Track C = **C1d 라인 누적** / **C2 stock+fidelity**.
