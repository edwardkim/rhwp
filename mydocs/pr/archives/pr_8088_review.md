# PR #8088 리뷰 — P66 Skia 영역 사전 계획

## 최종 판정

**승인 — 명시한 통합 후보의 구현·집중 검사·실물 출력 검토 범위를 충족했다.**
원 contributor head는 `ef613b4f453955358a329580624f371ba9642482`, 실제 local 통합 code head는 `0b1c97075c0d2f7edb90dba781f9f7585bcfc627`다.
원 head의 녹색 CI와 통합 후보의 실제 실행을 구별한다. 원 PR을 단독 merge하거나 원격 approve한
상태가 아니다. 남은 조건은 사용자 원격 처리 승인, 통합 PR의 정확한 최종 head CI와 mergeability다.

## 접수와 의존 관계

- [원 PR](https://github.com/edwardkim/rhwp/pull/8088), contributor seo-rii, base devel, non-draft.
- 사용자 지시는 seo-rii PR 묶음 처리이며 후속 지시에 따라 Draft #7932는 제외했다.
- reviewer edwardkim을 requested_reviewers API로 지정·확인했다. source 변경 전 상태를 고정했다.
- 기본 작업공간의 `review/seo-rii-20261021`, base `f45878c7caafb43ca1fd017ddd93484a0acc6ad2`에서 P66 → P67 → P68 순으로
  기능/보정4개 commit을 원 저자 보존해 중복 없이 체리픽했다. `ef613b4f` → `6e9c598a7`.
- 충돌이 없었으며 최종 tree `fd971bc26bd0961b6123cadcaf61108057f0986a`는 원 P68 head와
  current base의 `merge-tree --write-tree` 결과와 같다. reviewer source·test·workflow 보정은 없다.
- [검토 계획](../../plans/pr_review_seo_rii_20261021.md). #536은 전체 멀티 렌더러 추적 이슈로 닫지 않는다.

## 구현 주장·실제 소비·독립 검사

공용 immutable plan은 floor/ceil 원점·출력 크기·교차 영역과 bitmap 예산을 할당 전에 계산하고 실제 영역 renderer가 같은 값을 소비한다. 기존 full-page device origin을 유지한다.

소비 경로: geometry와 raster options → `SkiaRegionPlan::new` → output/working/peak 예산 → renderer의 plan 기반 copy.

2²⁴ endpoint·16,384 axis·16M working pixels·128MiB working+output 경계, 비유한/퇴화 geometry·독립 output 제한·off-page working0을 코드와 검사로 대조했다. 새 숫자 예외나 좌표 clamp는 없다.

정식 `skia_region_plan` 8개와 기존 `issue_536_native_viewport` 8개가 통합38개 실행에 포함됐다. 실제 문서4개 ×3배율에서도 plan.output_bytes와 실제 full output 크기가 같고 음수/off-page 창의 crop이 일치했다.

작은 합성 계약은 독립 한컴 일치로 승격하지 않았다. 실제 API와 독립 Print의 전쪽 TSV·대표
PNG·측정 raw·조건·producer hash·초기 진단 실패와 정상 실행은 [공통 검증 기록](../assets/issue_536_seo_rii_20261021/README.md)에 있다.
실물 prepared/host/full 비교의 기준은 같은 입력의 독립 full-page raster이며, 한컴 비교는 별도 수행했다.

코드 위치: [plan 생산](../../../src/renderer/skia/region_plan.rs#L48), [동일 plan의 renderer 소비](../../../src/renderer/skia/renderer.rs#L667), [정식 경계 검사](../../../tests/cases/skia_region_plan.rs).

## 조판 원칙과 입력·시각 확인

저장 LineSeg 수용·재조판·문단/표 소속·줄 높이·pagination/rowspan/continuation 규칙을 변경하지 않았다.
해당 엔진의 컷·요구 높이 변경 검토는 비해당이다. 기존 PageLayerTree의 좌표·clip·순서·font를
같은 replay로 소비하며, 문서 ID나 특정 수치 조합으로 화면을 맞추지 않는다.

기존 HWP4개와 독립 Print4개의 commit blob·SHA를 확인했다. 모두1쪽이고 Native/WASM의 전쪽
최저 raw는99.63470% / 99.29125%, 새 host/prepared 최저는99.63470%다. 전체 base 대조8행의
최대 하락0.00000%p, Native base/후보의 네 문서 RGB가 모두 바이트 동일했다. font 예외나 baseline,
래칫·golden 완화, 신규 실물 px 핀은 없다. 상용 font bytes·비공개 corpus를 커밋하지 않았다.

목록·계속 줄·뒤 본문·네 style·baseline·간격을 Print와4배로 직접 판독했다. 그림 crop·두 그림의
위치와 그룹 도형의 외곽/gradient/그림자도 확인했다. pic-crop의 Print 검은 테두리와 rhwp의 가는
회색 테두리, 글자 AA/농도는 같은 base에도 남는 residual이다. 새로운 위치/내용 차이는 없다.
수식 개선이나 완전한 문서 fidelity를 주장하지 않는다. 원본을 보정한 작업이 아니므로 기존 실물
회귀의 기대값·px 핀을 변경하지 않았다. 기존 source path 참조의 #8109 관계 검사는 그대로 유지했다.

## CI 재사용과 이번 실행

원 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39281130203)와 변경 범위의 check는
실제 성공했다. 원 P68 head의 Full CI32 success·5 skipped, CodeQL·Render Diff 성공도 API로 확인했다.
GitHub Full CI 재사용 규칙에 따라 reviewer가 변경하지 않은 source의 광범위 Rust/Native 전체 회귀와
lint를 반복하지 않았다. current-base 자동 merge-tree 동등성과 새 base의 실제 출력 보존을 위에 연결했다.
이것을 미실행 통합 후보 CI의 성공으로 표시하지 않는다.

이번에는 통합 source에서 canonical 집중38 PASS, Native CLI/lib build, 실물4개 ×3배율의
새 API/이동 창·frame/counter 검사, cache off/on 성능9쌍, 실제 software Vulkan probe,
Native/WASM 및 실제 API의 전쪽 TSV·대표 PNG 판독을 수행했다. WASM 실행 코드는 이번 native-skia
변경에서 제외돼 기존 최적화 producer를 재사용했으며 현재 입력의 Canvas 출력은 다시 실행했다.
명령·SHA·표본 범위·초기 font budget 거부와 정상 재실행을 공통 증거에 기록했다.

## 남은 한계와 원격 조치

preflight 성공은 allocation/render 성공 보증이 아니다. Skia intermediates·resource cache·호출자 버퍼는 예산 밖이다. subtree culling이나 전체 RSS 한도를 주장하지 않는다.

검토 판정과 원격 승인은 별도다. 통합 PR 생성·push·GitHub review/comment·merge·원 PR close는
사용자 승인 뒤 해당 PR/head를 다시 확인해 수행한다. 오늘할일은 초기 조사 단계에서 갱신하지 않았다.

## Merge 후 contributor PR comment 계획

통합 PR의 실제 merge SHA·devel 포함·최종 CI와 이 review/asset 존재를 먼저 확인한다.
원 contributor의 단계별 기여와 원 head를 링크해 한국어 존댓말로 감사와 통합 사실을 알리고,
원 PR을 직접 merge한 것으로 표시하지 않는다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment),
전쪽 수치·판독·남은 차이·성능 측정 범위와 merge SHA 고정의 `host-pic-crop-01-review.png`,
`host-pic-crop-01-overlay.png`, `helvetica-styles-zoom.png`를 Markdown 이미지로 포함한다.
승인된 경우에만 UTF-8 body-file로 게시하고 API로 재조회한 뒤 원 PR을 통합 PR 링크와 함께 close한다.
Draft #7932·fork branch·#536은 보존한다.
