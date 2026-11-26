# #7754~#7765 단계 3: 전체 검증과 최신 devel 기능 보존

## 수용 조건과 고정 소스

대상은 #7754, #7755, #7756, #7757, #7759, #7760, #7761, #7762, #7765다.
각 원본의 전쪽 Native와 최적화 fresh WASM을 모두 98% 이상으로 유지하고 가능한 쪽은
100%까지 개선한다. 원본 TTF → 일치하는 TTC face → OTF(CFF) 순서를 바꾸지 않는다.

단계 2의 회귀 복원은 `c7e28cee7852b8643d65831decef39a6cae23a28`, 당시 생산 코드는
`2f46647df16da00365b22a1364cdc0b28b8344bc`에 고정했다. 이후 최신 devel 위로 리베이스했다.
검증 중인 소스 33의 생산 코드 고정점은 `adaa1963674e1a69c4369b80160ac6eb873bec1b`다.
후행 커밋의 문서, 테스트 경로, 원장 순서, 실제 생성된 공개 바인딩은 별도로 검증한다.

## 소스 32에서 완료한 증거

10개 평문 원본 583쪽과 실제 암호 원본 24쪽을 별도로 비교했다. 각 경로 607쪽의 전쪽
최저는 Native 98.08%, WASM 98.11%다. 시장구조조사 315쪽은 양쪽 최저 98.40%이며
WASM의 원본 chunk 10개와 합본을 보존했다. 실제 암호 원본의 24쪽 PNG는 같은 경로의
평문 원본과 전부 바이트가 같다. 이 결과를 새 생산물의 결과로 바꾸어 부르지 않는다.

Native 공식 검사는 root lib 3,853 PASS/12 ignored, 내부 lib 182 PASS,
placeholder 2 PASS, 직접 PDF 36 PASS다. fmt, workspace all-targets Clippy,
정책 검사와 기록된 Node·Studio 검사도 통과했다. HWPX 편람 Native 383쪽은
최저 99.97%, 표시 100.00% 382쪽, 정본 하락 0쪽이다. 기여자 공통 4문서의 두 경로,
8개 조건은 모두 100.00%이며 대표 review PNG를 직접 판독했다.

이전 기준으로 실행한 보호 문서 Native 23문서 2,023쪽은 32쪽 하락으로 실패했다.
사용한 보호표와 원문 TSV를 보존했다. 전체 Rust nextest는 컴파일 중 최신 devel 반영을
위해 취소했으며 실제 테스트 실행은 0건이다. 전체 Rust 통과로 확대하지 않는다.
증거는 `mydocs/pr/assets/issues7754_7765/source32/`에 있다.

중첩 셀 선언 최소 높이를 0·2000·2400·4280 HU로 바꾼 독립 한컴 Print 인과 실험도
보존했다. 단일 XML 속성만 바꾼 사본은 `.hwpx.bin`으로 보존하며 활성 회귀 입력에
추가하지 않는다. 입력 차이와 job 영수증은 `causal-cell-minimum/`, 실제 Print는
`pdf/issue7765/causal-cell-minimum/`에 있다.

## 최신 devel과 소스 33 생산물

2026-11-22 KST에 `upstream/devel`을 다음 SHA로 갱신했다.

- 이전 기준: `d7b5286cbf6401678a70eed3d66236a68dd2a72c`
- 새 기준: `5665e538292a0686a912f6a4d8950c55be89929a`
- 유입 변경: #9292의 편집 기능 17건 통합, #9297의 CI 요약 변경

격리 작업 트리 `/home/tsjang/rhwp-worktrees/task-7754-rebase-5665`에서 30개 커밋을
재적용했고 `range-diff` 30행 모두 패치가 동일했다. 이어 소스 32 종료 증거를 반영했다.
현재 작업 브랜치는 `task/7754-nine-fidelity-devel`이다. 공유 Cargo 작업을 모두 종료한 뒤
메인 checkout을 전환했으며 기존 소스 32 브랜치는 보존했다.

새 기준과 후보의 Native·최적화 WASM 빌드 4개가 모두 exit 0이다. Native 명령은
`cargo build --locked --profile release-test --target-dir target/pr-review --features native-skia --bin rhwp`다.
WASM은 정본 locked wrapper와 wasm-opt를 사용했다. 명령과 생산물 신원은
`source33/gates/producers.json`에 기록했다.

| 생산물 | SHA-256 |
| --- | --- |
| 기준 Native | `c02d7932b79f712bb07e435445a8bd0f69998d6bcef1eea7226fbb26281e63e1` |
| 후보 Native | `14d4ae261ee487dfa3603e13b928f16d38326462b31d828d56b0d7f4670bb6bb` |
| 기준 최적화 WASM | `c24c42b8475106f34d19ec2883826ad1386f5e959bd3b49bcf4180313f1053a9` |
| 후보 최적화 WASM | `3c5dff21cb6a620f49b1a7328f3e3a07e9149faea631f62f4e0ec59b81b0ebee` |

