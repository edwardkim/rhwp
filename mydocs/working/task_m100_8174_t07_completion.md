# #8174 T07 — 게시 계약과 14개 패키지 최종 소비 완료

- 승인: 2026-10-28 사용자 「T07 역시 나누지 않고 한 단위로 작업 범위를 잡고 진행해주세요」.
- 범위: [완료 계획](../plans/task_m100_8174_t07_completion.md)의 F01–F05 전체.
- 상태: **T07 완료, 필수 잔여 0건.** T08·T09와 실제 registry 게시·제품 릴리즈는 별도다.
- 패키지·구현 검증 source: `8e1581df605b957b9c66fc21f696c004abc7b9d2`. T06 기준은
  `c918b3f9b60e12e332ee1d38f66ea524d975a740`이다.
- 명령·archive·전쪽 TSV·PNG·실행 원장: ignored
  `output/pr-review/issue8174/t07-completion-20261028/`.

## 확정한 게시·소비 계약

MD01–MD09의 기존 선택을 다시 묻지 않고 manifest와 안내에 반영했다. 본체는 **0.9.0**,
내부/resource는 0.1.0, 보정 fork는 승인된 기존 보정 버전이다. 14개 모두
`rust-version="1.93.1"`과 `publish=["crates-io"]`를 선언한다. 제품 MINOR 동기화 규칙에
따라 Studio·editor·VS Code·브라우저 확장 manifest/lock도 0.9.0으로 맞췄다.
외부 의존성 lock 항목은 T06과 같고 본체 version만 변경했다. Rust 구현·기본 feature·
편집/렌더링 구현과 기대값은 변경하지 않았다.

[패키지 README](../../packaging/crates-io/README_PACKAGE.md)에 핵심 parser/IR,
DocumentCore/service·저장·표준 오류의 공식 소비 진입점과 0.9.x 소스 호환 계약을 명시했다.
내부 renderer/layout·JS 제품 facade·보정 fork의 직접 API까지 이 계약으로 확대하지 않는다.
현재 검증 target은 Rust 1.93.1의 Linux/WSL2 `x86_64-unknown-linux-gnu`와
`wasm32-unknown-unknown`이다. 미검증 하위 Rust·추가 OS를 약속하지 않는다.

[수동 운영 가이드](../manual/crates_io_guide.md)에 기능·CLI 설치·지원표, 비신뢰 PDF의
정책/격리 host 책임, 초회 게시·owner 확인·부분 실패·보안 업데이트·yank 인계를 정리했다.
책임자는 edwardkim이고, crates.io 일정은 제품 0.9.0 릴리즈와 별도다. 기존 네 채널 tag
workflow에는 연결하지 않았다. `parserAllocationBoundsProven=false`, 전체 heap/RSS와
동기 main-thread 선점의 비보장도 유지한다.

## 포장 단계의 실제 보정

Cargo include와 source 정책을 맞춰 최근 추가한 Nimbus Roman 계량 자료·고지와
`pdf_isolation_host` 예제를 포함했다. 기존 검증용 PDF 디렉터리의 URW 고지는 자료 디렉터리에
동일 바이트로 복사해 패키지 내부에서 읽을 수 있게 했으며 원본은 보존했다.

실제 source 사전 검사는 backend의 합성 `minimal-cff.otf`가 notice registry에 없어 처음
거절했다. 생성기·네 glyph의 원본 합성 출처·SHA-256·MIT/Apache 고지 경로를 등록해 보정했다.
글꼴 검사를 생략하거나 corpus 제외 규칙을 풀지 않았다. backend의 docs.rs metadata도
기존 Python 부가 기능 대신 지원하는 기본 Rust 구성을 문서화하도록 정리했다.

모든 archive는 같은 깨끗한 Git source에서 `cargo package --no-verify --registry crates-io`로
생성했다. 정규화 manifest의 path/workspace/patch 제거, 원래 manifest·VCS SHA·전 파일
바이트·실행 권한·고지·내장 자료·개인 자료 제외를 검사했다. **Cargo 자체의 포장 후 verify와
실제 crates.io parent dry-run을 실행한 것으로 표시하지 않는다.** 이후 내려받은 archive를
독립 workspace에서 실제로 빌드·실행했다.

