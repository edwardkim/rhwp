# #7754~#7765 단계3: 전체 검증과 최신 devel 기능 보존

## 시작점과 수용 조건

단계2의 구현과 회귀 복원을 `c7e28cee7852b8643d65831decef39a6cae23a28`에 고정했다. 생산 코드는 `2f46647df16da00365b22a1364cdc0b28b8344bc`다. Native와 최적화 fresh WASM의 원본별 모든 쪽98% 이상, 가능한 쪽100%를 유지한다. 원본 TTF → 일치하는 TTC face → OTF(CFF) 순서를 바꾸지 않는다.

## 완료한 고정 소스 검증

소스32의 Native Skia공식3종은 root lib3853 PASS/12 ignored, 내부 lib182 PASS, placeholder2 PASS, 직접 PDF36 PASS다. fmt와 workspace all-targets Clippy도 exit0이다. 최소 글꼴 suite·unit 정책 확인과 Node36개가 통과했다. 같은 fresh WASM의 Canvas 글꼴26개, Studio 별칭·Canvas 비교·CanvasKit 준비도 통과했다. 실제 명령·원본 로그는 `mydocs/pr/assets/issues7754_7765/source32/gates/`에 보존했다.

HWPX 편람 Native383쪽은 최저99.97%, 표시100.00%382쪽, 정본 하락0쪽이다. 기여자 공통4문서·두 경로8조건은 모두100.00%이며 직접 review를 판독했다. 전체 Rust nextest와 나머지 전쪽 검증은 실제 종료를 기다린다.

## 최신 devel 유입과 검증 계획