## 자동 수집 정책 보정

첫 정책 검사에서 복원한 `tests/issue_2279_layout_oracles.rs`가 현재 base에 없는 새
integration source로 판정돼 실패했다. 정책을 완화하지 않고
`tests/cases/issue_2279_layout_oracles.rs`로 옮겼으며 증거 include 경로만 조정했다.
이동 전 파일의 직전 변경 SHA와 시각은 테스트 첫머리에 기록했다. 같은 WASM에서 생성한
Studio 공개 바인딩 3파일을 함께 고정한 커밋은 `e6895accbe23daa2e57fa73c48bc9fe61c55ee57`다.

파생 suite를 재생성한 뒤 suite·unit 정책과 fmt가 통과했다. 첫 실패의 상태와 실제 로그는
`source33/gates/initial-policy-failure/`에 보존했다. 전체 Rust와 Native 공식 검사는
진행 중이며 미실행 게이트를 CI로 넘기지 않는다.

## 리베이스로 유입된 실제 편집 기능의 보존

#9273의 공개 편집 API를 기준·후보의 실제 라이브러리에서 각각 실행했다. 원본 HWP와
실제 한컴 재저장 HWPX에 30줄·70줄을 추가하고 편집 직후, HWP 저장 후 재열기,
HWPX 저장 후 재열기를 독립적으로 비교했다. 12개 조건 42쪽의 쪽수와 줄 소속이 같고
대응 PNG는 전부 바이트 단위로 같다. 독립 한컴 PDF와의 조건별 전쪽 최저도
Native 98.07~100.00%이며 base/head 하락은 0쪽이다.

실제 명령, 라이브러리·프로그램 신원, 글꼴 환경·캐럿, 양쪽 전쪽 TSV와 진단 소스는
`source33/gates/rebase-native-live/`에 있다. 진단 소스는 `.rs.txt`이며 새 활성 회귀나
golden이 아니다. WASM 라이브 편집과 17건의 저장 원본 재실행은 별도로 진행 중이다.

## Studio와 글꼴 추적 JS 검증

소스 33의 실제 WASM을 root package와 Studio public에서 SHA로 대조했다.
Studio 단위 검사 2,822개와 TypeScript 검사가 exit 0이다. 글꼴 추적의 계약·실제
fresh WASM 검사 16개도 Node 24.15.0에서 exit 0이다.

처음에 JS 검사를 Rust 전용 최소 글꼴 mount에서 실행했을 때는 `/usr/bin/node` 20과
숨겨진 `ttfs/FONTS.md` 때문에 환경 오류 2건이 났다. 실패 로그를 보존하고 지원 Node 24
환경으로 바로잡아 재실행했다. Rust 회귀의 최소 글꼴 환경은 유지한다. 완료 증거는
`source33/gates/frontend/`에 있다. 브라우저·기여자 공통 검사는 아직 이 결과에 포함하지 않는다.

복학원서는 새 후보의 실제 PNG에서 Native 99.00%, WASM 98.99%를 확인했다.
독립 Print와 접수증·절취선·후속 문구를 같은 영역에서 2배 확대 판독했다. 영문 face와
절취선의 미세 차이를 기록했으며 100% 일치로 판정하지 않는다. 증거는
`source33/direct-review/`에 있고 다른 문서의 직접 판독을 대신하지 않는다.

## 보호 글꼴 환경과 기존 본인 PR

영월 Native 설정의 누락된 캐시 JSON은 보존된 명시 설정과 같은 실행의 웹폰트 매핑을
정본 알고리즘으로 합쳐 복원했다. 기준 SHA-256과 정확히 같으며
`source33/font-environments/`에 보존했다. 설정 복원과 실제 글꼴 프로그램 확보·시각 통과는
구분한다. outline의 이전 Native 설정과 일부 기준 글꼴 바이트의 동일성은 계속 확인한다.
새 정본을 적용한 보호 전쪽 검증은 진행 중이며 확인된 하락을 통과로 판정하지 않는다.

본인 열린 PR #9293은 다른 작업의 브랜치다. 읽기 전용 CI 조사에서 합성 테스트 글꼴
`tests/fixtures/hft_mapped_symbol_program_tracking/fallback.ttf`의 고지 원장 누락으로
Frontend gate가 실패했다. 2파일 보정안을 준비했으나 다른 원격 브랜치는 수정하지 않았다.
별도 원격 변경에 대한 사용자 답을 기다리면서 현재 9개 이슈의 검증을 계속한다.

#9293과 충돌한 4개 원장은 서로 다른 문서 레코드를 같은 파일 끝에 추가한 충돌이었다.
본 작업의 TSV 블록과 참조 JSON 10개만 앞으로 옮겨 모든 행·수치·해시와 타 문서의 순서를
보존했다. 격리 index의 가상 커밋으로 같은 #9293 head와 재시뮬레이션한 결과는 exit 0이다.
원격 브랜치를 수정하지 않은 이 보정은 `a1b092c754345b8c374ba366e96468e3422424e0`에 있다.

