# PR #8144 — 제목 표와 본문 흐름의 완결 처리

## 최종 판정

**승인.** 원 기여 head `72181b51b0a734c23cad9c3f0a0f5649e391b150`을 보존하고 최신 devel과 통합했다. 최종 코드 `21ba44e3ea75966321d7184e2d94a8620e378013`의 필수 로컬 검증과 편집 저장본의 독립 Print 확인을 완료했다. 남은 merge 전 조건은 제출한 통합 PR의 최신 head CI 성공과 충돌 없음 확인이다. 사용자는 완전한 처리와 메인터너 보정·수용을 승인했다.

## 범위·승인·이력

- 사용자 지시: 2026-10-30 “8144 PR을 완전하게 처리해서 마무리”. 앞서 승인한 메인터너 충돌 보정과 수용 가능한 기여 PR의 머지·기여자 안내를 이어 수행한다.
- [원 PR #8144](https://github.com/edwardkim/rhwp/pull/8144): pblsketch, devel 대상, bug/layout/rendering/table, milestone 1. 원 head는 최신 devel과 충돌해 직접 머지할 수 없다.
- 기준 devel `168aaaf2bf5a4ef5ecc70a61087aeabf2d9d2433`; branch `review/pr8144-completion`.
- 원 head는 [#8121](https://github.com/edwardkim/rhwp/pull/8121)의 `5462965c8f7769bb6738814bf1eb0a90f12040b5`를 조상으로 포함한다. 같은 host의 Square/TAC도 필수 범위다. 다른 보류 #8269 코드는 포함하지 않는다.
- 범위 기록 `5054b02af9` → 기여 이력 보존 merge `2ca841aa3c` → 공통 행/원점 보정 `dba42989ba` → 각주 색 없음 보정 `21ba44e3ea`.
- 원 #8144/#8121은 별도 해결 대상 issue가 없는 공개 재현 PR이다. #7699/#7548은 참조 계약이며 이 처리에서 다시 close하지 않는다.
- 과거 누적 `review/pblsketch-20261030`는 다른 보류 작업 보존용으로 유지한다. #8145의 글꼴 예외를 확대 적용하지 않는다.
- maintainer 일반 경로, intake/local_validation/visual_fixture_evidence/rework_and_exceptions/post_merge를 적용했다. 원 기여의 첫 기여 수용은 #8165에서 이미 완료했다.

## 원인과 공통 조판 계약

| 주장 | 원본·독립 근거 | 생산 → 측정 → 배치 | 결과와 비적용 경계 |
|---|---|---|---|
| 같은 host의 Square 앵커와 TAC도 실제 줄 소속을 갖는다 | 원 control 순서, 저장 가로 폭 40816HU와 TAC 40819HU, 독립 Print의 2줄. 한컴 정상 재저장 전폭/10HU 여유 폭 대조군은 각각 2줄/1줄 | `typeset/inline_flow/plan.rs`가 공통 `ObjectPlacementFrame`의 배제 영역과 원점을 계산 → `paragraph_flow.rs`가 `InlineFlowPlan.occupied_end`로 fit하고 같은 plan을 commit → `layout/paragraph_layout.rs`가 그 행과 부동 표 확정 원점을 paint | 별도 host y 보정을 제거. dirty·합성·가변 폭 저장 줄은 프레임 보존의 근거로 쓰지 않는다. 캡션·중첩 표·셀 각주는 기존 소유자에게 반환 |
| 제목 아래 저장 원점과 빈 줄을 보존한다 | 실제 한컴 HWP/HWPX 저장 LineSeg, 원 사례 3개와 9개 선행 빈 문단, 독립 Print | `stored_page_square_preserves_source_origin`이 같은 물리 쪽의 유효 저장 줄과 제목 바닥 관계를 인증 → typeset이 원점 0과 host 줄 점유를 기록 → layout과 `HeightCursor`가 같은 원점을 소비 | 임의 8문단 상한 제거. 명시적 쪽/단 나눔, 되감김, dirty, 합성, 누락 줄, 본문 높이가 탐색 경계. #7699 상단 예약과 중복하지 않음 |
| 명시적 새 쪽의 TAC가 제목과 겹치지 않는다 | 한컴 정상 저장의 명시적 2쪽 문서: 빈 문단은 p1, 제목·TAC·본문은 p2 | `section/flow.rs`의 후속 TAC도 `vpos_snap_current_height_with_storage`를 소비 → `HeightCursor`가 인증된 새 쪽 원점을 중간 문단의 로컬 0과 구분 → fit/배치에 같은 높이 전달 | 원점을 paint에서 별도로 더하거나 출력 clamp로 겹침을 가리지 않음. 양쪽 실제 전체 2쪽과 내용 소유 확인 |
| 편집·되돌리기·재열기도 같은 계약을 유지한다 | 원 HWPX의 후속 표 host pi4, 선행 빈 문단 pi0, 같은 host pi0에 `검증 ` 삽입. 정확히 그 export 바이트를 독립 Print | `DocumentCore`/실제 브라우저 `HwpDocument`의 스냅샷·insert·export·restore API → 같은 저장 바이트 재열기 | 전역 session 편집 여부로 깨끗한 다른 저장 문단을 버리지 않음. 해당 문단의 dirty/합성 여부로 판정 |
| 각주 구분선의 명시적 색 없음을 보존한다 | 원 HWPX `noteLine color="none"`, 독립 Print의 선 부재. #1984 원 `4aeea2e90f` → 통합 `1bfeabcae6`은 RGB·여백만 보존했음을 확인 | 공통 HWPX ColorRef parser/serializer → `note_separator::has_ink`를 각주·미주 layout이 소비 | 선의 잉크만 없애고 예약 높이·위아래 여백·각주 내용 위치는 유지. 검정색 대조군은 선만 추가 |

분할 컷·rowspan·이어받기 계산은 변경하지 않았다. 새 행 경로는 전체 plan이 실제 예산에 들어갈 때 commit하며, 들어가지 않는 분할과 셀 각주 예약은 상태를 변경하기 전에 기존 소유자에게 반환한다. 기존 열 전환도 후보 plan 전체가 새 열 예산에 들어가는지 확인한다. 명시적 쪽 경계는 새 2쪽 원본, 기존 분할 경로는 전체 회귀로 확인한다.

## 시각 검증과 글꼴

최종 [전쪽 결과 표](assets/pr_8144/results-table.md), [TSV·대표 PNG·재현 명령](assets/pr_8144/README.md), [입력/출력 해시·한컴 작업·빌드 provenance](assets/pr_8144/provenance.json)를 보존한다. 원 기여의 입력/PDF를 유지하고, 추가 대조군은 [fixture 생성 기록](../../tests/fixtures/pr8144/README.md)에 연결했다.

실제 한컴 2024 Print에서 포함된 Haansoft Batang·KoPubBatangLight·KoPubDotumLight를 확인했다. 로컬 `HBATANG.TTF`를 먼저, KoPub 디렉터리를 다음으로 공급하고 선언한 나눔스퀘어 대체 환경을 base/후보·Native/WASM에 동일 적용했다. 전체 HWP 글꼴 디렉터리를 공급하면 기존 Native 64MiB 총 예산에 필요한 폰트가 밀려날 수 있어 최종 검증은 실제 필요한 약 59MiB로 제한했다. 글꼴 프로그램·등록 예산 코드는 변경하지 않았다. 로컬 글꼴 파일은 커밋하지 않았다.

원 PDF 제목은 고딕 글꼴이고 현재 한컴 Print는 바탕 대체 글꼴이다. 원 PDF 비교도 유지하며 새 Print로 점수를 대체하지 않았다. 현재 Print에서는 제목·회색 상자·글머리표·본문·각주를 같은 프로그램으로 대조한다. 확대 판독에서 발견한 `none` 구분선도 실제 보정했다. 수식 개체는 이 표본들에 없으며, 괄호·작은 기호·글머리표·각주 번호와 줄간격을 직접 확인한다.

## 직접 판독과 관계 회귀

최종 Native/fresh WASM의 제목·회색 TAC 상자·글머리표·괄호·각주 번호·본문/각주 줄간격을 독립 Print와 같은 영역에서 확인했다. 선행 9개 빈 문단은 실제 공간을 점유하고 명시적 쪽 나눔은 첫 쪽의 빈 문단과 둘째 쪽의 제목·본문을 구분한다. 편집 3경로는 삽입 글자와 뒤 표/본문을 보존하고, 각주의 불필요한 검정 구분선은 사라졌다. 제목의 세부 글자 폭·rasterization 차이는 남지만 표 경계·본문 시작·줄바꿈·각주 영역의 겹침이나 누락은 없다. 원 PDF의 고딕/현재 Print의 바탕 대체 차이도 별도 기록했다.

[변경 전후 관계 검사](assets/pr_8144/relations-before-after.md)는 실제 base/후보 RenderTree에서 영향 사례 10개가 양쪽 backend 모두 FAIL→PASS, 여유 폭 정상 대조군은 PASS→PASS임을 보여 준다. Rust 회귀를 base에서 실행했다고 주장하는 자료가 아니라 실제 출력의 독립적인 비겹침·순서 대조다. 정식 12개 회귀는 원본 속성·셀 내용·소유·순서·undo/재열기를 검사하며 설치 글꼴이나 임의 px 핀에 의존하지 않는다.

## 실제 검증 결과

- 집중 회귀 12건: PASS. 현재 suite manifest가 배정한 실제 모듈을 선택했다.
- 전체 `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 6 --no-fail-fast`: **12,062 passed, 45 skipped, 0 failed**. 실행 2,190초. 기존 nextest 제외 설정을 유지했다.
- Native Skia: root lib **3,858 passed, 12 ignored**, workspace lib 15/165/2 passed; missing picture 2 passed; direct PDF 14 passed.
- 최종 Native CLI와 최적화 WASM 빌드: PASS. 실제 Chrome Canvas에서 실행했으며 producer 해시는 provenance에 기록했다.
- 원 사례·경계·편집 재열기 14개 비교: 15쪽/경로, Native/fresh WASM 모두 전쪽 90% 이상·독립 PDF 쪽수 일치·누락 없음. 원 PDF 포함 최저 Native 94.76313%, WASM 94.55817%; 현재 한컴 Print 비교 최저 98.71947%/98.69053%.
- 기존 정상 대조군 7개: 13쪽/경로, 악화 없음. 실제 편집 3건도 각 1쪽/경로이며 PNG 재열기·undo·Native/WASM 저장 바이트 동일성을 확인했다.
- `cargo fmt --all -- --check`: PASS.
- Native/WASM/workspace all-target Clippy (`-D warnings`), workspace build, manifest check, `git diff --check`, 문서 링크 검사 3개: PASS.

로컬 빌드는 공용 `target/pr-review`, `CARGO_BUILD_JOBS=6`, release-test opt-level 1/codegen-units 64를 사용했다. `src`는 `21ba44e3ea` 이후 바뀌지 않았다. Studio 소스·패키지·workflow는 변경하지 않았으므로 Studio 전용 테스트나 배포를 추가하지 않았다. 실제 WASM 브라우저 출력·편집 검증은 위와 같이 완료했다.

초기 dba 전체 회귀는 6,987건 통과 후 위 각주 보정을 위해 SIGINT로 중단했다. 실행 중이던 6건의 signal 종료를 제품 실패 또는 통과로 계산하지 않는다. 첫 최종 집중 실행도 prepare 후 suite가 재배정되어 0건 선택되어 종료됐으므로 통과 근거에서 제외했다. 현재 manifest의 suite 021/025로 12건을 실제 실행했다. 이후 그 배정으로 전체 12,062건을 실제 통과했다. 최종 측정 주석의 파일 바이트 수 변화 때문에 마지막 manifest 검사에서 자동 suite 배정 drift가 발생했다. 정식 `--prepare`로 재생성하고 모듈 1,872개의 집합 동일성, 실행 대상으로 발견된 12,062건과 전체 PASS 기록의 논리 테스트 이름 집합 동일성(누락·추가 0)을 확인했다. 새 배정의 실행 파일 컴파일·집중 9+3건·workspace all-target Clippy·fmt·manifest를 통과했다. 코드나 테스트 동작은 바뀌지 않아 36분의 전체 실행을 중복하지 않았다. 테스트 집합 해시는 provenance에 남겼다.

## Self-review

공통 조판 원칙의 적용 대상은 저장 원점과 Square/TAC 행 계획이다. 입력의 유효 저장 폭·vpos → 공통 frame/plan → fit/commit → 실제 paint 소비를 대조했고, 별도 backend y 보정 대신 확정된 원점을 공유한다. 문서 ID·8문단 상한·출력 감추기 분기는 사용하지 않는다. 빈 문단은 글자가 없어도 줄 공간을 소유하며, 여유 폭 정상 대조군은 같은 줄을 유지한다. 새 전체 plan이 예산에 맞지 않는 경로는 기존 분할 소유자에게 반환한다.

메인터너 보정은 기존 #7699의 상단 예약, #7548의 저장 줄/부동 표 계약과 Storage 오류 전달을 유지한다. 각주 색 없음은 공통 ColorRef를 왕복 보존하고 각주·미주에서 같은 잉크 판정을 쓴다. 저장본과 실제 편집본을 각각 독립 Print로 비교했으며, 새 회귀는 시각 증거 확보 후 추가했다. 현재 대상의 기존 절대 px 핀은 관계 검사로 바꿨고 소스 줄 메트릭·바깥여백에서 얻는 독립 계약은 유지했다.

이 검토는 메인터너 자신의 통합 보정에 대한 self-review다. 원 PR의 과거 CI나 작성자 캡처로 최종 후보 검증을 대신하지 않는다. 최종 local gate와 제출 head CI를 아래 기록 및 GitHub 상태로 확인한 뒤 merge한다.

## Merge 후 contributor PR comment 계획

원 기여의 제목 아래 원점·빈 문단 보존 성과를 먼저 인정하고, 최신 devel 통합·공통 행 계획·9개 빈 줄과 명시적 쪽 경계·각주 색 없음 보정을 설명한다. 기여 이력을 보존한 통합 PR과 실제 merge SHA, 로컬/CI 결과, 전쪽 수치와 직접 판독 결론을 존댓말로 전달한다.

[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고 아래 asset을 실제 merge SHA로 고정해 Markdown 이미지로 게시한다.

- `mydocs/pr/assets/pr_8144/candidate/{native,wasm}/same-host/review/review_001.png`
- `mydocs/pr/assets/pr_8144/candidate/{native,wasm}/nine-empty-break/review/review_002.png`
- URL: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/<asset-path>`

merge 뒤 원 #8144/#8121의 merged 상태와 조상 포함을 확인한다. active review archive 이동·오늘할일 운영 기록·devel 동기화 뒤 `--body-file`로 게시하고 API로 한글·URL·중복 여부를 확인한다. 완료 증적은 정식 경로에 보존하고 root `output/` 전체 및 유휴 `target/pr-review/debug`를 정리한다.
