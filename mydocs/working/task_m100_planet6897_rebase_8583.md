# planet6897 통합 검토 — PR #8583·#8203 반영

## 리베이스와 중복 대조

2026-11-04, 검토 브랜치 `review/integration-20261103-planet6897-open`을
기존 devel `1924d371d6757392d063b8dc2e9bbe9a298be817`에서
새 devel `96827b3accd98ad0b80f76260b42fba24eded504` 위로 리베이스했다.
원 PR 39건의 범위는 유지하며 새 외부 PR을 추가로 체리픽하지 않는다.

PR #8583의 `a0f490fa955dfa1fc4a8b7b075fbaffff664d38c`와 로컬 `eae46acddb`는
PostScript 전용 글꼴 목록 보정으로, patch-id `a43892f42ca2005c9ef2cef3b95973a69c22dc5f`가
같다. 구현·검사·분석 문서 3개도 devel과 바이트가 같아 로컬 중복 커밋을 제외했다.

`a4fb27f651`(리베이스 전 `8912bfab73`)은 **편람 검사에 명시한 글꼴 환경 전달**이라는
다른 변경이다. 당시 devel `96827b3acc`는 그 함수에 `font_environment=None`을 전달하고 선택 인자도
없으므로, 커밋 이력뿐 아니라 실제 코드 대조로 미포함을 확인했다. 사용자가 제시한 조건인
‘같은 내용이 devel에 있으면 제외’에 따라 이 변경은 유지했다. 두 변경을 같은 것으로 취급하지 않는다.

원 SHA별 재적용 대조와 원시 rebase 로그는 ignored output의
`logs/rebase-96827-range-diff.log`, `logs/rebase-96827.log`에 기록했다.
코드 보존을 확인할 때 SHA가 다르다는 사실만으로 기능 미포함을 판정하지 않는다.

## 충돌 보정과 실제 검사

충돌은 `scripts/visual-sweep-font-registration.mjs` 1개 파일이다. 새 devel의
`sourceProgramFamilies`와 요청 전체의 프로그램 목록·중복 제거 소유를 유지하며,
기존 통합의 파일당 32MiB·요청당 64MiB 공급 상한도 보존했다. 예산을 family마다
초기화하면 동일 요청의 여러 family가 한도를 초과하므로 요청 전체에 한 번만 둔다.

- 충돌 해소 후 기존 Node 10건 통과.
- 여러 family가 요청 예산을 공유하는 반례 추가 후 Node 11건 통과.
- Visual Sweep Python 247건 통과.
- 새 devel의 선언 프로그램·PUA 세대·Print 전환 관련 Studio Node 13건 통과.

이 검사는 합성 소비 계약·도구 검증이다. 실제 문서의 Native/fresh WASM 일치나
새 base 대비 무하락의 증거로 승격하지 않는다. 원시 로그는 ignored
`logs/rebase-96827-native-pua-final.log`, `logs/rebase-96827-visual-sweep-python.log`,
`logs/rebase-96827-font-metric-node.log`에 보관한다.

## 새 base의 기능 보존 검증

PR #8583의 Print 메트릭 전환·오류 후 복원, 실제 원본 프로그램 family 소유,
Print 2024 정상 TTF 윤곽, JPEG 프레임·일반 그림의 동일 RGB 소비를 확인한다.
원 PR의 편람 HWPX 383쪽 Native/fresh WASM TSV와 대표 30·130·140·198·220·272쪽을
재사용할 때 source·입력·PDF·환경을 구분한다. 현재 후보의 실제 출력도 별도로 산출한다.

PR #8203은 텍스트 추출에서 소유가 확인된 개체 자리표를 다시 내보내지 않는 변경이다.
`text_file_control_placeholders`의 중첩·공유 앵커·원문 자리표·불완전 또는 역순 슬롯
검사를 현재 후보의 Rust 경로에서 확인한다.

현재 보정은 표의 수용 원점·셀 경계·원본 결측 기호 전진·PDF family 해소와 보안 출처다.
공통 생산자와 Native/Canvas/SVG의 소비, 정상 대조군·미세 간격·내용 누락을 확인한다.
새 base와 후보의 페이지별 raw는 사용자 지정 0.1%p 초과 하락 금지로 비교하며,
PR #8389·#8397의 기준 표시값 무하락도 별도로 유지한다.

## 검증 귀속과 남은 조건

리베이스 전 제품 `40778b67d6`의 전체 Rust는 12,511건 중 12,510건 통과·1건 실패·45건 skip이었다.
새 base·추가 보정의 통과로 재사용하지 않는다. `874fac36f2`의 Native 집중 결과 중
PDF 무표기판 1·5·13쪽은 전부 98% 이상이나 전쪽·fresh WASM의 대체가 아니다.
같은 제품의 표 높이는 개선되지 않아 추가 소비 보정을 진행했다.

진행 중이던 이전 제품의 historical WASM 검사는 rebase 전에 소유 프로세스만 종료했다.
완료한 쪽은 이전 소스 근거로 보존하고, 중단된 쪽이나 이전 빌드가 남았다는 이유로
새 후보를 통과로 기록하지 않는다. 공유 `target/pr-review`는 재사용하고 빌드 잡은 8개다.

최종 소스의 필수 회귀·lint·최적화 fresh WASM·전쪽 시각 검증·확대 판독·개별 리뷰 갱신을
완료한 뒤 PR 생성 승인 요청 단계로 간다. 현재 PR 생성·push·댓글·close는 실행하지 않는다.

## 제품 `2071603998`의 집중 재검증

- Native Skia 출처 계약 12건 통과, exit 0. 문서 유래 글꼴 이름의 실제 검출과
  정적 경로 메타데이터의 오탐 반례를 함께 확인했다.
- PR #8583·#8203 및 현재 보정의 집중 회귀 18건 통과, exit 0. 원본 전진 보존과
  프로그램 선택, 중첩 개체 자리표, 그림 첫 프레임·이어받기, PDF 공백 줄 계약을 포함한다.
- Studio TypeScript, 갱신한 suite 정책, Rust 단위 테스트 분류 검사 통과.
- 개별 리뷰 문서 39개 존재와 로컬 링크 누락 0건을 확인했다. 내용의 최종 수치 갱신은
  최종 제품의 Native/fresh WASM 증거를 확보한 뒤 수행한다.