## 남은 검증과 자원 관리

소스 33의 모든 대상·보호 원본을 같은 입력·독립 Print·DPI·compat·글꼴 조건으로 기준과
후보에서 비교한다. 실제 암호 원본 24쪽, 공식 편람, 기여자 공통 검사, Studio 브라우저,
유입 기능의 실제 동작과 직접 확대 판독을 함께 완료한다. 누락·미달·정본 하락은 해소한 뒤
다시 검증하며, 전체 Rust·필수 lint·workspace build의 실제 종료를 확인한 후 PR을 준비한다.

디스크에는 종료한 작업 소유 frozen probe library의 바이트가 같은 11개만 hard link로
중복 제거했다. 경로와 SHA를 유지하면서 약 2 GB를 확보했고 공유 Cargo target은 보존했다.
Native 공식 lib 컴파일 중 스왑이 가득 찬 것을 확인해 보호 sweep과 WASM 대상 sweep의 바깥 스케줄러만
잠시 멈췄다. 진행 중인 문서의 자식 캡처는 끝까지 실행하고, lib 게이트 종료 후 자동으로
스케줄러를 재개한다. Cargo나 진행 중인 캡처를 취소하지 않는다.

## 소스 33 배포용 문서와 143E 직접 판독

배포용 문서 3쪽의 예산 표·단위 줄·후속 문구와 143E 문서 1쪽의 인용 부호·영문 용어·
숫자·앞뒤 본문을 독립 Print 및 기준/후보의 두 경로와 2배 확대 대조했다.
배포용 문서 해당 쪽의 raw는 양쪽 100.00%이며 전쪽 최저와 구분했다. 143E는
Native 99.93%, WASM 99.94%로 기록했다. PNG·입력·PDF의 신원과 실제 판독 내용은
`source33/direct-review/`에 연결했다.

## Native 공식 lib 종료

소스 33의 필터 없는 Native `cargo test --features native-skia --lib`는 컴파일
31분 23초 뒤 root 3,851개와 내부 library 182개가 통과했다. 합계 4,033 PASS,
0 FAIL, 기존 ignored 12개다. 실제 명령·최소 글꼴 환경·원문 로그는
`source33/gates/rust/`에 보존했다. 자원 조절에 사용한 바깥 sweep 스케줄러 3개는
lib 게이트 종료 후 자동 재개됐다. placeholder·직접 PDF·Clippy·workspace build·전체
nextest는 아직 이 lib 통과에 포함하지 않는다.

Native lib의 root 통과 수가 소스 32보다 2개 적은 이유도 확인했다. 새 devel이
옛 제품명 표시 투영 검사 4개를 제거하고 원본 자모·모음 보존 검사 2개로 대체했다.
새 두 검사는 기준 devel에도 있고 이번 실행에서도 통과했다. 수집 누락이 아니며
crate를 구분한 실제 이름 차이는 `source33/gates/rust/unit-inventory-devel-change.json`에 있다.

## 직접 판독에서 확인한 #7754 추가 보정

24쪽 원본은 수치 98% 이상이지만 첫 쪽의 8번 문단은 배치를 완료로 판정할 수 없다.
실제 Hancom 13.0.0.3901이 만든 HWPX의 저장 cs=3500, sw=36520 HU와 문단
좌·우 여백 3500·2500 HU는 단 폭 42520 HU의 완성 본문 상자를 설명한다.
독립 Print와 원본의 기대 x=160.0533 px, 폭=486.9333 px에 대해 현재 두 경로는
x=206.7 px, 폭=406.9 px로 그려 여백을 다시 적용한다. 같은 문단의 Square 그림은
본문 왼쪽 여백 안에 있다. 단 오른끝만 검사하는 현재 판정과 완성 문단 상자의 계약을
기존 반례·사양과 함께 추적해 공통 경로에서 보정한다.

Enter 키 U+F03A0도 현재 원문을 보존하면서 표시만 화살표로 투영한다. 실제 공급 가능한
Haansoft Batang·HCR 프로그램에는 원 glyph가 있으므로 같은 SFNT의 측정·paint 공급을
확인한다. 옛한글 표제는 원문 자모를 유지하면서 올바른 조합·글꼴을 공급해야 하며,
새 devel에서 제거한 제품명 전용 문자 치환을 되살리지 않는다.

현재 Native placeholder 검사도 실제 exit 0, 2 PASS로 종료했다. 자식 종료 후 바깥 Rust
큐를 종료하고 소스 33의 나머지 게이트는 미실행으로 보존했다. 진행 중인 테스트를
중단하지 않았다. 새 보정의 최종 생산물에서 필수 검증을 다시 완료한다.
소스 33의 수치·직접 판독 상태는 `source33/status.json`과 확대 패널에 명시했다.
