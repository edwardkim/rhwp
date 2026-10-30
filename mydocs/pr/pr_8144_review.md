# PR #8144 — 제목 표 아래 본문·빈 문단의 완결 처리

## 최종 판정

**메인터너 보정 후 수용 가능 — 보정·재검증 진행 중.** 2026-10-30 사용자가 이 PR의 완결 처리를 지시했다. 앞서 승인한 충돌 보정과 수용 가능한 PR의 머지·기여자 안내 범위를 이어서 수행한다. 아래 미실행 항목을 완료로 판정하지 않는다.

## 고정 범위와 처리 경로

- 원 PR: https://github.com/edwardkim/rhwp/pull/8144, pblsketch, head `72181b51b0a734c23cad9c3f0a0f5649e391b150`.
- 최신 base: `168aaaf2bf5a4ef5ecc70a61087aeabf2d9d2433`.
- 기본 작업 branch: `review/pr8144-completion`. 과거 누적 `review/pblsketch-20261030`는 다른 보류 작업·기준 PDF의 보존용이며 통째로 반영하지 않는다.
- maintainer 일반 경로와 intake/local_validation/visual_fixture_evidence/rework_and_exceptions/post_merge를 적용한다. 첫 기여 수용은 #8165에서 이미 완료했다.
- #8144는 #8121 head `5462965c8f7769bb6738814bf1eb0a90f12040b5`를 조상으로 포함한다. 같은 host의 Square/TAC 배치도 필수 검증 범위다. #8269 등의 코드는 포함하지 않는다.
- 원 기여 이력을 보존해 최신 devel와 통합하고 메인터너 보정을 분리한다. 로컬 필수 검증을 마친 통합 head를 일반 PR로 제출하고 최신 GitHub CI·self-review 뒤 merge한다. 원 PR의 merged 상태와 기여자 안내까지 확인한다.
- #8145의 기존 Native 제목 누락 예외를 이번 PR에 확대하지 않는다. 원본 교체·임의 좌표 보정·허용치 완화로 통과시키지 않는다.

## 확인한 문제와 독립 근거

1. 원본 3개는 한컴 2024로 실제 저장한 문서이며 원 PR에 독립 Print PDF가 있다. 선행 빈 문단은 1000HU 줄 높이와 600HU 간격으로 점유하고 제목 host·TAC 문단이 뒤따른다.
2. 같은 host의 #8121 원본은 저장 줄 하나에 Square anchor와 TAC 표가 함께 있으나, 독립 한컴 Print와 실제 재저장본은 anchor 줄과 표 줄을 나누어 배치한다. 기존 누적 후보의 단순 배제 영역 회피는 표와 뒤 본문이 한 줄 높은 상태였다. 현재 문서의 control 순서·실제 너비와 공통 줄 계획을 추적한다.
3. `stored_first_line_clears_page_square_table`의 8문단 제한은 물리 쪽 경계가 아니다. 저장 줄·명시적 쪽/단 나눔·dirty 상태의 유효한 경계로 판정해야 한다. 정상 한컴 저장을 거친 9개 선행 빈 문단·명시적 쪽 나눔 대조군을 준비한다.
4. 최신 devel의 #7699 상단 예약, #2019 빈 문단 계약, #7548 어울림 흐름, #8174의 공통 줄/원점·Storage 전달을 보존한다. 기존 #8165·#8145도 정상 대조군으로 확인한다.

현재 base Native 재출력: same-host 51.06666%, title-inline 42.35117%, 빈 문단 HWP 44.94916%, HWPX 47.02465%; 각 1쪽. 원 사례는 아직 해결되지 않았다. base 최적화 WASM도 완료했다. 같은 순서로 51.10469%, 42.06504%, 44.88125%, 46.77153%이며 각 1쪽이다.

## 완료 조건

- 현재 base + 원 기여 코드의 충돌을 계약에 따라 보정하고, 줄 구성·점유 끝·실제 배치가 같은 결과를 소비하도록 수정한다.
- 원본 4개, 적용/비적용 경계, 편집·저장·재열기, 관련 정상 대조군을 실제 실행한다. 편집은 같은 저장본의 독립 Print를 확보한다.
- 전쪽 Native/fresh WASM 90% 이상·PDF 쪽수 일치·base 대비 2%p 초과 하락 없음 및 직접 확대 판독을 충족한다. 관련한 기존 회귀가 실패하면 원 해결 PR·보정·독립 증거를 먼저 대조한다.
- 시각 조건을 충족한 뒤 정식 관계형 회귀를 보완하고 수정 전 FAIL/후 PASS를 확인한다. 임의 문단 수와 px 핀을 기대값으로 삼지 않는다.
- Rust lint 전 묶음, 전체 release-test, Native Skia 3종, 최적화 WASM·실제 브라우저 시각 검증을 완료한다. 새 보정 head에는 원 PR의 과거 녹색 CI로 필수 검증을 대신하지 않는다.
- 입력·PDF·TSV·대표 PNG와 정확한 source/build를 정식 경로에 반영한 뒤 PR 제출·CI·머지·후속 안내·정리를 완료한다.

작업 중 로그·진단은 `output/pr-review/pr8144-completion/`에만 보관한다. 구현의 실제 소비 경로와 각 검증 결과는 완료 시 이 문서에 이어 기록한다.

## 메인터너 보정 진행

