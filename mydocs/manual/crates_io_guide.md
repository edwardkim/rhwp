---
kind: guide
status: active
canonical: mydocs/manual/publish_guide.md
last_verified: 2026-10-28
---

# crates.io 소비 계약과 초회 수동 게시

이 문서는 #8174의 확정된 MD01–MD09를 실행 가능한 계약으로 정리한다.
본체의 준비 버전은 **0.9.0**, 최소 Rust는 **1.93.1**이며 실제 registry 게시 일정은
제품 릴리즈와 별도로 결정한다. 현재 게시 완료를 뜻하지 않는다. 게시·보안 업데이트·부분 실패
복구 책임자는 **edwardkim**이다. 기존 Release Binary의 npm·VS Code·Open VSX 네 채널은
그대로 유지하며 crates.io를 tag workflow에 자동 연결하지 않는다.

## 사용자와 지원 계약

주 대상은 Rust 문서 처리 애플리케이션 개발자, 다음은 CLI 자동화와 Rust WASM 앱 개발자다.
설치·parser/IR·core/service 편집·명시적인 HWP/HWPX 저장·저장 손실 보고 예제와 0.9.x
호환 계약은 [패키지 README](../../packaging/crates-io/README_PACKAGE.md)에 함께 동봉한다.
0.9.x 패치는 공식 핵심 진입점의 소스 호환을 유지하고, 깨지는 API 변경은 다음 minor와
이행 안내로 제공한다. 내부 renderer/layout, JS 제품 facade와 보정 fork의 직접 API까지
이 계약으로 확대하지 않는다. 출력 보정에 따른 쪽수·픽셀·오류 문구는 별도 검증 대상이다.

| 소비 환경 | 공식 검증 구성 | 경계 |
| --- | --- | --- |
| Rust Native | Rust 1.93.1, Linux/WSL2 `x86_64-unknown-linux-gnu`; 기본·최소·PDF 제외 구성 | parser/IR·service/core·저장, 오류 표준 연동. `default-features=false`도 parser-only는 아님 |
| Native CLI | 같은 환경; `--bin rhwp`, 기본 및 `native-skia`, PDF 제외 빌드 | PNG·direct PDF·Sixel·Kitty는 `native-skia` 필요. 한글 출력은 실제 폰트를 공급 |
| Rust WASM | Rust 1.93.1, `wasm32-unknown-unknown`, facade on/off, 최적화 완료 후 Chromium 실행 | API off는 소비 앱의 start를 보존. wasm-bindgen/JS는 같은 빌드의 glue 사용 |
| Studio | 브라우저의 제품 Worker·호스트 경계 | F07 제한된 양식 단추 실행·F08 암호화 복구는 Studio host 기능이며 Rust 일반 실행기가 아님 |
| 다른 OS/target | 기존 GitHub Release 지원과 구분 | 이번 crates.io 독립 소비 검증으로 미검증 OS·하위 MSRV를 약속하지 않음 |

PDF·JBIG2·JPEG 2000·ICC는 기본 기능이다. xref 복구·SVG 글리프·목적지 링크·양식·보호 복구도
명시된 기능 계약 안에서 제공한다. 모든 codec/profile·JScript·OS 앱 실행을 지원하는 뜻은 아니다.
[보안 기능 경계](security_feature_limits.md), [PDF 자원 정책](pdf_resource_policies.md),
[Studio 기능](studio_document_features.md), [CLI](cli_commands.md)를 함께 따른다.

비신뢰 PDF는 요청 한도와 격리 host를 함께 적용하는 지원 대상이다. Linux CLI의 `import-pdf`는 별도 자식의
Landlock·seccomp·CPU/주소 공간·시간 감독을 사용하고 필요한 커널 기능이 없으면 실패한다.
Rust 서버의 직접 동기 호출에는 OS 격리가 자동 추가되지 않는다. WASM은 실제 Worker 종료와
재생성 경로를 사용한다. 동기 main thread는 협력적 검사만 가능하며 즉시 선점하지 못한다.
`parserAllocationBoundsProven=false`는 유지한다. 예약 카운터를 전체 live heap/RSS의 증명으로
설명하지 않으며, 원본 처리·재열기·편집·저장에 연결된 요청 정책을 host가 유지해야 한다.

## 패키지와 순서

정본 목록은 [source-policy.json](../../packaging/crates-io/source-policy.json)이다.
내부 패키지도 공개 의존성으로 배포하되 전체 직접 API의 안정성을 약속하지 않는다.
모든 후보 manifest는 `publish=["crates-io"]`와 `rust-version="1.93.1"`을 선언한다.

