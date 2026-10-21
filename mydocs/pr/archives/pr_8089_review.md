# PR #8089 리뷰 — P67 bounded Native page snapshot cache

## 최종 판정

**승인 — 명시한 통합 후보의 구현·집중 검사·실물 출력 검토 범위를 충족했다.**
원 contributor head는 `be6c8558fbf98e378e8e3c846a6d0f39a88d7152`, 실제 local 통합 code head는 `0b1c97075c0d2f7edb90dba781f9f7585bcfc627`다.
원 head의 녹색 CI와 통합 후보의 실제 실행을 구별한다. 원 PR을 단독 merge하거나 원격 approve한
상태가 아니다. 사용자의 원격 처리 승인을 받았고 통합 PR 후보 CI도 성공했다. 남은 조건은 정확한 최종 trailing head CI와 mergeability다.

## 접수와 의존 관계

- [원 PR](https://github.com/edwardkim/rhwp/pull/8089), contributor seo-rii, base devel, non-draft.
- 사용자 지시는 seo-rii PR 묶음 처리이며 후속 지시에 따라 Draft #7932는 제외했다.
- reviewer edwardkim을 requested_reviewers API로 지정·확인했다. source 변경 전 상태를 고정했다.
- 기본 작업공간의 `review/seo-rii-20261021`, base `f45878c7caafb43ca1fd017ddd93484a0acc6ad2`에서 P66 → P67 → P68 순으로
  기능/보정4개 commit을 원 저자 보존해 중복 없이 체리픽했다. P66 뒤 `be6c8558` → `10636fcc5`.
- 충돌이 없었으며 최종 tree `fd971bc26bd0961b6123cadcaf61108057f0986a`는 원 P68 head와
  current base의 `merge-tree --write-tree` 결과와 같다. reviewer source·test·workflow 보정은 없다.
- [검토 계획](../../plans/pr_review_seo_rii_20261021.md). #536은 전체 멀티 렌더러 추적 이슈로 닫지 않는다.

## 구현 주장·실제 소비·독립 검사

immutable N32 image가 renderer/tree 이후에도 픽셀을 소유하며, 같은 session key의 warm 요청은 tree 구성과 paint replay를 건너뛴다. 새 surface 전에 이전 image를 해제하고 보관/출력 예산을 검사한다.

소비 경로: session revision/font generation/page/scale → 한 개 `PreparedSkiaPage` → warm read_region → stamped NativeFrame.

`did_mutate` → `clear_caches`, font 공급의 validate/construct 성공 뒤 generation 증가와 invalidate, 명시 clear/limit 변경, page/scale key를 실제 소비까지 추적했다. rejected/no-op font·invalid region·off-page·oversized bypass의 기존 frame 보존과 eviction 순서를 대조했다.

정식 `issue_536_native_region_cache` 13개를 통합38개 실행으로 통과했다. 실물4개 ×3배율의 prepared output·4개 moving/음수/off-page window가 full replay의 crop과 같았다. 실제 session의 cache off/on 교차9쌍에서 pixels·replay21→1·warm hit0→20도 확인했다.

작은 합성 계약은 독립 한컴 일치로 승격하지 않았다. 실제 API와 독립 Print의 전쪽 TSV·대표
PNG·측정 raw·조건·producer hash·초기 진단 실패와 정상 실행은 [공통 검증 기록](../assets/issue_536_seo_rii_20261021/README.md)에 있다.
실물 prepared/host/full 비교의 기준은 같은 입력의 독립 full-page raster이며, 한컴 비교는 별도 수행했다.

코드 위치: [세션 key·hit·eviction](../../../src/native_host/region_cache.rs#L113), [N32 snapshot과 읽기](../../../src/renderer/skia/prepared_page.rs#L76), [편집 무효화](../../../src/native_host/mod.rs#L184), [폰트 성공 뒤 무효화](../../../src/native_host/fonts.rs#L123).

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

원 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39281911402)와 변경 범위의 check는
실제 성공했다. 원 P68 head의 Full CI32 success·5 skipped, CodeQL·Render Diff 성공도 API로 확인했다.
GitHub Full CI 재사용 규칙에 따라 reviewer가 변경하지 않은 source의 광범위 Rust/Native 전체 회귀와
lint를 반복하지 않았다. current-base 자동 merge-tree 동등성과 새 base의 실제 출력 보존을 위에 연결했다.
원 head의 재사용 근거와 새 통합 PR의 CI 성공은 아래에 분리해 기록했다.

이번에는 통합 source에서 canonical 집중38 PASS, Native CLI/lib build, 실물4개 ×3배율의
새 API/이동 창·frame/counter 검사, cache off/on 성능9쌍, 실제 software Vulkan probe,
Native/WASM 및 실제 API의 전쪽 TSV·대표 PNG 판독을 수행했다. WASM 실행 코드는 이번 native-skia
변경에서 제외돼 기존 최적화 producer를 재사용했으며 현재 입력의 Canvas 출력은 다시 실행했다.
명령·SHA·표본 범위·초기 font budget 거부와 정상 재실행을 공통 증거에 기록했다.

## 통합 PR의 실제 CI와 승인

[통합 PR #8135](https://github.com/edwardkim/rhwp/pull/8135)의 정확한 후보
`c1fff8ce3d5cb1b048a53f04fda16942175760c7`에서 [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39613689693), [CI](https://github.com/edwardkim/rhwp/actions/runs/39613689636), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39613689798), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39613689949), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39613689632)가 모두 실제 성공했다.
CI preflight는 `fast_pass=false reason=no-green-build-candidate`였으며,
원 fork CI를 통합 PR CI로 대신하지 않고 새 source branch의 Full CI를 완료했다.
마지막 완료 이벤트의 정책 audit는 GitHub API가 Archive B를 늦게 반영해 pending이었다.
기존 테스트를 재실행하지 않고 [정책 집계 job](https://github.com/edwardkim/rhwp/actions/runs/39617769583)만
재실행해 최신 성공 상태로 갱신했다.
사용자는 push·Open PR 생성·CI 성공 후 병합·원 PR3개 종료와 후속 comment를 승인했다.
이 기록 뒤 변경은 허용된 mydocs 문서뿐이다. 최신 trailing head의 실제 fast-pass·required
check와 최신 base의 merge-tree를 확인한 뒤 승인된 병합·후속 처리를 수행한다.

## 남은 한계와 원격 조치

기본8MiB·최대64MiB의 image 하나를 보관한다. 측정한 요청당 중앙값은 off0.38–1.89ms/on약0.04ms이며 첫 paint·window 표시·FPS 또는 장시간 RSS 보증이 아니다. C ABI/wire 변경과 GPU cache는 없다.

사용자 승인 뒤 통합 branch push와 Open PR #8135 생성을 완료했다. 아직 통합 merge·원 PR close·후속 comment는 수행하지 않았다. 원 PR을 GitHub에서 별도 approve한 것으로 표시하지 않는다. 오늘할일은 통합 후보 CI 성공 뒤 이 trailing 기록과 함께 갱신했다.

## Merge 후 contributor PR comment 계획

통합 PR의 실제 merge SHA·devel 포함·최종 CI와 이 review/asset 존재를 먼저 확인한다.
원 contributor의 단계별 기여와 원 head를 링크해 한국어 존댓말로 감사와 통합 사실을 알리고,
원 PR을 직접 merge한 것으로 표시하지 않는다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment),
전쪽 수치·판독·남은 차이·성능 측정 범위와 merge SHA 고정의 `host-pic-crop-01-review.png`,
`host-pic-crop-01-overlay.png`, `helvetica-styles-zoom.png`를 Markdown 이미지로 포함한다.
승인된 경우에만 UTF-8 body-file로 게시하고 API로 재조회한 뒤 원 PR을 통합 PR 링크와 함께 close한다.
Draft #7932·fork branch·#536은 보존한다.