집중 실행을 준비하면서 검토 worktree의 파생 suite 배치가 이전 소스에 해당하는 것을 발견했다.
그 실행은 컴파일 단계에서 소유 프로세스만 종료하고, 실제 worktree에서 `--prepare`와
정책 검사를 완료한 뒤 다시 실행했다. 중단 실행을 테스트 통과나 전수 실행 증거로 쓰지 않는다.
12건 출처 검사는 실행한 실제 모듈과 12개 최종 결과를 확인했으며, 파생 배치 변경과 구분한다.

실제 Native 출력에서 표 첫 쪽은 90.95%이나 마지막 행의 높이 차이가 여전히 남는다.
영월 서식 1쪽은 99.97%로 기준을 회복했지만 3쪽 99.92%는 기준 표시 99.97%보다 낮다.
두 결과를 완료로 표시하지 않는다. 3쪽의 text run 456개 기하는 이전 제품과 전부 같으므로,
글꼴 공급·실제 paint의 차이를 먼저 대조한다. 표는 저장 프레임 조회의 실제 원점·캐시 소유를
기존 `RHWP_DIAG_SCAN` 진단으로 구분해 추적한다.

원시 결과는 ignored output의 `logs/rebase-96827-feature-preservation-prepared-minfont.log`,
`logs/rebase-96827-studio-typecheck.log`, `logs/rebase-2071603998-native-probes.log`에 있다.

## 후속 devel 반영과 편람 변경 중복 제거

2026-11-04, 사용자가 제시한 `4c892a9886c12e8d946a38c936d27f9595dd66ea`를
새 devel `782ab3e2d06380d7d36660f07d76ef4c8c7c0c0a`와 대조했다.
PR #8588의 통합 커밋 `056652594ba2cd1a515c9a5def1f80c8e1275064`에 편람 글꼴 환경
전달 변경이 포함됐다. 원 커밋과 로컬 `a4fb27f651e17dc1b238b93005086fc165fe9e1b`의
stable patch-id는 `380f38b255baf135c8313d57174500142c3df831`로 같고,
구현·검사·분석 문서 3개도 새 devel과 바이트가 같다.

따라서 `a4fb27f651`만 제외하고 새 devel 위로 리베이스했다.
리베이스 직전 head는 `c6701105b4c58fc7841f2f643324a115f418e237`,
직후 head는 `0891a492f44b5f28a0a3bffb574385bd034cd034`다.
`range-diff`에서 이전 128개 중 해당 중복 1개만 빠지고 나머지 127개 패치는 모두 같다.
새 base의 조상 관계, 작업 트리의 변경 없음, 해당 3개 파일의 devel 일치도 확인했다.
기능을 되돌린 것이 아니라 devel의 동일 구현을 사용하도록 중복 이력을 제거한 것이다.

첫 리베이스는 일시적인 index lock으로 멈췄고, Git 프로세스 종료와 lock 부재를 확인한 뒤
`rebase --continue`로 완료했다. 소스 충돌은 없었다. 중단 로그와 완료 로그를 구분하며,
대조 결과는 ignored output의 `logs/rebase-782ab3e2d0-range-diff.log`와
`logs/rebase-782ab3e2d0-result.json`에 있다. 이전 제품의 렌더링 결과를 새 head의
재출력 결과로 바꾸어 기록하지 않는다. 최종 제품의 필수 검증은 계속 진행한다.

편람 gate의 도구 계약 검사는 새 head에서 19건 통과했다(exit 0).
첫 실행은 테스트 모듈을 직접 지정해 같은 디렉터리의 보조 모듈을 찾지 못했으며,
정식 discovery 방식으로 다시 실행해 확인했다. 로그는
`logs/rebase-782ab3e2d0-handbook-tests-discovery.log`다.

새 base PR #8588의 duration 게시 기능은 Node 계약 64건과 workflow 계약 4건이
통과했다. Visual Sweep Python 검사도 전체 247건 통과했다. 각각 exit 0이며,
원시 로그는 `logs/rebase-782ab3e2d0-duration-node.log`,
`logs/rebase-782ab3e2d0-duration-workflow.log`,
`logs/rebase-782ab3e2d0-visual-sweep-python.log`다.

개별 리뷰에 기재할 적용 SHA도 두 번의 `range-diff`를 연결해 갱신했다.
최초 대조표 107개 항목 모두 현재 head의 조상임을 확인했고,
ignored `rebase-current-sha-map.json`을 최종 리뷰 갱신의 입력으로 지정했다.
이는 커밋의 적용 이력 대조이며 최종 제품의 기능·시각 검증 수로 계산하지 않는다.

검증용 후보 worktree는 코드 `dd0ac546e3389f1703e67cb86f1c5bdf615ff0c9`로,
base worktree는 `782ab3e2d06380d7d36660f07d76ef4c8c7c0c0a`로 맞췄다.
기존 package가 남아 있는 사실은 최신 빌드 완료로 취급하지 않는다.
후보 fresh WASM은 공용 `target/pr-review`·빌드 잡 8개로 최적화를 포함해 실행하며,
다른 Cargo 명령은 해당 빌드가 종료된 뒤 순차 실행한다.

## 새 base 후보의 최적화 WASM·Studio 확인

코드 `dd0ac546e3389f1703e67cb86f1c5bdf615ff0c9`의 WASM 빌드는 컴파일 13분 6초,
최적화·패키지 생성 포함 14분 15초에 exit 0으로 완료됐다. Rust 1.93.1,
wasm-pack 0.15.1, wasm-bindgen 0.2.127, wasm-opt version 130을 사용했다.
검증용 package를 ignored `producer-packages/rebase-dd0ac546e3/`에 고정하고,
루트 `pkg/`와 Studio public의 JS·WASM 해시 일치도 확인했다.

- JS SHA-256: `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`.
- WASM SHA-256: `ee4043521f602cf7df7815851d4067fc7825e6db82a0c8c6a6b5cfc8363b0e7d`.
- 실제 WASM PDF 가져오기 2010·2020·무표기판의 저장 HWPX가 제출본과 각각 바이트 단위로 같다.
  PR #8443의 실제 편집·저장 사례도 제출본과 같다. 단순 텍스트나 쪽수 비교로 대신하지 않았다.