## 실제 archive 원장

합계 **16,258,558 bytes**다. 아래 순서는 검증한 자식→부모 의존 순서다.
각 archive의 VCS source는 위 SHA로 고정했고 게시 제한은 crates.io 전용이다.

| package | version | 파일 수 | archive bytes | SHA-256 |
| --- | --- | ---: | ---: | --- |
| rhwp-codec-resource | 0.1.0 | 7 | 5,326 | `3b46d75c4d5b4e4b9c8bc255f303a6625106b75d22d4c448920bde9110bdcd42` |
| rhwp-contracts | 0.1.0 | 11 | 32,015 | `329e584f635ec7f82e3a00f682acaa66d87ef53167907a33c38503f9c84f6cbc` |
| rhwp-jpeg-decoder | 0.3.2-rhwp.1 | 27 | 53,259 | `9565a3ccb0338ff8b91aedf1b571545cc8d7d1facf7a28eafd22456463845db2` |
| rhwp-jpeg-encoder | 0.7.1-rhwp.1 | 22 | 42,466 | `5ccc3218ec86cda9db7348de756128fc26f7e8d43da24c029cd749d1f71bf880` |
| rhwp-ooxml-chart | 0.1.0 | 14 | 103,918 | `5ea6842e891c651694c77a48113495cb3619cb3e6ebadf323224c17bce6ae8a3` |
| rhwp-password-crypto | 0.1.0 | 7 | 15,909 | `bdea4a688c05b12868c63db8f15faef7b121029477a478a3998e5fafa89ad09c` |
| rhwp-svg2pdf | 0.13.0-rhwp.1 | 27 | 59,098 | `454daa02873c221c9a40b452dad1b45de829ffa5de7c74c1d00e1872b7a4c559` |
| rhwp-zune-jpeg | 0.5.15-rhwp.1 | 39 | 92,527 | `44edaca89c58ec524d6270452926d1a1a87c46652d1e3659c802ffc4cdea4a20` |
| rhwp-hayro-ccitt | 0.3.0-rhwp.1 | 15 | 16,768 | `1a6d01595c6b76e0e18d2a6770a7738d9f10a02a38069d9b357afc45300f5dc2` |
| rhwp-hayro-jpeg2000 | 0.4.0-rhwp.1 | 42 | 96,179 | `6d446f0857239e08f60f88607da487153a869b8fb5c7a86e7c88fe842a01caa5` |
| rhwp-qcms | 0.3.0-rhwp.1 | 22 | 56,538 | `4620442a258059c4238db09688d7c754a3cc04f89e4c782d0cc27d88e03d00ea` |
| rhwp-hayro-jbig2 | 0.3.0-rhwp.1 | 34 | 70,541 | `f71ade26568a21026fc0ff3e9f4bbc57a1bd4cc6aa11d42a2d2b231d2ca96a35` |
| rhwp-pdf-import-backend | 0.3.78-rhwp.1 | 430 | 6,966,045 | `b17d0655b81b1c58723e20e4aafe9a20daacef1bd66f2099de209e32f7151ab9` |
| rhwp | 0.9.0 | 1836 | 8,647,969 | `81820df4adec28e9aafd9b0e396d8e20bb6705c4cb0dd881b31cd02cb11f1faf` |

원본 `.crate`·파일 원장·다운로드 요청·체크섬·소비 명령은 `packages.json`,
`source-preflight/`, `successful-commands.json`, `http-requests.json`에 보존한다.
이전 S05의 0.8.7 archive나 source 검사를 이번 0.9.0의 실제 소비 결과로 바꾸지 않았다.

## 실제 소비·운영 검증