| 순서 묶음 | 이름 | 버전 |
| --- | --- | --- |
| 1 | rhwp-codec-resource, rhwp-contracts, rhwp-ooxml-chart, rhwp-password-crypto | 0.1.0 |
| 1 | rhwp-jpeg-decoder | 0.3.2-rhwp.1 |
| 1 | rhwp-jpeg-encoder | 0.7.1-rhwp.1 |
| 1 | rhwp-svg2pdf | 0.13.0-rhwp.1 |
| 1 | rhwp-zune-jpeg | 0.5.15-rhwp.1 |
| 2 | rhwp-hayro-ccitt, rhwp-qcms | 0.3.0-rhwp.1 |
| 2 | rhwp-hayro-jpeg2000 | 0.4.0-rhwp.1 |
| 3 | rhwp-hayro-jbig2 | 0.3.0-rhwp.1 |
| 4 | rhwp-pdf-import-backend | 0.3.78-rhwp.1 |
| 5 | rhwp | 0.9.0 |

같은 묶음 안에서는 선후 제약이 없다. 실제 순서는 도구의 topological order를 따른다.
2026-10-28 공개 API 조회에서 14개 이름은 모두 404였다. 이는 당시 미등록 관측이며 이름 예약·
owner 권한·향후 사용 가능성의 증명이 아니다. 새 이름의 owner는 첫 게시 뒤 공개 조회로 확인한다.
게시 직전 registry 정책·이름과 권한을 다시 확인한다.

## 로컬 검증과 읽기 전용 사전 점검

Git source 검사는 다음처럼 실행한다. source 후보 archive와 Cargo `.crate`는 구분한다.

```sh
python3 scripts/crates_io_source_preflight.py \
  --ref HEAD --output-dir output/pr-review/issue8174/source-preflight
python3 -m unittest scripts.tests.test_crates_io_source_preflight \
  scripts.tests.test_crates_io_publish_preflight -v
```

최종 `.crate` 14개는 깨끗한 동일 source에서 `cargo package --no-verify --registry crates-io`로
생성하고 파일·고지·내장 자료·정규화 manifest·VCS SHA·checksum을 대조한다. 자식이 아직 공개되지
않았으면 검증 전용 loopback sparse registry로 연결해 실제 포장·독립 소비한다. 사용자 전역
Cargo 설정/cache를 수동 수정하지 않고 소비 workspace에 repository path/patch를 넣지 않는다.
빌드는 공용 `target/pr-review`에서 순차 수행한다. 검증 registry에는 업로드 API를 제공하지 않는다.
`--no-verify`의 archive 생성과 실제 consumer 빌드·실행을 각각 기록한다.

### 기존 출력 파일과 gzip 무결성

같은 공용 target을 사용하는 Cargo 작업이 모두 끝난 뒤, 이번에 생성할 정확한
`target/pr-review/package/<name>-<version>.crate` 파일만 실행별 ignored `output/`의
`previous-archive/`로 이동해 보존한다. 새 출력 파일이 없는 상태에서 포장한다.
target 전체·package 디렉터리·Cargo cache를 삭제하지 않으며 다른 작업의 산출물을 이동하지 않는다.
`cargo package`, `cargo publish --dry-run`, 실제 `cargo publish`로 archive를 다시 생성할 때마다
이 조건을 확인하고, 새 원장의 bytes·SHA-256과 아래 읽기 전용 점검을 다시 대조한다.

T08의 Cargo 1.93.1 검증에서 기존 출력 경로를 재사용한 archive 6개에 gzip 뒤의 잔여 바이트가
검출됐다. tar 원본 파일·원장 checksum이 일치해도 gzip footer 이후의 전체 파일 검사가 필요하다.
사전 점검은 단일 gzip 스트림의 종료·CRC·잔여 바이트와 기존 tar/manifest 계약을 함께 검사한다.
잔여 데이터·추가 gzip member·잘린 footer가 있으면 새 파일에서 재포장하고 새 origin으로 소비한다.
이미 내려받은 Cargo cache의 같은 버전 파일이나 index checksum을 덮어쓰지 않는다.

실행 원장은 `source_sha`, `packages[]`의 `name`·`version`·`bytes`·`sha256`을 담는다.
그 원장과 archive를 읽어 공개 registry만 조회하는 사전 점검은 다음과 같다.

```sh
python3 scripts/crates_io_publish_preflight.py \
  --ledger output/pr-review/issue8174/t08-completion-20261028/final-package/packages.json \
  --archive-dir output/pr-review/issue8174/t08-completion-20261028/final-package/crates \
  --output output/pr-review/issue8174/t08-completion-20261028/final-package/publish-preflight.json
```