- 실제 브라우저에서 271개 assertion, 실제 Worker 17개(정상 11·메모리 실패 3·엔진 trap 3),
  12쪽 렌더링·저장과 실패 후 복구를 확인했다. exit 0이며 시각 일치율의 증거와 구분한다.
- Studio 전체 Node 2,720건과 TypeScript 검사 통과, 각각 exit 0.
- 최종 목록의 입력·PDF·글꼴 환경 188개를 현재 Git과 대조해 누락·바이트 불일치 0건을 확인했다.

원시 증거는 ignored output의 `logs/rebase-782ab3e2d0-fresh-wasm-product.json`,
`logs/rebase-782ab3e2d0-wasm-producer-parity.log`,
`logs/rebase-782ab3e2d0-fresh-wasm-runtime.log`,
`logs/rebase-782ab3e2d0-studio-node.log`, `logs/rebase-782ab3e2d0-studio-typescript.log`,
`logs/rebase-782ab3e2d0-input-audit.json`이다.

Native 제품은 같은 코드에서 최소 글꼴 5개만 노출해 별도로 빌드한다. 위 결과만으로
최종 전체 Rust·Skia·lint·전쪽 시각 보존 검증이나 39개 리뷰의 최종 완료를 선언하지 않는다.

## PR #8548·#8592 반영 리베이스

2026-11-04, 작업지시자가 devel 확인과 리베이스 우선 진행을 요청했다.
fetch 결과 devel이 `782ab3e2d06380d7d36660f07d76ef4c8c7c0c0a`에서
`7fdd5441ff1e77c083818e920a48089f294de35e`로 변경됐다. 새 base는 semanticist21
10건 통합의 조판·선택·저장·보안 보정과 후속 기록을 포함한다. 외부 PR 39건의
검토 범위에 새 PR을 추가로 체리픽한 것은 아니다.

직전 head `9383c4240d0603e52ac0b03fbb064b7730ec43ba`를 새 base 위로 리베이스했다.
직후 head는 `1fbe9e58ded48fa9f50dc96172106dff6e7bf96e`다. 기존 133개 커밋은
모두 유지됐고, `range-diff`의 129개는 같은 패치다. 차이 4개는 아래 충돌 보정 3개와
같은 함수의 문맥이 바뀐 후속 커밋 `570a5bd0cf`다. 후자는 diff 문맥만 달라졌다.

- `typeset.rs`: 새 base의 저장 tail 소유·정확한 끝 일치와 PR #8422의 저장 pen 소유
  1HU 환산 오차 비교를 함께 유지했다. 새 base의 뒤 저장 프레임 발행도 보존했다.
- `composer/line_breaking.rs`: 새 base의 `skipped_unlined`와 PR #8443의 `in_start_run`을
  각각 유지했다. 줄 없는 문단 처리와 편집 뒤 자리차지 개체의 점유 보존을 함께 적용한다.
- `rhwp-contracts/provenance.rs`: 새 base `63532b1ee5`의 컨테이너 단위 출처 선언이
  기존 보정의 경로를 이미 포함했다. `nativeFontSupply`·`fontEnvironment`와 PNG의
  관측·선언 환경 표지를 정본으로 유지하고 중복 상수를 추가하지 않았다. 해당 파일은
  새 base와 바이트가 같다. 기존 보정의 분석 문서와 반례 검사는 유지했다.

충돌 해소는 구현 보존의 실제 검증을 대신하지 않는다. 관련 기존 회귀와 새 base에서
들어온 기능, 원본·독립 Print의 출력을 새 head에서 검증해야 한다. GitHub 운영 지침의
checkout 포함 CI 예산 조정도 확인했으며 로컬 전체 회귀 명령은 그대로 유지한다.

이전 제품 `dd0ac546e3`으로 실행하던 소유 프로세스 24개만 SIGTERM으로 종료했다.
강제 종료는 없고 산출물은 삭제하지 않았다. 완료한 Native 3개 입력·34쪽,
WASM 12개 입력·124쪽, historical WASM 6개 입력·146쪽과 미완료 checkpoint를
이전 제품의 근거로 구분했다. 편람 383쪽 gate는 미완료이며 새 head 통과로 쓰지 않는다.
전체 Rust `2a4062d57a` 실행은 이 요청 전에 종료 코드 143으로 중단됐고, 테스트 summary와
잔여 Cargo/Rust 프로세스가 없어 통과·전체 실행 완료로 기록하지 않았다.

원시 대조·충돌 해소·이전 실행 종료 기록은 ignored output의
`logs/rebase-7fdd5441ff-range-diff.log`, `logs/rebase-7fdd5441ff-resolutions.jsonl`,
`logs/rebase-7fdd5441ff-owned-validation-stop.json`, `logs/rebase-7fdd5441ff-result.json`에 있다.

충돌 해소 구문의 포맷 차이 한 곳을 수정한 뒤 `cargo fmt --all -- --check`와
`git diff --check`를 통과했다. 포맷 재검사 로그는
`logs/rebase-7fdd5441ff-fmt-corrected.log`이며 exit 0이다.

### 새 base의 출처 컨테이너 선언과 반례 검사

포맷 보정 head `4b149b6cdfc514b9858ed644be2cf8120c731cd4`에서 최소 글꼴 5개를
노출한 Native Skia 집중 검사 58건 중 57건이 통과했다. TAC 간격·편집 뒤 개체 점유,
첫 프레임과 새 base의 조판·선택·저장 기능 검사는 통과했다. 원시 로그는
`logs/rebase-4b149b6cdf-focused-minfont.log`이며 exit 100이다.

한 건은 `provenance_contract::static_path_metadata_does_not_hide_document_values_or_forged_paths`의
설정 검증에서 실패했다. 새 base `63532b1ee5`는 `nativeFontSupply` 컨테이너 전체를
선언한다. 기존 반례가 정적 선언 경로에 반드시 `unavailable`이 있다고 가정했으므로
문서 값 탐색에 도달하기 전 설정 assertion이 실패했다. 실제 CLI 출처 검사는 모두 통과했다.

반례의 겹침 토큰을 고정된 계약 이름 `nativeFontSupply`로 바꾸고, 선언 배열의 첫 위치가
아닌 배열 내 실제 겹침을 확인한다. 문서 값의 위치 `nativeFontSupply.unavailable[]`,
위조한 최상위 경로와 중첩된 `untrustedFields`를 각각 검출하는 assertion은 유지했다.
출처 선언·제품 코드·탐색기의 허용 조건은 바꾸지 않았다. 관련 검사만 다시 실행해 확인한다.

