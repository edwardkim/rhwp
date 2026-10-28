# Task #7280 Stage 55 — R5 상태 소유·구역 조정 책임 묶음

- 이전: [Stage54](task_m100_7280_stage54.md).
- 승인 범위: [구현계획 §4.1](../plans/task_m100_7280_impl.md#41-r3r5-책임-묶음-진행으로-전환)의 R5 전체.
- 시작 head: `e641b74a7`, 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: **2026-09-23 R5 책임 묶음 구현·검증 완료**. R6 및 최종 제출 게이트는 남아 있다.
- 검증 제품 SHA: `7947ee45fa560eab3dd6f2d6a7af2ce94f759e88`.
  원격 push·PR·댓글과 주 checkout의 Studio `pkg` 갱신은 하지 않았다.

## 상태 소유와 문단 중심 호출 순서

`state::TypesetState`가 비공개 data 필드로 `data::StateView`를 소유한다. 외부에는 불변 Deref만
제공하며 DerefMut·가변 필드 접근자는 없다. 문단/표/컨트롤/각주 조정자는 확정 항목 추가,
흐름 전진, 커서 기록, 예약 명령을 호출한다. 읽기 관측면은 각 조회 국면의 최신 상태를 제공한다.
읽기 모델은 새 IR이나 복제 snapshot이 아니며, 넓은 필드 관측면을 제공한다는 한계는 남긴다.

| 소유자 | 책임 | 외부 경계 |
| --- | --- | --- |
| state/data | 기존 상태 필드·수명 | 읽기 전용 관측; 외부 직접 쓰기 불가 |
| state/transition | 생성·쪽/단 flush·advance·reset | 기존 전이 순서 유지 |
| state/notes | 각주 projected 예산 Query·실제 예약 Command | 기존 notes/reservation 이동, 구분선·footer 예약 불변 |
| state/commands | 항목·흐름·커서·배치 결과 반영 | fit 정책을 다시 판단하지 않음 |
| state/finalize | 빈 꼬리·머리말/꼬리말·쪽 번호 확정 | state 내부에서 페이지 변경; 기존 private 테스트도 같은 구현 호출 |
| section | 구역 설정·원본 문단 순회·종료 | 공개 facade 및 resumable API 유지 |

문단 순회는 지연 표 선행 flush/저장 넘침 관측 → 명시 경계 → 저장 경계 → 빈 꼬리 흡수 →
어울림 흡수 → 쪽 보장/제목 보호 → 본문·표 배치 → 앵커·vpos 후처리 → 컨트롤 순서다.
`entry`의 None과 `tail`의 true는 원래 문단 루프 continue를 뜻한다. tail의 내부 탐색
루프 continue는 함수 반환으로 바꾸지 않았다. 컨트롤 뒤 이전 문단 인덱스 갱신도 그대로다.
구역 종료는 지연 그림 → 미주 → 지연 표 → 단 flush → 후행 흡수 부착 → 빈 꼬리/쪽 번호
확정 → 상태 소비 순서를 유지한다. `section.rs`에 실제 호출 순서가 있다.

WASM 재개 경로는 `table/continuation/job.rs`의 생성·완료에서 같은 상태 소유자와
`into_result`를 사용한다. 결과 필드의 소유권을 이동하며 페이지 전체를 복제하지 않는다.
페이지 종료 구현도 state 안에 두어 state가 section 조정자를 역호출하지 않게 했다.

남은 의존은 명시적으로 보존한다. state는 R1–R4의 배치 결과 타입을 소비하고, section은
TypesetEngine의 기존 측정/format facade를 호출한다. 엔진의 compatibility profile Cell과
측정용 scratch 수명을 재설계하지 않았으며 모든 코드가 순수 함수라는 주장은 하지 않는다.
기존 조판 예외의 타당성·IR 개편·표준 의미 규칙 재정의는 이번 변경에 포함하지 않는다.

## 이동·명령 의미 대조

증거 경로의 공통 접두사는 `output/7280/r5-batch/`다.

- `structural-proof.json`: 기존 상태 메서드와 명령 캡슐화 후 구역 단계 추출 본문
  **118/118 일치**. data 접근·경로·format 차이와 명시한 반환 경계를 정규화했다.
  이 검사는 캡슐화 전후 명령 의미 전체의 단독 증거가 아니다.
- `command-proof.json`: 호출 인자를 대입해 펼친 명령/이동 함수 **27/29 자동 일치**.
  나머지 두 곳은 의도적인 소유 경계 변화여서 별도로 확인했다. 미주 간격 갱신은 기존
  문단/마지막 줄 존재 및 skip 조건을 유지하고, 그 뒤 같은 값을 소유자에게 적용한다.
  resumable 종료는 같은 finalize 함수와 같은 결과 필드 이동을 호출한다.
- 중간 `check-1.log`의 E0594/E0596은 직접 쓰기를 컴파일러가 거부한 증거다.
  잔여 쓰기·추출 입력·경로를 정리한 최종 head에서는 빌드/Clippy가 통과했다.
- 기존 private 쪽 마무리(감추기) 테스트의 호출 경로 한 줄만 새 소유 위치로 연결했다.
  테스트 모듈·이름·assertion·기대값·baseline·ignore 및 IR/public API는 변경하지 않았다.
- 미주 좌표의 `-=`를 음수 `+=`로 치환하지 않고 별도 명령에 원래 연산을 보존했다.
  중간 `fad203507` 빌드는 중단하고 `prior-fad203507/`에 로그를 보존했다.
  아래 결과는 모두 최종 `7947ee45f`에서 다시 실행한 것이다.

## 검증 결과

검증 worktree는 clean 확인 후 재사용한 `/home/edward/mygithub/rhwp-review-7280-r3k`다.
고정 base 대비 정책 검사이며 파생 suite/manifest는 커밋하지 않았다.

| 검사 | 결과·증거 |
| --- | --- |
| fmt·고정 base manifest | 통과, 1,382 sources / 28 suites + 20 exceptions; fmt.log, manifest.log |
| 고정 base unit-tier | 4,205 tests / 298 modules / cfg support 28; unit-tier.log |
| Native Clippy -D warnings | 통과; clippy-native.log |
| 누적 집중 | **395 passed / 0 failed**; nextest-focused.log |
| 전체 회귀 | **10,096 passed / 0 failed / 기존 제외 50**; nextest-full.log |
| 기준본과 회귀 대조 | 통과 테스트 이름 10,096개 동일; regression-comparison.json |
| Native | 12문서 **353쪽** dump-pages·RenderTree·SVG 동일; native-comparison.json |
| fresh Docker WASM | 빌드 성공, 9문서 **329쪽** RenderTree·SVG 동일; build-wasm.log, visual-wasm-comparison.json |
| 선택 PNG | **19쪽** 전후 및 Native/WASM 동일; visual-{native,wasm}-comparison.json |
| 좌표·클리핑 후보 목록 | 9문서 × 13종 동일; ledger-comparison.json |

실제 호출 경계의 기존 계약도 연결했다. `table_continuation_does_not_reapply_page_hide`는
이어받은 표 쪽에 원 앵커의 감추기를 다시 적용하지 않음을, `test_measure_endnote_advance_side_effect_free`는
미주 Query의 상태 비변경을 확인한다. `issue2424_resumable_pagination_commits_only_after_final_fragment`와
`issue2424_resumable_delete_commits_only_after_final_fragment`는 재개 중간/최종 commit 수명을 검사하며
동일 전체 로그에서 모두 통과했다. 새 테스트를 복제하거나 페이지 수 검사로 대체하지 않았다.

전체 회귀 요약: `Summary [ 454.816s] 10096 tests run: 10096 passed (5 slow), 50 skipped`.
Native Cargo는 순차 실행했다. nextest 버전/observation 설정 경고는 기존과 같다.
Chrome 시작 실패는 재시도하여 해소했고 실패 로그도 보존했다. 테스트 실패로 집계하지 않는다.

`provenance.json`, `native/{base,head}/provenance.json`, `targets.json`에
입력/PDF/바이너리·도구·쪽을 기록했다.
Native standalone CLI를 테스트 빌드 전에 `bin/head`로 고정해 이후 nextest 재링크와
구분했다. 실제 캡처용 CLI SHA-256은 `cfbdf26808f79d4b2dc7568961db87b53de45501ae263549fdd0abb803c637b7`다.
fresh WASM SHA-256은 `2259ae2d48efa31826498c6f018b4da91560dd95ada6b1a8fd86e8ff0383b8b5`이며 review worktree의 pkg를 사용했다.
기준 CLI/WASM은 고정 baseline의 해시를 확인해 재사용하되 이번 출력·PNG는 새로 생성했다.

## 직접 시각 확인과 판정 범위

대표 compare·standalone overlay·review를 직접 확인했다. 어울림 본문과 후속 문단,
표/각주 쪽 경계, 다단 미주/제목/그림 배치가 이전과 같음을 확인했다.
연구 보고서 76쪽은 기준본·변경본 compare를 함께 열어 각주 102와 표 뒤 영역을 대조했다.

- `native/head/square-body/square-body/compare/compare_001.png`
- `native/head/square-host/square-host/overlay/overlay_001.png`
- `native/head/square-table/square-table/review/review_005.png`
- `native/{base,head}/deferred-picture/deferred-picture/compare/compare_076.png`
- `native/head/endnote-2022-09/endnote-2022-09/compare/compare_012.png`
- `native/head/endnote-between20/endnote-between20/overlay/overlay_022.png`
- `wasm/head/endnote-zero/endnote-zero/review/review_016.png`
- `wasm/head/endnote-no-separator/endnote-no-separator/review/review_016.png`

기존 PDF 대비 글꼴·수식·표 선 차이, between20 22쪽의 줄 소유/세로 위치 차이와
no-separator 16쪽 하단 줄 이월 차이는 보존되었으며 개선으로 보고하지 않는다.
**판정은 리팩토링 동작 보존 충족**이다. 모든 353쪽의 시각 전수 판독이나 한컴 피델리티
완성을 주장하지 않는다. ledger의 단일 SVG 파일 카운트 한계도 기존과 같으므로 총 쪽수는
dump-pages와 실제 export 파일 집합으로 검증했다.

## 재현과 다음 범위

`validate.sh`: review head 고정 → prepare → fmt → 고정 base 정책 → Native Clippy →
standalone CLI build/보존 → 누적 집중. 이후 영향 시각 확인 뒤 `run-full.sh`로 전체 회귀를 실행한다.
Docker는 review worktree에서 `docker compose -p rhwp --env-file .env.docker run --rm --no-deps
-e CARGO_BUILD_JOBS=4 -e BINARYEN_CORES=4 wasm`을 실행했다.
출력 생성은 `native.mjs base/head`, `visual-retry.mjs base/head native/wasm`,
`visual.mjs base/head ledger`와 compare 스크립트를 사용한다.

`typeset.rs`는 R5 시작 **12,035줄 → 7,902줄**이다. 상태/호출 경계의 코드가 늘었으므로
전체 LOC·CC 감소를 주장하지 않는다. **R5는 완료했으며 다음은 R6 구조 정본·기여자 규칙
관리 지도·재계측**이다. 최종 제출용 WASM/workspace Clippy·workspace build·Native Skia
등 PR 전체 게이트와 실제 제출 base 갱신 검증은 아직 남아 있다.