- 범위 기록 `5054b02af9`, 원 기여 통합 `2ca841aa3c`. 원 head `72181b51b0`와 최신 devel 이력을 모두 보존했다. `layout.rs`의 서로 다른 helper와 원점 유지 조건을 함께 살렸고, `flow.rs`의 Storage 전달, `vpos.rs`의 현재 source height cursor·후속 원점 연결을 보존했다. 아직 원격에 게시하지 않았다.
- 같은 문단의 Square 앵커와 TAC는 `inline_flow::plan_with_storage`의 원본 control 순서·공통 행 계획으로 처리한다. `typeset/inline_flow/plan.rs`에서 동일한 `ObjectPlacementFrame`으로 배제 영역과 부동 표 원점을 확정한다. `paragraph_flow.rs`가 `occupied_end`로 수용하고 `plan.end`로 전진한다. `layout_inline_flow_plan`은 같은 행의 TAC box와 확정 부동 표 원점을 소비한다. 캡션·중첩 표·각주 또는 전체가 들어가지 않는 분할은 기존 소유자로 반환하며, 원 기여의 별도 host 보정 계산은 공통 행 계산으로 대체한다.
- 같은 host 원본의 실제 저장 줄 폭은 40816HU, 용지 여백 계산 폭은 40819HU다. TAC 폭 40819HU를 후자로만 비교하면 앵커와 한 줄로 합쳐진다. 정상 저장본의 일정한 가로 프레임은 보존하되 줄 소속은 공통 계획에서 다시 계산한다. dirty·합성·가변 폭 줄은 이 저장 프레임의 근거로 쓰지 않는다. 독립 한컴 정상 저장의 전폭 대조군은 2줄, TAC를 10HU 좁힌 대조군은 1줄로 출력한다. 이 대조군은 원 사례를 대체하지 않는다.
- 8문단 상한은 제거했다. 명시적 쪽·단 나눔, 저장 줄 되감김, 본문 높이 밖, dirty·합성·누락 줄이 실제 탐색 경계다. 첫 줄이 제목 바닥에서 시작하거나, 첫 앵커 줄은 원점 0에 있고 다음 줄이 제목 바닥에서 시작한다는 저장 증거로 원점 유지 여부를 정한다. 문서 전체의 편집 플래그 대신 해당 저장 문단의 유효성을 확인한다.
- 첫 보정 Native: 원본 3개 93.95562/94.29047/93.89490%, 9개 선행 빈 문단 97.76588%. 두 번째 보정에서 같은 host 원본은 96.98633%, 정상 한컴 전폭/여유 폭 대조군은 97.76588/97.76974%다. 명시적 쪽 나눔 뒤 원점 보정과 최종 WASM은 검증 중이다.
- #7699 HWP/HWPX/좁은 표, #7548 저장/재조판, #8145 cellzone, #8165 머리말·꼬리말은 첫 보정 Native에서 base와 같은 수치다. #7699 최저 99.07819%, #7548 99.81020%, cellzone 99.54886%, 머리말·꼬리말 91.11882%다. 최종 head의 재검증을 대신하지 않는다.

### 글꼴 판독

기본 환경의 회색 상자에서 글머리표 TextRun은 렌더 트리에 있고 위치도 셀 내부다. 설치한 나눔스퀘어의 `U+2022`는 cmap 항목은 있으나 `glyf`의 contour 수가 0이다. 같은 파일은 Linux와 Windows 폰트 디렉터리에 있다. 기준 same-host PDF의 실제 포함 face는 Haansoft Batang·KoPubBatangLight·KoPubDotumLight다. 확인한 로컬 `ttfs/hwp/HBATANG.TTF` 공급과 명시적 나눔스퀘어→Haansoft Batang 환경의 진단에서는 글머리표가 나타나고 98.58014%다. 이 설정은 원본·기본 렌더러를 바꾸지 않는다. 양쪽 base/후보와 Native/WASM에 같은 선언을 공급한 별도 검증 및 글꼴 해시·독립 Print 출처를 최종 증거에 보존한다.

### 명시적 쪽 나눔과 편집 경계

명시적 새 쪽의 Square 앵커가 옆 차선의 vpos 0 줄을 소유할 때, 뒤 TAC의 저장 첫 줄은 제목 바닥에 있다. `place_paragraph_flow_with_storage`가 TAC에도 공통 `vpos_snap_current_height_with_storage`를 호출하고, `HeightCursor`는 명시적 새 쪽·인증된 0 원점·유효한 저장 줄인 경우를 중간 문단의 로컬 0과 구분한다. 같은 커서 결과를 TAC fit와 확정 배치가 소비한다. 임의 높이를 더하거나 paint에서 겹침을 가리지 않는다. Native 수정 전 본문 쪽 40.50489%, 보정 후 97.78154%이며 첫 빈 쪽은 100%다. 전체 2쪽이 독립 Print와 같다.

실제 `DocumentCore`에서 뒤 본문(pi4), 선행 빈 문단(pi0), 같은 host(pi0)에 각각 `검증 `을 삽입했다. 현재 Native에서 각 1쪽이며 undo PNG는 해당 before PNG와 바이트가 같다. 실제 내보낸 HWPX의 한컴 Print 작업은 대기/실행 중이며, 이를 포함한 최종 Native/WASM 비교와 관계형 회귀 보완은 다음 검증 단계에서 완료한다. 진단 프로브는 현재 Native 빌드의 `release-test/deps/librhwp.rlib`를 명시해 연결했다.