보정 후 같은 최소 글꼴·Native Skia 환경에서 해당 출처 검사 12건을 모두 통과했다.
`logs/rebase-4b149b6cdf-provenance-corrected.log`, exit 0. 앞서 통과한 57건의 제품 코드는
변경하지 않았으며 이번 보정으로 렌더링 결과를 다시 생성하지 않았다. 집중 검사를 통과한
Native CLI 산출물을 `rhwp-candidate-rebase-4b149b6cdf`로 고정해 후속 시각 비교에 재사용한다.
SHA-256은 `71a6e065770d741971ad618e614a2d0bc1759cf2eefd4ce9b72ad395032f8d8f`다.

전체 회귀·최종 전쪽 Native/fresh WASM 비교가 아직 남아 있으므로 PR 준비 완료로 판정하지 않는다.

## 새 base의 전체 Rust 결과와 Native 전쪽 집계

`895205d5a7`의 최소 글꼴 전체 nextest는 12,558건 중 12,557 PASS, 1 FAIL,
45 skipped로 종료됐다. 종료 코드 100이며, 실패는 `issue_713_rowbreak_table_no_intra_row_split`
한 건이다. 통과한 검사까지 실패로 집계하거나 전체 실행이 성공했다고 표시하지 않는다.
실패 조건의 원 해결 이력·기본/Native 실행 구성·독립 Print·고정 base 전쪽을 대조한 결과는
[별도 분석](task_m100_713_clip_contract_revalidation.md)에 있다. 부적절한 viewport 대용 지표와
base에서 이미 미달인 원본은 [이슈 #8598](https://github.com/edwardkim/rhwp/issues/8598)로
이관했다. 전체 모듈과 근거를 보존했으며 공유 HWP·PDF와 정상 계약 검사는 유지했다.

이관 뒤 수집 목록은 2,045개에서 2,044개로 바뀌고, 차이는 `tests/issue_713.rs` 한 개다.
통과한 나머지 검사의 본문과 제품 코드는 이 이관으로 바뀌지 않았다. 전체 12,557 PASS를
재사용하고, 삭제만을 이유로 같은 전체 회귀를 반복하지 않는다. 검토 worktree에서 파생 suite를
다시 준비하고 manifest 검사를 통과했다. `.rs.txt` 사본은 자동 수집 대상이 아니다.

제품 코드 `4b149b6cdf`의 Native 80개 비교·406쪽을 완료했다. 전쪽 최저 raw는 91.85%이며
90% 미만·누락·독립 Print와 쪽수 불일치·원본/PDF 해시 불일치는 각각 0건이다.
PR #8408의 PDF 편집 저장본은 2010판 16쪽 최저 97.80%, 2020판 15쪽 최저 99.19%,
무표기판 15쪽 최저 98.54%다. 전쪽 점수만으로 확대 판독이나 최종 승인을 대신하지 않는다.

현재 `7bb9e66fd0`의 최적화 fresh WASM과 고정 base `7fdd5441ff`의 WASM을 공용
`target/pr-review`·빌드 잡 8개로 순차 빌드한다. 새 base 기능 32개 입력과 기존
PR #8389·#8397의 원 TSV에 대응하는 Native 비교도 실행한다. 최종 lint·Native Skia 3종과
fresh WASM 전쪽·기능 확인은 아직 완료하지 않았으므로 개별 리뷰의 최종 판정을 올리지 않는다.
PR 생성·push는 작업지시자 승인 대기다.

원시 로그는 ignored output의 `logs/rebase-895205d5a7-full-nextest-minfont.log`,
수집 차이는 `logs/issue8598-inventory-after.json`, Native 집계는
`logs/rebase-4b149b6cdf-native-80-audit.json`이다. 장시간 작업은 소유 worker·실제 child PID와
종료 코드를 별도 기록하며, 외부 셸 관측 종료를 빌드·비교의 실제 종료로 오인하지 않는다.

## 최적화 WASM·기존 TSV 보존과 최종 lint 보정

후보 `7bb9e66fd0`의 최적화 WASM은 exit 0으로 완료했다. 고정 package·루트 pkg·Studio
public의 JS/WASM 해시가 같다. WASM SHA-256은
`0fcc2a5d219ade9de64ce9657de95ce8769ce21d3bed4062835dc15e205ae3e1`이다.
실제 브라우저의 271개 assertion·Worker 17개·12쪽 렌더/저장과 실패 복구를 통과했다.
실제 PDF 가져오기 3종과 PR #8443 편집의 저장본도 제출 HWPX와 각각 바이트가 같다.

Native의 PR #8389·#8397 보존 검증은 17개 입력·고유 1,770쪽을 완료했다. 두 PR의
기준 TSV 2,153행을 대조해 원 PR 표시값 하락 0행이다. 과거 통합 기록과 환경이 다른
5행은 직전 제품을 같은 환경에서 다시 측정한 값도 충족했으며 그 구분을 결과 TSV에 유지한다.
필수 편람 HWPX Native 383쪽 gate도 passed, 기준 대비 하락 0쪽이다. 전쪽 최저 raw는
130쪽 99.96463%다. 같은 글꼴·입력·PDF·제품의 383쪽 결과를 기존 보존 목록에도 연결해
중복 렌더링하지 않았다. 이 결과만으로 아직 실행 중인 전체 fresh WASM 목록을 대신하지 않는다.

영월의 새 회귀는 검증 head `e8b36a41aa`에서 최소 글꼴 5개만 노출하고 1/1 PASS,
exit 0으로 완료했다. 기존 전체 검사 12,557 PASS의 본문은 그대로이며 이 새 검사를
이전 실행에 포함했던 것으로 집계하지 않는다. fmt·manifest·unit-tier도 각각 exit 0이다.

root Clippy는 `declared_first_frame.rs:182`의 `question_mark` 한 건으로 exit 101이었다.
`stored_row_atomic_picture_alignment`의 그림 띠 검색은 `Option<usize>`를 반환한다.
기존 `let Some(band) ... else { return None; }`를 같은 `position(...) ?`로 정리한다.
검색 조건·반환 인덱스·None일 때 일반 배치로 돌아가는 계약·계산값·컷·원점은 바꾸지 않는다.
원 보정 `d3b3f3d30d7`과 해당 함수의 실제 호출 경로·정렬 범위를 확인했다.
그림 행·정확한 컷·첫 프레임의 기존 집중 검사와 lint를 다시 실행하고, 새 빌드의 제품과
기존 고정 산출물의 동일성을 확인한 뒤 증적 재사용 범위를 확정한다. 아직 lint 통과나
새 제품의 시각 결과를 선언하지 않는다.

원시 근거는 ignored output의 `logs/rebase-4b149b6cdf-historical-native-audit.json`,
`historical-native-display-comparison-4b149b6cdf.tsv`, `handbook-gate-4b149b6cdf/`,
`logs/rebase-e8b36a41aa-new-source-supplier-minfont.log`,
`logs/rebase-7bb9e66fd0-remaining-rust-native-clippy.log`에 있다.


## Native 완료 자료 고정과 남은 검증 분리 — 2026-11-05

고정 제품 `4b149b6cdf`의 검토 입력 80건·406쪽은 전쪽 90% 이상이며 최저 91.85%다.
PR #8389·#8397 보존 범위는 17문서·1,770쪽, 원 TSV 2,153행의 표시값 하락 0행이다.
Native 최저 94.96164%이며 `issue7905` 전 6쪽 최저도 98.86043%다.
실제 Native TSV·출처 명세와 원 기준 대조를 정식 자산 `pr8389_8397/`에 고정했다.

새 base PR #8548의 32입력·72쪽은 같은 글꼴 환경의 base/후보 raw가 전쪽 동일하다.
65쪽은 PNG도 동일하고 7쪽은 raw가 같아도 경계 픽셀이 달라 추가 직접 판독 대상으로 유지한다.
compact render-tree 동일성은 직렬화하지 않는 clip·page_fragment까지 같다는 근거로 사용하지 않는다.
필수 편람 HWPX Native 383쪽 gate도 exit 0이며 최저 99.96463%다.
같은 고정 제품·입력·PDF·환경의 캡처를 기존 보존 범위에도 재사용해 중복 출력하지 않았다.

`b90db36de7`은 Clippy가 요구한 Option 반환 문법만 바꿨다. 이 커밋의 fmt와 root Clippy,
남은 workspace·WASM lint·Native Skia 검증을 공용 target에서 순차 실행한다.
새 제품의 해시와 실제 영향 경로를 확인하기 전에는 위 이전 제품의 캡처를 최종 head의 새 실측으로
표시하지 않는다. fresh WASM 전체 비교·최종 개별 리뷰 갱신과 PR 생성은 아직 완료가 아니다.


### Native base 80건 최종 대조와 경계 직접 판독

base 80건 출력이 끝난 뒤 기존 PNG가 남아 있는 쪽수 불일치 문서는 같은 실루엣 함수로
공통 쪽을 비교했다. 원래의 base/PDF 쪽수 차이는 manifest에 유지했으며 정상 쪽수로 바꾸지 않았다.
후보 406쪽의 최종 분류는 수치 대조 통과 396쪽, base 파싱 거부 뒤 후보 출력 6쪽,
base 누락 쪽 복원 4쪽이다. 미측정·후보 90% 미만·0.1%p 초과 하락은 각각 0쪽이다.
10쪽의 복원 사례를 base와의 수치 일치로 집계하지 않는다. 정식 TSV와 제품 해시는
`mydocs/pr/assets/planet6897_20261103/base-native-{comparison.tsv,provenance.json}`에 고정했다.

새 base `split-row`의 RGB 차이 7쪽(3·4·5·7·8·11·15쪽)은 기존 캡처로 review·overlay를
합성해 후보와 독립 PDF를 직접 판독했다. 5쪽은 base의 review·overlay도 함께 대조했다.
표의 행·열과 이어받기 내용은 유지되며, 5쪽 후속 본문의 높이와 11쪽 일부 글자 간격의 PDF 차이는
그대로 보인다. 이를 전 영역이 독립 PDF와 완전히 같다고 표현하지 않는다. 후보/base raw는
같고 실제 RGB 변화도 기록된 경계의 명암 차이이며, compact tree의 일치를 미직렬화 플래그의
동일성으로 확대하지 않는다. 판독 이미지는 ignored output의 `newbase-split-boundary-panels/`다.

`b90db36de7`의 fmt·root Clippy·WASM 대상 Clippy는 모두 exit 0이다. workspace와
Native Skia 검증 및 별도 제품 고정은 계속 진행한다. 제품 동등성을 확인하기 전까지는
기존 12,557건 PASS 재사용의 최종 증명을 완료한 것으로 표시하지 않는다.


### workspace 전체 target의 문서 lint 수정

root·WASM Clippy와 workspace build는 exit 0으로 종료됐다. `--workspace --all-targets`
Clippy는 `issue_8543_leading_tac_shape_line.rs`의 문서 목록 뒤 빈 문단 누락에 따른
`doc_lazy_continuation` 7건으로 exit 101이었다. 한 곳에 빈 모듈 문서 주석을 넣어 목록과
뒤 설명을 나눴으며, 검사 본문이 변경 전과 바이트 동일함을 확인했다. 기대값·허용치·제품 코드는
변경하지 않는다. 전체 target lint는 보정 뒤 재실행한다.

원격 `upstream/devel`에 `22947f6a90`이 확인됐다. PR #8285의 이동한 번호 자리표 code-unit과
PR #8585/통합 PR #8601의 글상자 TAC 그림별 소유 줄·말미 공백 정렬이 추가됐다.
기존 측정은 고정 제품의 과거 실행으로 보존하고, 새 base로 리베이스한 뒤 실제 소비 경로의
적용·비적용과 기존 39 PR의 개선 유지 여부를 대조한다. 본인 열린 PR은 0건이다.


## 새 base 22947f6a90 리베이스와 개별 판정 현황 — 2026-11-05

이전 head `9ed985e958`의 로컬 144개 커밋을 `22947f6a90` 위로 리베이스했다.
새 코드 head는 `a7e5ce7112`이며 충돌은 없었다. `git range-diff --abbrev=40`의
144개 항목이 모두 `=`이므로 기존 패치를 삭제·변경하지 않았다. 원 PR/SHA 대응표도
새 매핑으로 합성해 과거 source SHA와 실제 적용 SHA를 구분한다.

새 base의 실제 제품 변경은 `model/paragraph.rs`, `layout/paragraph_layout.rs`,
`layout/shape_layout.rs`다. PR #8285의 이동한 번호 자리표 중복 계상 방지와
PR #8585/통합 PR #8601의 글상자 다중 그림 소유 줄·공통 말미 공백 정렬을 보존한다.
본문 가운데 정렬은 기존 조건을 helper로 추출한 것이며, `last_inline_object_pos.is_some()`은
호출자가 비어 있지 않은 TAC 위치 목록의 최댓값을 전달하는 조건과 대응한다.
새 그림 배치 계획은 글상자 안 TAC 개체가 여러 composed line에 속한 경우에 적용된다.
번호 자리표 변경은 문단 나누기·저장·재열기 경로에서 검증한다.

충돌이 없으면 전체 비교를 다시 하지 않는 작업지시자의 범위를 적용한다. 기존 전체 측정의
고정 제품·입력·PDF 출처를 유지하고, 새 base의 적용·비적용 경로와 영향 사례를 새 제품에서
집중 확인한다. 기존 TSV를 새 head에서 다시 출력한 값으로 바꿔 기록하지 않는다.
Rust는 새 공통 제품 코드와 추가 검사를 포함해 필수 게이트를 순차 실행하며, 동일 Cargo target을
중복 사용하지 않는다. `a7e5ce7112`의 fmt·suite prepare/check·unit tier check는 exit 0이며
번호 자리표 집중 검사를 실행 중이다. 같은 제품의 Native/fresh WASM 고정 작업도 후속 대기한다.

진행 현황은 체리픽 39/39 PR, 개별 리뷰 문서 39개다. 문서의 수용 가능 판정은 37개,
‘수용 가능 — 최종 공통 검증 대기’를 명시한 것은 PR #8408·#8530의 2개다.
개별 보정 사유의 보류는 없지만 최신 base 이후 최종 승인은 아직 0개다. 기존 판정문을
새 head의 최종 승인으로 집계하지 않는다. 원격 PR 생성·push·approve·close는 실행하지 않았다.
진행 감사와 원시 검증 기록은 ignored output의 `logs/pr39-progress-20261105.json`,
`logs/rebase-22947f6a90-result.json`, `logs/rebase-a7e5ce7112-rust-worker.json`에 있다.


## 새 base 검증과 전쪽 증거 감사 — 2026-11-05

`a7e5ce7112`의 최소 글꼴 기본 전체 회귀는 **12,561건 모두 PASS, 45 skipped,
실제 exit 0**으로 끝났다. 검사 실행 시간은 997.005초이며 제품 성능 개선 수치로 해석하지 않는다.
원 PR에서 추가한 번호 자리표 1건·글상자 그림 2건도 앞선 집중 검사에서 PASS했다.
fmt·suite/tier 검사·workspace 전체 target Clippy·WASM Clippy·workspace build·doc test도
각각 exit 0이다. Native Skia의 unfiltered library와 CI가 지정한 21개 모듈을 후속 실행한다.
CI 원문에서 추출한 모듈 집합과 예약된 모듈 집합이 정확히 같은지 별도로 대조했다.

### 실제 소비 경로와 독립 검출 조건

번호 자리표의 실제 `Paragraph::split_at` 호출은 본문·표·각주 편집과 외부 붙여넣기 명령에 있다.
공개 WASM API로 같은 원본을 만들고 나누어 HWP/HWPX로 재열 때의 본문 보존은 구제품에서도
통과하므로 결함 검출 증거로 집계하지 않는다. 내부 문자 수 11 계약은 upstream의 실제 Rust
회귀로 검사했다. 합성 번호 입력은 공개 API·저장 계약 확인용이며 독립 Print 기준이 아니다.

PR #8585의 공개 원본을 구제품 `7bb9e66fd0`의 실제 WASM API로 열면 사진 여섯 장이 한 줄에
놓인다. 독립 Print의 2열·3줄 조건으로 실제 FAIL(exit 1)을 확인했다. 새 package 고정 뒤 같은
API 검사를 실행하도록 연결했고, 글상자 포함·정렬은 최소 글꼴 Rust 검사에서 이미 통과했다.

기존 Native 캡처의 80+17개 조합·2,176쪽과, 보존 범위 WASM 17문서·1,770쪽의 저장된
render tree를 읽어 글상자 소비 경로를 조사했다. 새로 렌더링하거나 백분율을 계산한 검사가 아니다.
보수적 다중 줄/다중 개체 후보는 Native PR #8498 원본의 2쪽 한 곳이며, WASM 보존 범위는
후보·누락 tree가 모두 0이다. 이 보조 조사를 clipping 동등성이나 최신 head 전쪽 시각 통과로
확대하지 않는다. 새 제품의 집중 Visual Sweep은 해당 5쪽 원본, PR #8585의 1쪽과 정상 대조군
#8122의 1쪽, 총 3입력·7쪽을 각 backend에서 확인하도록 예약했다.

### 원시 종료 코드와 파일 해시로 발견한 중단 복구

기존 제품의 WASM 80개 입력 worker가 exit 0으로 종료돼도 전 입력이 통과한 것은 아니었다.
현재 작업 트리를 리베이스하는 동안 일부 기여 fixture가 일시적으로 제거돼 입력 열기 단계에서
37건이 실패했고, 이미 별도 완료한 1건을 제외하면 감사 시점의 유효 결과는 44건·256쪽,
미완료는 36건·150쪽이었다. 처음 한 건이라고 보고한 수치를 전체 감사 뒤 36건으로 정정했다.
측정과 리베이스가 같은 변경 가능한 입력 경로를 공유한 작업 순서 문제이며 제품 회귀로 분류하지 않는다.

검증은 완료 manifest·전쪽 TSV·입력/PDF SHA와 실제 Native/WASM package 해시까지 대조한다.
완료된 44건은 유지하고, 이 36건만 같은 고정 제품으로 복구 실행한다. 실패 시도의 log와 상태는
보존하고 복구 상태를 별도 파일에 기록한다. 이후 전체 80건 감사를 다시 통과해야 완료로 판정한다.
원시 renderer worker의 exit 0이나 과거 review 문서의 수용 가능 판정으로 이 절차를 대신하지 않는다.
입력 파일을 읽는 검증이 진행 중이면 작업 트리를 다시 리베이스하지 않고, 안전한 경계와 입력
불변성을 먼저 확보한다. source SHA가 다른 산출물을 같은 실행의 증거로 합치지 않는다.

### 보존 범위 완료와 공간 확보

고정 제품 코드 `4b149b6cdf`·최적화 WASM build `7bb9e66fd0`의 PR #8389·#8397 보존 범위는
Native/fresh WASM 모두 17문서·1,770쪽, 원 TSV 2,153행의 표시값 하락 0행으로 끝났다.
Native 최저 94.96164%, WASM 표시 최저 94.65%다. 정식 TSV와 실제 source/package 해시는
`mydocs/pr/assets/planet6897_20261103/pr8389_8397/`에 고정했다. 최신 rebase의 집중 검증과
이 과거 제품의 전쪽 캡처는 출처를 구분한다.

디스크 여유가 2.7GB로 줄어 종료된 실험 제품 사본 16개(약 1.29GiB)를 정리했다.
사용 중인 고정 제품과 재현 기준 제품은 유지했다. 전체 Rust 회귀를 중단하지 않고 해당 작업이
실제 종료한 경계에서 Cargo/Rust·debug 실행/열린 참조가 모두 없음을 확인한 뒤, 사용이 끝난
`target/pr-review/debug/incremental`만 정리했다. compiled dependency cache와 현재
`release-test`·WASM 산출물은 보존했다. 가용 공간은 3,902,468,096→19,521,028,096바이트로
회복됐고, 뒤이어 Native 검증을 재개했다. 기록은 `logs/debug-incremental-cleanup-boundary.json`이다.


## 최신 제품 Rust 게이트 완료와 source 재확인 — 2026-11-05

`a7e5ce7112`의 Native Skia unfiltered library와 실제 CI의 integration 모듈 21개가
모두 exit 0으로 종료됐다. 앞선 기본 전체 12,561 PASS를 포함한 순차 Rust 게이트는
33/33 완료다. 공용 target·jobs 8·실제 최소 글꼴 5개 환경을 유지했다. 실제 명령·시각·
source와 종료 코드는 `mydocs/pr/assets/planet6897_20261103/rust-validation-a7e5ce7112.json`에
기록한다. 이 파일은 Rust 실행의 완료 근거이며 전체 PR 준비 완료 판정은 아니다.
추가로 새 base의 영향 경계와 기존 8472·8529·6699·8122 Native 집중 검사도 통과했다.

원 PR 39개를 GitHub의 읽기 전용 API로 일괄 재확인했다. 모두 Open·non-Draft이며
`jangster77` reviewer 요청이 유지된다. 고정 head 39개는 현재 관측 head와 모두 같다.
`source-head-recheck-20261105.tsv`에 원 PR 링크·각 head·확인 시각을 보존한다.
PR metadata의 `base.sha`는 PR별 과거 값일 수 있으므로 현재 devel ref로 해석하지 않았다.
별도 `git/ref/heads/devel` 조회의 실제 값은 `22947f6a90`이며 현재 기준과 같다.
원격 PR 생성·push·review·comment·close는 수행하지 않았다.

최신 Native/fresh 최적화 WASM 제품 고정과 실제 브라우저·API 검증은 후속 순차 실행 중이다.
새 기준 자체의 같은 입력 출력을 비교할 수 있도록 `22947f6a90` 전용 작업공간을 준비하고,
후보 제품 작업이 종료한 다음 같은 target에서 기준 제품을 빌드하도록 계획한다.
새 base의 집중 범위는 기존에 정한 3문서·7쪽이며 전체 기존 문서 재측정으로 확대하지 않는다.
번호 공개 API의 저장 정상화가 원 결함을 검출하지 못하는 한계는 유지하고, 실제 내부 문자 수
계약 및 글상자 사진의 적용·비적용 경계와 구분한다.

최종 리뷰 생성기는 완료 manifest와 전쪽 TSV가 일치하는 자료만 수집하게 했다. 부분 TSV나
과거 전체 측정을 최신 source의 새 실행으로 잘못 기록할 수 없게 실제 전쪽 producer/base와
최신 필수 검증 source/base를 별도 필드로 다룬다. 리베이스 재사용 조건과 최신 집중 보존 검증이
완료되지 않으면 최종 완료 문구를 생성하지 않는다. 이 도구는 ignored output 안의 검토 보조 도구다.


## 중단된 WASM 전쪽 복구 완료와 증적 대응 감사 — 2026-11-05

고정 제품 `4b149b6cdf`·WASM package `7bb9e66fd0`의 중단 입력 36개가 모두 종료했다.
복구 worker의 실제 exit 0 뒤 별도 감사도 exit 0이다. 전 80입력·406쪽의 WASM 최저 raw는
92.03052%이며, 90% 미만·누락·쪽수·입력/PDF·제품 해시 오류는 0건이다. Native 같은
80입력·406쪽도 완료 manifest와 전쪽 원문을 다시 감사해 오류 0건을 확인했다.
이는 새 렌더 실행이 아니라 이미 고정한 산출물의 감사다. 정확한 producer·base·package와
최저값은 `mydocs/pr/assets/planet6897_20261103/whole-page-capture-summary.json`에 보존한다.

대표 쪽 선택 100행 중 같은 입력·쪽 중복을 제외하면 93조합이다. Native 93조합 모두
실제 입력·독립 PDF·쪽·backend·source PNG SHA와 review/overlay 연결을 확인했다.
파일 대응 감사와 직접 판독은 별도다. 이번 점검에서 직접 읽은 여섯 review/overlay 쌍은
읽은 실제 PNG 해시와 남은 차이까지 ignored 판독 원장에 기록했고 전체 승인으로 확대하지 않는다.
한 개요 증적의 공용 key에 들어 있던 `-wasm`은 backend 중립 이름으로 정리했다. 같은
Native 입력·쪽으로 패널만 재구성했으며 renderer를 다시 실행하지 않았다.
WASM 대표 패널과 동일한 파일 대응 감사도 기존 완료 PNG를 사용해 후속 실행 중이다.

최신 source `a7e5ce7112`의 Native Skia 제품 빌드·고정은 exit 0이고 SHA-256은
`69113f81385355d22811a526ba8dd3e15c90bdffb58f858eddc59be4aef6c62b`다.
최적화 fresh WASM은 컴파일 중이며 이 빌드의 실제 브라우저/API 결과를 과거 package 결과로
대체하지 않는다. 새 base와 후보의 직접 대조는 상속한 PR #8585 및 정상 #8122의 2문서·2쪽이다.
후보의 전체 집중 범위 3문서·7쪽 중 나머지 PR #8498의 5쪽은 같은 입력의 리베이스 전 후보와
대조한다. 이 구분은 원래 작업 개선 유지와 새 base 기능 보존을 각각 입증하기 위한 것이다.
같은 환경의 WASM base 전쪽 대조와 최종 개별 리뷰·주석 갱신·PR 본문 준비는 아직 남아 있다.


## 최신 최적화 WASM·공개 API·실제 영향 쪽 확인 — 2026-11-05

`a7e5ce7112`의 제품 worker 13단계가 실제 exit 0으로 모두 종료됐다. 최적화 WASM
SHA-256은 `5b9319147bb66ad1f316354d9c3613f37ec34bb522323915abbb3d6591df12f2`이며
JS는 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`다.
실제 브라우저의 271 assertion·17 Worker·12쪽 렌더/저장 및 실패 복구가 PASS다.
PDF 가져오기 3종과 EMC 실제 편집의 새 WASM 저장본은 이전에 보존한 Native 산출물과
4/4 바이트 동일하다. 이를 최신 Native importer를 다시 실행한 증거로 바꾸어 표시하지 않는다.
번호 공개 API는 양쪽 모두 PASS였던 호환성 검사이고, 내부 문자 수 결함 검출은 정식 Rust
검사와 구분한다. 사진 입력의 공개 API는 구제품에서 FAIL, 새 제품에서 2열·3줄·열 정렬 PASS다.

누락 없이 원래 세 Clippy 명령을 각각 기록하기 위해 Native root Clippy도 공용 Cargo target의
순차 경계에서 별도로 실행해 exit 0을 확인했다. 실제 성공 Rust 게이트는 34개다. 같은
검증된 package의 JS·WASM을 기본 `pkg`와 Studio public에 원자적으로 동기화했다.
JS와 type binding은 이미 같은 바이트였고 WASM 두 사본만 변경됐다. 사용자 브라우저의
새로고침을 강제로 호출하지 않았다. 제품 source·테스트 assertion은 바뀌지 않았다.

최신 Native/fresh WASM의 집중 범위는 각 3입력·7쪽이다. PR #8498 원본 전5쪽씩은
리베이스 전 후보와 정밀 raw가 모두 같고 실제 PNG 10쪽도 RGB 동일하다. 상속한 그림
줄 배치 입력은 Native 99.95891%·WASM 99.96404%, 정상 #8122는 99.92750%·99.92779%다.
모든 입력·PDF·제품 해시, 전쪽·쪽수·90% 게이트 감사는 오류 0건이다. 최신 TSV 6개와
PR #8498의 실제 변경 전 TSV 2개를 원문 그대로 보존한다.
`mydocs/pr/assets/planet6897_20261103/rebase_scope_22947f6a90/`의 대표 review/overlay는
최신 실제 PNG에서 패널만 합성했다. 첫 합성에서 출력 디렉터리 생성이 누락된 오류를
수정해 재실행했고 retry exit 0이다. renderer를 재실행하거나 제품 코드를 고치지 않았다.
사진 입력의 Native/WASM은 실제 두 그림을 한 쌍씩 직접 읽어 여섯 개의 2열·3줄 정렬과
표 외곽·후속 문단을 확인했다. 작은 래스터 경계 차이를 완전 일치로 표시하지 않는다.

WASM 대표 증적도 중복 제외 93개 입력·쪽의 파일 대응 감사에서 오류 0건이었다.
최신 제품 검사와 같은 환경의 원 devel·후보 대조는 구분한다. 새 기준 `22947f6a90`의 실제
제품 빌드와 PR #8585·#8122의 대조, 기존 base의 WASM 80입력 대조·최종 개별 리뷰와
회귀 주석 갱신·PR 템플릿 완성은 남아 있다. 이 항목을 기존 후보 점수로 대신하지 않는다.


## 상속 기능 WASM 32입력의 같은 환경 대조 완료 — 2026-11-05

기존 dispatcher가 로드한 인수는 과거 게시 TSV를 reference로 삼아 `split-row` 세 쪽과
`notice-reopened` 한 쪽을 0.1%p 초과 하락으로 표시하고 exit 1로 끝났다. 그 비교 원문은
ignored 실패 기록으로 보존했다. 게시 당시 글꼴 기록은 `/home/edward/mygithub/ttfs` 및
rowbreak의 명시 HMKMM.TTF 공급이며 현재의 전체 시스템 프로그램 공급과 구분한다.
이를 현재 고정 base 대비 제품 회귀라고 단정하거나 기존 원 TSV가 통과했다고 바꾸지 않는다.

완료한 base `7fdd5441ff`와 후보 `4b149b6cdf`의 실제 캡처를 reference로 다시 집계했다.
renderer를 다시 실행하지 않았다. 32입력·72쪽의 같은 환경 비교는 실제 exit 0이며
정밀 raw는 72쪽 모두 동일하다. 원본·PDF SHA, 전체 쪽수, 인쇄 profile·조판 세대·PDF 래스터,
PNG exporter, 글꼴 프로그램과 공급 순서, 고정한 Native/WASM product를 각각 감사했다.
글꼴 공급 metadata는 두 실행에서 2,070개 항목과 순서가 동일하며 실행별 fingerprint를 보존한다.
0.1%p 초과 하락·측정 누락은 0건이다. 원 공개 TSV와 다른 환경의 raw 값을 수치 통과로
바꾸는 대신 현재 base의 실제 출력을 근거로 삼았다.

정식 기록은 `mydocs/pr/assets/planet6897_20261103/inherited-8548-wasm-preservation.json`이다.
이는 새 base의 #8548 기능 보존 대조이며 새로 접수하거나 체리픽한 PR은 아니다.
새 기준 `22947f6a90`의 실제 제품/2문서 대조와 주 검증 대상 80입력 WASM base 비교는 별도다.
