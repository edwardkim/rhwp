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