2026-11-22 KST에 `upstream/devel`이 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`에서 `5665e538292a0686a912f6a4d8950c55be89929a`로 이동했다. #9292의 편집17건 통합과 #9297의CI 요약 변경이다. 현재 후보와 최신 devel의 병합 시뮬레이션은 충돌이 없다. 기존 결과를 새로운 생산물의 결과로 바꾸어 부르지 않는다.

진행 중인 고정 소스 Cargo 종료 증거를 보존하면서 격리된 작업 트리에서 rebase를 준비한다. 새 base/head의 실제 출력, 원본별 주요 경계와 유입 PR 기능을 비교하고 원본별 전쪽 Native/fresh WASM, 필수 lint·회귀·편람·정본 보호를 새 생산물에서 완료한다. 필수 미달은 구현을 보정한 뒤 재검증한다.

기존 본인 열린 #9293은 다른 작업의 브랜치다. 읽기 전용 CI 조사에서 합성 테스트 글꼴 `tests/fixtures/hft_mapped_symbol_program_tracking/fallback.ttf`의 라이선스 원장 등록 누락으로 Frontend package gate가 실패했다. 임의로 그 브랜치를 덮어쓰지 않는다. 본 후보와 #9293의 병합 시뮬레이션은 공통 원장4파일이 충돌하므로 새 PR 생성 전 해결이 필요하다. 실제 진단과 당시 head를 출력 기록에 보존한다.


## 리베이스 준비와 소스32 종료 결과

격리 작업 트리 `/home/tsjang/rhwp-worktrees/task-7754-rebase-5665`에서30개 커밋을
최신 기준5665e538 위에 재적용했다. 후보는 `16011f7341412830b88b732f82a0618464276c8c`이며
`range-diff`30행 모두 패치 동일이다. 공통 생산 파일16개를 겹쳐 수정하지만 실제 결과의
유지는 아직 새 생산물에서 확인해야 한다. 메인 checkout의 진행 중 Cargo 소스는 바꾸지 않았다.

이전 소스의 nextest는 root lib·lib-test 컴파일 중 취소했고 실제 테스트 실행0건이다.
이전 d7 기준의 새 참조 비교도 종료해 새 기준 검증으로 전환한다. 완료했던 개별 게이트는
그대로 보존하고 전체 Rust 통과로 확대하지 않는다.

소스32의 최적화 Native와 fresh WASM은10개 평문 원본583쪽 전부98% 이상이다.
실제 암호24쪽을 별도로 포함한 각607쪽의 최저는 Native98.08%, WASM98.11%다.
시장구조조사315쪽은 양쪽 최저98.40%이며 원본10개 WASM chunk와 합본을 정식 보존했다.
Native의 이전 보호표23문서2023쪽은32쪽 하락으로 실패했다. 원문·사용한 보호표를
보존했으며 최신 기준5665의 상향 보호표와 새 생산물로 재검증한다.

중첩 셀 최소 높이0·2000·2400·4280HU의 독립 한컴 Print 인과 실험도 정식 보존했다.
사본은 실제 바이트의 `.hwpx.bin`으로 보존해 활성 회귀 입력을 추가하지 않는다.
`mydocs/pr/assets/issues7754_7765/causal-cell-minimum/`의 단일 XML 속성 변경과
`pdf/issue7765/causal-cell-minimum/`의 Print·실제 job 영수증을 연결했다.

디스크에는 종료한 작업 소유 frozen probe library의 바이트 동일11개만 hard link로
중복 제거했다. 모든 경로·SHA는 유지했고 약2GB를 확보했다. 공용 Cargo target에는 손대지 않았다.

## 2026-11-22 KST: 고정 소스33 검증 생산물과 조건 복원

후보를 `task/7754-nine-fidelity-devel`의 `adaa1963674e1a69c4369b80160ac6eb873bec1b`로
고정했다. 새 기준5665와 후보의 최적화 Native 빌드는 각각 exit0이며, 기준 최적화 WASM도
완료했다. 후보 최적화 WASM과 이후 전쪽 비교·전체 Rust 검증은 진행 중이다.
이 시점에는 소스33의 시각 수치나 전체 회귀 통과를 아직 선언하지 않는다.

기준 Native 실행 파일 SHA-256은
`c02d7932b79f712bb07e435445a8bd0f69998d6bcef1eea7226fbb26281e63e1`,
후보는 `14d4ae261ee487dfa3603e13b928f16d38326462b31d828d56b0d7f4670bb6bb`다.
실제 빌드 명령은 `cargo build --locked --profile release-test --target-dir target/pr-review
--features native-skia --bin rhwp`이며 두 생산물은 별도로 고정했다.

영월 Native 설정의 누락된 캐시 JSON은 보존된 명시 설정과 같은 실행의 웹폰트 매핑을
정본 알고리즘으로 합쳐 복원했다. 기준 SHA-256과 정확히 일치하며, 정식 증거는
`mydocs/pr/assets/issues7754_7765/source33/font-environments/`에 보존했다.
설정 복원과 글꼴 프로그램 확보·실제 시각 통과는 구분한다. outline의 이전 Native 설정과
일부 기준 글꼴 바이트의 동일성은 계속 확인한다.

## 소스33 자동 수집 정책 보정

네 생산물의 실제 빌드가 모두 exit0으로 끝나 전쪽 비교를 시작했다.
새 회귀 정책 확인에서 복원한 `tests/issue_2279_layout_oracles.rs`가 현재 base에
존재하지 않는 새 integration source로 판정돼 실패했다. 정책을 완화하지 않고
`tests/cases/issue_2279_layout_oracles.rs`로 옮겼으며 정식 증거 include 경로만 조정했다.
이전 파일의 직전 변경은 `d54d21d54532bd086bffef1574ece96cbc81bd9a` (2026-11-22T03:38:32+09:00)다. 기존 시각 근거와 검사의 의미는
유지한다. 파생 suite를 다시 생성하고 정책·집중 회귀·전체 필수 검증을 재실행한다.

같은 최적화 WASM에서 생성한 Studio 공개 바인딩3파일도 함께 고정한다.
생산 Rust 코드는 소스33 고정점과 동일하며 새 테스트 위치와 공개 바인딩은
최종 커밋에서 검증한다. 첫 실패의 상태와 로그는 별도로 보존했다.

## 중첩 표의 실제 편집 검증과 원장 충돌 원인

유입 PR #9273의 실제 공개 편집 API를 기준5665·후보 소스33에서 각각 실행했다.
두 원본 ×30·70줄 ×편집 직후·HWP 재열기·HWPX 재열기의12조건42쪽은
대응 PNG가 바이트 단위로 같다. 독립 한컴 PDF와의 각 조건 전쪽 최저도
Native98.07~100.00%이며 base/head 하락0쪽이다. 정식 증거는
`source33/gates/rebase-native-live/`다. WASM 라이브 편집은 별도로 진행 중이다.

다른 본인 PR #9293과 충돌한4개 원장은 서로 다른 문서 레코드를 같은 파일 끝에
추가한 충돌이었다. 본 작업의 TSV 추가 블록과 참조 JSON10개만 앞으로 이동해
모든 행·수치·해시·타 문서 순서를 보존했다. 격리된 Git index의 가상 커밋으로
동일 #9293 head와 다시 병합 시뮬레이션한 결과 exit0이다. 다른 원격 브랜치는
수정하지 않았다. 이 순서 변경은 필수 전체 회귀 실행 전에 적용했다.

## Studio 및 글꼴 추적 JS 검증

소스33의 실제 최적화 WASM을 root package와 Studio public에서 SHA 대조한 뒤
Studio 단위 검사2822개와 TypeScript 검사를 완료했다(exit0). 글꼴 추적의 계약·실제
fresh WASM 검사16개도 Node24.15.0에서 exit0이다. 처음에 Rust 전용 최소 글꼴 mount에서
실행했을 때는 `/usr/bin/node`20과 숨겨진 `ttfs/FONTS.md` 때문에 환경 오류2건이 났다.
실패 로그를 보존하고 지원 Node24 환경으로 바로잡아 재실행했다. Rust 최소 글꼴 조건은
그대로 유지한다. 완료 결과는 `source33/gates/frontend/`에 있다.

복학원서는 후보 Native99.00%·WASM98.99%의 실제 새 PNG로 접수증·절취선·후속 문구를
2배 확대 판독했다. 영문 face와 절취선의 미세 차이를 기록했으며100%라고 확대하지 않는다.
전체 문서와 다른 원본의 확대 판독 완료를 대신하지 않는다.
