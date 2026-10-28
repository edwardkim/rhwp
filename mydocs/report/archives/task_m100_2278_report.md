# Task M100 #2278 최종 결과보고서 — C2b: 3D 입체·ofPie 보조플롯 렌더 (#1431 Track C)

- 이슈: #2278 (OPEN — close는 메인테이너 판단)
- 브랜치: `local/task2278` (fork push 완료)
- 계획: `task_m100_2278.md`/`_impl.md` (v1) → `task_m100_2278_v2.md`/`_impl_v2.md`
  (투영 모델 재계획)
- 작성일: 2026-07-20

## 1. 목표와 결과

C2a까지의 "코퍼스 28종 placeholder 0건"에서 남은 시각 갭인 **3D 입체감**과
**ofPie 보조플롯**을 실측 기반으로 정합했다. 추가로 시각판정 과정에서 원형
계열 **슬라이스 밀착**(흰 테두리 제거)과 **쪼개진원형 explosion**(작업지시자
편입 승인)을 반영했다.

| 갭 | 결과 |
|---|---|
| 3D 막대 입체 | view3D 파싱 + 시어 투영(rAngAx=1) — 두께·깊이·방(축·격자·틱) 전부 유도식/실측 |
| 3D 원형 | 타원비 `sin(rotX)·cos(persp/2°)` + 하반부 측벽 `rx×0.207` (실측 캘리브레이션) |
| ofPie 보조플롯 | 주 원(결합 슬라이스 실측 초록 [4]) + 보조 원/누적 막대 + serLines(검정·접선) |
| 팔레트 #5 | `DEFAULT_PALETTE[4]` = **#27A172** (정답지 2파일 교차 실측), 하늘 [5] 강등 |
| 슬라이스 밀착 | 원형 계열 6개소 흰 테두리 제거 (정답지 원주 전수 스캔 흰 run 0건) |
| 쪼개진원형 | 계열 `c:explosion`(%) 파싱 + 중심각 방향 오프셋 렌더 (코퍼스 25%) |

## 2. 진행 경과 (커밋)

| 단계 | 내용 | 커밋 |
|---|---|---|
| 1 v1~v3 | 오블리크 고정 근사(눈대중 상수) — 투영 모델로 대체됨 | d525828d·668d50f4·087af357 |
| 재계획 | 설계 리뷰 3렌즈 → 수행/구현계획서 v2 (투영 모델) | a91e4afb·1dd5a69d |
| 1R | view3D 파싱 + ShearProj + render_bars_3d/value_grid_3d | f60c71c4 |
| 1R v2 | 방 선 처리 한컴 정합(#808080/0.75 균일·값/카테고리 틱·바닥 무채움) | 9b3ba8dd |
| 2 | 3D 원형 — 타원+측벽, 타원+벽 블록 fit | a7720b40 |
| 3 | ofPie 보조플롯 + 팔레트 #5 + 레이아웃/시작각/serLines 실측 캘리브레이션 | bd6ddebd |
| 3 v2 | 원형 계열 슬라이스 밀착 (시각판정 확정) | d720f28e |
| 3 v3 | 쪼개진원형 explosion (작업지시자 편입) | fb89b5bd |

핵심 실측 기록은 각 단계 보고서(`task_m100_2278_stage{1,1_v2,1_v3,1r,1r_v2,2,3}.md`)에
정리. 정답지 PDF의 차트가 **2702×1577 비트맵 임베드**라는 사실을 확인하고
`pdfimages` 추출 → 픽셀 실측(색·기하·각도)을 전 단계 캘리브레이션 방법으로 사용.

## 3. 변경 면적

`src/ooxml_chart/{mod,parser,renderer}.rs` + 통합 테스트
`tests/issue_2278_chart_3d_ofpie.rs` — 공통 레이아웃/문서 코어/저장 경로 무접점
(OLE blob 보존 계약 불변). 2D 막대/라인/분산/주식 경로 무접촉, 2D 원형은
explosion 부재 시 출력 불변.

## 4. 검증 (최종)

- 단위: `cargo test --lib ooxml_chart` **134 passed / 0 failed**
  (Stage1R 110 → 틱/스타일 3 + pie3d 7 + ofPie 파서 4·렌더 7 + 밀착 가드 1 +
  explosion 2)
- 통합: `issue_2278_chart_3d_ofpie.rs` 3 (3D 막대 8케이스·pie3d·ofPie 2종) +
  기존 차트 스위트(1453/1882/2277 3종/2129/1431_scatter) 전부 무회귀
- 전수: `cargo test` **exit 0, 269 스위트 전부 ok, 3,259 passed / 0 failed**
- `cargo clippy --all-targets -- -D warnings` 무경고, fmt 수정 파일 한정
- 시각판정: **작업지시자 통과 4회** — 1R v2(3D 막대 4종), Stage 2(3차원원형),
  Stage 3(ofPie 2종), v2/v3(밀착·쪼개진원형 — localhost studio + WASM 재빌드로
  브라우저 확인). 대조 자료 `output/poc/chart_c2b/compare/` 8종
  (한컴 임베드 원본 vs rhwp 동일 스케일 합성).

## 5. 알려진 편차 / 잔여 (범위 밖)

- 한컴 3D 측벽·벽면의 **그라데이션 음영**은 평면 음영(±0.25 shade)으로 근사
- perspective 배율 cos(persp/2°)·시어 fit은 코퍼스 카메라(30/30, 15/20) 1점
  캘리브레이션 — 타 카메라 실샘플 확보 시 재검
- dPt 단위 explosion, splitType val/percent/custom, 3D ofPie: 코퍼스 부재
- 플롯/범례 가로 배분·제목 크기 차이: 기존 승인 규격 유지
- rotX=0 막대(시어 폴백)·음수각 카메라(클램프): 실샘플 확보 시 확장

## 6. 후속

- upstream devel PR 제출 (`Refs #2278` — 이슈 close는 메인테이너 판단)
- #1431 트래킹 갱신 제안: C2b 체크. Track C 잔여 후보: 콤보 라인 누적/마커,
  `c:smooth`/`c:size` 세부, 하단 가로 범례 순서 실측, 음수값 실누적