검증 registry는 loopback index/download만 제공했다. Rust·문서/host·WASM 앱은 저장소 밖의
독립 workspace이며 manifest에 repository path/patch가 없다. 사용자 전역 Cargo 설정/cache는
수동 수정하지 않았다. 모든 Cargo 작업은 공용 `target/pr-review`에서 순차 실행했다.
Native·문서 소비 graph의 14개, WASM graph의 13개는 실제 다운로드 checksum/source 경로를
대조했다. `rhwp-svg2pdf`는 Native PDF 출력 전용이므로 WASM에서 제외되는 것이 정상이다.
선택한 외부 package/version 집합은 root lock의 범위 안이며 추가 갱신은 없다.

| 항목 | 실제 결과 |
| --- | --- |
| parser/IR/service/core/저장 | 기본·최소 Rust로 HWP/HWPX 열기→두 형식 저장·재열기→편집 marker 보존 통과. 최상위 문단 텍스트·section 중심 smoke이며 모든 story/속성의 무손실 주장이 아님 |
| 패키지 README | Rust 예제 3개 추출·컴파일·실행, CLI constrained 가져오기 명령 실행 통과 |
| 기본 PDF·개발용 제외 구성 | Native·WASM 각각 기본 41개 가져오기·저장·재열기 통과. 제외 구성의 JBIG2/JPX 거절과 정상 후속 요청 통과 |
| 공용 공개 계약 | 표준 오류 1 + CLI 5 + 암호 2 + JPEG 3 + OS 감독 7 + 정책 6 + 요청 수명 43 = **67 PASS** |
| 주 CLI 설치 | registry에서 기본·native-skia·PDF 제외 3개 구성 설치 통과. 기본 설치본 5개 명령, 기존 CLI 16개·가이드·제외 동작 통과. 기존 paired stdout/stderr는 의도된 0.8.7→0.9.0 version stamp만 대조에서 치환 |
| Rust 문서·host 예제 | 기본 의존 구성의 root rustdoc·PDF 정책 문서 생성 통과. 실제 package 예제를 일반 사용자·사용자 네임스페이스 root에서 실행해 파일/네트워크 제한과 성공 출력 확인 |
| WASM facade on/off | 두 구성 모두 release·wasm-opt 완료. 실제 브라우저에서 엔진 0.9.0·별도 앱 start 소유·HWP 왕복 확인 |
| 실제 Worker | **267 assertions·17 Workers PASS**. 전체 12쪽 렌더링/저장, 메모리 실패·엔진 trap·활성 상태 보존·재생성 포함 |
| 독립 브라우저 | **53 PASS**. 기본 PDF 41개, 제한/timeout·실패 후 복구·보호 저장/오암호·양식 source·6종 목적지·SVG artwork 7종/할당 거절 포함 |
| Node·TypeScript | **2,473 PASS / 2 skip**, tsc 통과 |
| source/게시 preflight | 기존 source 18 + 게시/복구/출력 경계 14 = **32 PASS**. 실제 CLI dirty 거부·exit 2도 확인 |
| 문서·source 대조 | 변경 링크·diff 검사 통과. 메타데이터 기존 5개 파일의 20건은 T06과 동일하며 신규 오류 0건 |

예제와 rustdoc는 처음에 최소 의존 구성으로 선택해 PDF feature가 활성화되지 않았으므로,
일반 사용 안내와 같은 기본 의존 workspace에서 다시 검증했다. 테스트용 직접 backend 의존도
본체와 같이 default feature를 제외해 불필요한 legacy 기능 합산을 제거했다. 기대값을 바꾸지
않았고 최종 실행 원장은 위 통과 결과를 가리킨다. 초기 진단 로그는 별도로 보존했다.
복사한 JPEG 검사에는 기존 `unused_mut` 경고 1건이 있으며 검사 3개는 통과했다.

backend 자체 lock의 비활성 `crypto-bigint 0.7.5` yank 경고는 포장에서 관측됐다. 본체와
최종 Native/WASM 소비 lock에는 이 package가 없음을 확인했다. 지원하지 않는 backend
직접 기능 전체의 검증으로 확대하거나 본체 의존성을 무제한 갱신하지 않았다.