HEAD가 원장 source와 다르거나 dirty면 재포장한다. digest·version·VCS·MSRV·게시 guard와
의존 순서를 검사하고, 공개된 동일 버전은 owner `edwardkim`·checksum 일치·비-yank일 때만
`already-present`로 분류한다. 403·429·5xx와 checksum/owner 불일치는 중단하며 미등록으로
취급하지 않는다. 실패하면 이전 성공 보고서도 실패로 덮어쓴다. 이 도구는 토큰을 읽거나
업로드하지 않고 `publication_authorized=false`를 유지한다. 로컬 통과는 게시 승인이 아니다.

## 실제 게시 승인 뒤 수동 실행

1. 메인테이너가 게시 일정·source SHA·14개 버전/digest·계정 권한을 확정한다. 기존 네 채널
   릴리즈와 독립된 승인이다. 토큰 값은 명령 인자·문서·로그로 전달하지 않는다. 제한된 권한의
   Cargo credential provider/환경을 사용하며 스크립트로 credentials 파일을 읽어 출력하지 않는다.
2. 깨끗한 승인 source에서 실제 crates.io 설정으로 다시 포장·독립 소비·위 사전 점검을 완료한다.
   loopback source replacement를 제거했는지 확인한다. 최신 공개 index에 자식이 보인 뒤에만
   부모 dry-run을 실행할 수 있다. 미게시 자식을 무시하거나 path fallback으로 성공시키지 않는다.
3. 의존 순서대로 정확한 한 패키지만 선택해 다음 명령을 수행한다. `--allow-dirty`·`--no-verify`로
   게시 검증을 생략하지 않는다. root workspace의 다른 도구를 함께 게시하지 않는다.

   ```sh
   cargo publish --locked --registry crates-io --manifest-path <Cargo.toml> -p <name> --dry-run
   # 재생성된 archive의 version/digest를 승인 원장과 대조한 뒤, 게시 승인 범위 안에서 실행
   cargo publish --locked --registry crates-io --manifest-path <Cargo.toml> -p <name>
   ```

4. 각 업로드 뒤 공개 exact-version index·archive checksum·owner를 확인하고 실행 원장에
   source/digest/시각/결과를 기록한다. 모든 자식이 실제 index에서 보인 다음 부모로 진행한다.
   로컬 registry 검증은 실제 crates.io의 parent dry-run 완료와 구분한다.
5. 최종 공개 버전으로 path 없는 Rust/CLI/WASM 소비를 다시 실행하고 완료 보고한다. 공개된
   패키지의 docs.rs 빌드 결과도 확인한다. 로컬 rustdoc 성공만으로 실제 docs.rs 서비스 성공을
   선언하지 않는다. 첫 게시 후 owner가 edwardkim인지 확인하며 owner/team 추가는 별도 승인한다.

## 부분 실패·보안 업데이트

시간 초과라도 업로드가 성공했을 수 있으므로 재실행 전에 exact version을 조회한다.
owner·checksum이 일치하는 비-yank 버전만 건너뛰고 남은 자식→부모 순서를 재개한다.
같은 버전을 덮어쓰지 않는다. digest가 다르면 즉시 중단하고 새 보정 버전·의존성 갱신·소비
재검증을 준비한다. 이름 충돌도 임의 rename으로 우회하지 않고 graph 변경을 메인테이너에게 보고한다.

잘못 공개한 버전의 삭제를 복구 수단으로 가정하지 않는다. 필요하면 별도 승인 후
`cargo yank --registry crates-io --version <version> <name>`를 수행한다. yank는 이미 설치된
사용자의 파일을 제거하거나 기존 lockfile의 소비를 막지 않는다. 보안 문제는 edwardkim이
`SECURITY.md`의 신고 경로로 접수하고 영향 feature/target·버전·보정 릴리즈·migration을 공지한다.
비밀 토큰이 노출되면 토큰 폐기·교체도 별도 권한으로 수행하며 값을 기록하지 않는다.

이후 OIDC/trusted publishing 연결은 별도 운영 변경이다. 초회 준비를 이유로 workflow·secret·
owner·태그·업로드를 자동 변경하지 않는다. 실제 게시 전 조건은 당시
[Cargo publish 계약](https://doc.rust-lang.org/cargo/commands/cargo-publish.html),
[registry index 계약](https://doc.rust-lang.org/cargo/reference/registry-index.html),
[Cargo SemVer 호환 안내](https://doc.rust-lang.org/cargo/reference/semver.html)를 함께 확인한다.