## 전쪽 시각 검증과 직접 판독

| 범위 | 문서 수 | 쪽수/실행 경로 | Native raw 최저 | WASM raw 최저 |
| --- | ---: | ---: | ---: | ---: |
| CFF·Type3·본문·중첩/clip·비공개 대조군 | 17 | 49 | 98.34664% | 98.79246% |
| 일반 HWPX Print 대조군 | 4 | 38 | 99.66991% | 99.17555% |
| JPEG PDF | 17 | 17 | 100% | 100% |

**각 실행 경로 38개 문서·104쪽**, 합계 208쪽이다. `all-page-gates.json`의 76개 결과 모두
완전 실행·독립 기준 쪽수 일치·90% 미만/누락/측정 불가 0쪽이다. 원본 TSV는
`visual/candidate/`, `visual/inline/candidate/`, `visual/jpeg/`에 Native/WASM을 나눠 보존했다.
전체 **208개 현재 PNG가 같은 실행 경로의 T06 PNG와 바이트까지 동일**하다.

현재 산출물의 Native CFF p1·Type3 p3, WASM Type3 p11, 양쪽 inline 객체 p13,
JPEG 반투명 출력의 compare/standalone overlay/review를 직접 읽었다. 같은 영역의 CFF 기호·
문자열과 Type3의 범위/도 기호·본문 간격도 2배 확대해 대조했다. 글자·내용 순서·줄바꿈·표/그림
포함·여백·alpha와 기존 출력이 유지됐다. 독립 PDF와의 기존 글꼴/획·보간 차이는 남아 있으며
엄격 RGB 100%나 모든 입력의 피델리티 완료로 설명하지 않는다. 패키징으로 생긴 새 차이는 없다.
판독 경로·해시는 `direct-review.json`, PNG 동일성은 `t06-page-byte-equivalence.json`에 있다.
비공개 입력과 식별 자료는 계속 로컬 검증에만 사용했다.

Native는 `release-test` 기능 검증이며 제품 release 성능 측정이 아니다. 독립 WASM 앱은
자신의 export 3개를 추가한다. on은 25,642,721 bytes
(`6322817ca0d29e0c1814c3fbfca980fad18a5074fa31c3299dfda305c6610d19`),
off는 3,176,830 bytes
(`bea8f9b0324cd32a3f6666019ee020acd19d0e394173bdc0ca3218ea3cd303e1`)다.
이를 표준 제품 bundle의 크기/속도 변화로 해석하지 않는다. 제품 pkg/Studio bundle 동기화와
최종 표준 fresh WASM·성능 확정은 기존 T08 범위다.

## 게시 인계와 다음 항목

공개 crates.io 조회 당시 14개 이름은 모두 404였다. `publish-preflight.json`은 당시 source의
실제 archive·owner/버전 공개 조회를 통과했고 `publication_authorized=false`를 유지한다.
이후 출력 경로의 부모 symlink 이탈도 막아 전체 32개 검사를 다시 실행했다. 미등록 관측은
이름 예약·owner 확보나 토큰 권한 확인을 뜻하지 않는다. 실제 첫 게시와 권한 확인·각 자식의
공개 index 반영·부모 dry-run·게시 뒤 소비·docs.rs 결과 확인은 수동 인계 순서대로 수행한다.
부분 실패는 같은 버전의 owner·checksum·비-yank 확인 뒤에만 건너뛰도록 준비했다.

T07 필수 잔여는 0건이다. **다음은 T08의 기존 source-unit 이관·최종 통합 게이트/성능**이며,
그 뒤 T09에서 최신 devel 기능 보존·제출 자료와 PR을 정리한다. 후속 source 변경이 archive에
영향을 주면 원장·digest와 해당 소비 증거를 갱신해야 한다. 필수 검증을 CI나 Draft로 넘기지 않는다.
원격 push·PR/comment·registry 업로드·owner 변경·토큰 설정·태그는 수행하지 않았다.
