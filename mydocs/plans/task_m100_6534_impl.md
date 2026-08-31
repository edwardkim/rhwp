# 구현계획 — Task M100 #6534 공공기관 XLSX 다운로드 자동 열기 오탐 제거

- **상위 수행계획**: [task_m100_6534.md](task_m100_6534.md)
- **이슈**: [#6534](https://github.com/edwardkim/rhwp/issues/6534)
- **작업 브랜치**: `task_m100_6534`
- **착수 기준**: `upstream/devel@9edf3d82ba47c539f4a5e0aa7fe3716671619bba`
- **계획 checkpoint**: `86bb071b8`
- **작성일**: 2026-08-31 KST
- **상태**: 2026-08-31 메인테이너 승인

## 1. 호출 계보와 책임 경계

현재 자동 열기 경로는 다음 두 판정을 서로 다른 시점에 수행한다.

1. Chrome/Firefox의 `downloads.onCreated`·`onChanged`가 다운로드 메타데이터를
   `rhwp-shared/sw/download-interceptor-common.js`에 전달한다.
2. 자동 열기가 승인되면 뷰어가 원본 URL을 다시 요청하고, Studio의
   `assertRemoteDocumentBytes()`를 거쳐 Rust `detect_format()`과 포맷별 parser로 들어간다.

이번 구현은 이 경계를 유지한다.

- **다운로드 adapter**는 탭을 열지 말지를 결정한다. 파일 본문을 읽거나 ZIP을 해제하지 않는다.
- **Studio 바이트 게이트**는 HWP/HWP3/HML과 HTML/XML을 빠르게 구분하고, ZIP은 아직 확정되지 않은
  컨테이너 후보로만 표시한다.
- **Rust parser**만 ZIP 중앙 디렉터리의 HWPX 필수 엔트리를 확인해 최종 `FileFormat::Hwpx`를 확정한다.

새 `FileFormat::Zip` variant는 추가하지 않는다. 이를 추가하면 CLI·서비스·진단·변환의 exhaustive match를
불필요하게 넓힌다. 일반 ZIP/OOXML은 기존 `FileFormat::Unknown`과
`UNSUPPORTED_FILE_FORMAT` 경로로 조기 거부한다.

## 2. 다운로드 결정 모델

### 2.1 공유 순수 함수

`rhwp-shared/sw/download-interceptor-common.js`에 다음 의미의 결정 함수를 둔다.

```js
classifyDownload(item, { metadataFinalized })
// => { action: 'intercept' | 'defer' | 'ignore', reason: string }
```

`reason`은 테스트와 진단을 위한 닫힌 문자열 집합으로 두고 사용자 데이터·URL 전체를 담지 않는다.
기존 `shouldInterceptDownload(item)`은 저장소 밖 복사본이나 후속 코드의 boolean 호환을 위해 남기되,
`metadataFinalized: true`인 분류 결과를 boolean으로 축약하는 wrapper로 만든다. Chrome/Firefox adapter는
wrapper가 아니라 새 결정 함수를 직접 사용한다.

### 2.2 우선순위 결정표

| 우선순위 | 관측 근거 | 결정 | 대표 reason |
| ---: | --- | --- | --- |
| 1 | URL/referrer가 DEXT5 등 `NON_REFETCHABLE_PATTERNS` | `ignore` | `non-refetchable` |
| 2 | 확정 파일명이 `.hwp/.hwpx/.hml` | `intercept` | `hwp-filename` |
| 3 | 확정 파일명이 알려진 비-HWP 형식 | `ignore` | `non-hwp-filename` |
| 4 | 구체적인 비-HWP MIME | `ignore` | `non-hwp-mime` |
| 5 | HWP URL/finalUrl/MIME가 있고 메타데이터 미확정 | `defer` | `provisional-hwp-evidence` |
| 6 | 같은 HWP 보조 근거가 있고 filename 확정 또는 terminal | `intercept` | `final-hwp-evidence` |
| 7 | HWP 근거 없음 | `ignore` | `no-hwp-evidence` |

알려진 비-HWP 확장자는 이번 보고 사례와 동일 계열로 한정한다.

- Excel: `.xls`, `.xlsx`, `.xlsm`, `.xlsb`, `.xlt`, `.xltx`
- Word: `.doc`, `.docx`, `.docm`, `.dot`, `.dotx`
- PowerPoint: `.ppt`, `.pptx`, `.pptm`, `.pot`, `.potx`
- 기타 명백한 문서 컨테이너: `.pdf`, `.zip`, `.odt`, `.ods`, `.odp`

MIME도 위 형식의 표준 `application/vnd.*`, `application/pdf`, `application/zip`만 음성 근거로 쓴다.
`application/octet-stream`과 빈 MIME은 중립이다. `.bin` 같은 임시 확장자는 확장자 없는 공공기관 HWP
호환을 위해 음성 근거로 쓰지 않는다.

파일명 신호는 비대칭으로 처리한다. `.hwp` 파일명은 잘못된 generic/non-HWP MIME보다 우선하고,
`.xlsx` 파일명은 URL이나 MIME의 HWP 힌트보다 우선한다. 사용자가 실제로 저장하게 된 이름을 가장
강한 의도 신호로 보는 정책이다.

## 3. 이벤트 adapter 설계

### Chrome `rhwp-chrome/sw/download-interceptor.js`

- `onCreated`에서는 신선한 id를 지금처럼 먼저 session state에 기록한다.
- 확정 HWP 파일명만 즉시 `intercept`한다. URL/MIME 단독 HWP 근거는 `defer`하고 탭을 열지 않는다.
- `onChanged` 재조회 뒤 `delta.filename.current` 또는 terminal state가 있으면
  `metadataFinalized: true`로 재분류한다.
- `finalUrl`만 바뀌고 filename/terminal이 아직 없으면 보조 근거만 갱신된 것이므로 계속 보류한다.
- 기존 `processingDownloadPromises`, settings fail-closed, download id당 1회 처리, `file://` HWP의
  cancel/erase best-effort는 그대로 둔다.

### Firefox `rhwp-firefox/sw/download-interceptor.js`

- Chrome과 동일한 분류 context를 전달한다.
- Firefox 고유의 `browser.storage.session` 부재 시 memory fallback과 terminal cleanup은 유지한다.
- in-flight 구현을 새로 이식하는 범위로 넓히지 않는다. 이번 변경은 공통 정책과 단계 정보 전달에 한정한다.

`rhwp-shared/sw/download-observer-state.js`는 이미 filename/finalUrl/terminal 재조회와 신선도 판정을
소유한다. 새 상태 필드가 필요하지 않으므로 변경하지 않는 것이 기본안이다. 구현 중 adapter가
`metadataFinalized`를 계산할 수 없다는 반증이 생길 때만 별도 계획 변경 승인을 받는다.

## 4. HWPX 구조 확정 설계

### 4.1 기준선 감사

`git ls-files -z '*.hwpx'` 기준 534개를 중앙 디렉터리만 읽어 확인했다.

| 분류 | 수 | 결과 |
| --- | ---: | --- |
| ZIP 매직 | 525 | 전부 `Contents/content.hpf`와 `Contents/header.xml` 동시 존재 |
| CFB 매직이지만 `.hwpx` 확장자 | 6 | 기존 `FileFormat::Hwp`로 수용 유지 |
| 공개 oracle fixture placeholder | 3 | 실제 문서가 아니므로 `Unknown` 유지 |

실제 ZIP HWPX의 최대 엔트리 수는 474개였다. 비밀번호 보호 표본
`samples/HWP5-password-123456.hwpx`도 두 필수 엔트리 이름이 중앙 디렉터리에 노출된다.

### 4.2 Rust 최종 확정

`src/parser/hwpx/reader.rs`에 payload를 압축 해제하지 않고 `Cursor<&[u8]>`와 `ZipArchive`로 다음 두
이름을 exact lookup하는 내부 helper를 둔다.

- `Contents/content.hpf`
- `Contents/header.xml`

`src/parser/mod.rs::detect_format()`은 `PK\x03\x04`만으로 `Hwpx`를 반환하지 않고 helper가 두
엔트리를 모두 확인한 경우에만 반환한다. 손상 ZIP, XLSX의 `[Content_Types].xml`·`xl/workbook.xml`,
일반 ZIP, 필수 엔트리 하나만 있는 ZIP은 `Unknown`이다. 새 포맷 variant나 payload 해제는 없다.

이로써 XLSX는 HWPX parser의 `MissingFile(Contents/content.hpf)`까지 들어가지 않고 자동 포맷 감지
경계에서 `UNSUPPORTED_FILE_FORMAT`으로 끝난다. 정상 HWPX parser는 기존 `HwpxReader`와 압축 해제
상한을 그대로 사용한다.

### 4.3 Studio 후보/확정 명칭 정정

`rhwp-studio/src/core/document-signature.ts`의 `DocumentByteKind`에 `zip`을 두고 ZIP 매직은 `hwpx`가
아니라 `zip`을 반환한다. `assertRemoteDocumentBytes()`는 HTML/XML 오류 페이지를 조기 차단하는
transport gate이므로 ZIP 후보는 Rust에 전달하되 HWPX로 확정했다고 주장하지 않는다.

Studio에서 별도 ZIP parser를 중복 구현하지 않는다. 브라우저에서 central directory parser를 새로
만들면 Rust와 판정 규칙이 이중화되고 ZIP64·손상 입력·보안 상한이 드리프트하기 때문이다.

Safari의 `rhwp-shared/security/file-signature.js`는 이번 다운로드 API 오탐 경로에 참여하지 않는다.
Safari 다운로드 API 추가나 별도 ZIP parser 도입은 범위 밖이다. 다만 Rust 최종 판정이 공통 WASM
경로에서 일반 ZIP을 거부하므로 늦은 `MissingFile` 오분류는 해소된다. 이 잔여 후보 게이트 차이는
최종 보고서에 명시한다.

## 5. 파일별 변경 allowlist

| 파일 | 변경 |
| --- | --- |
| `rhwp-shared/sw/download-interceptor-common.js` | 3상태 분류, 음성 확장자/MIME, reason, boolean 호환 wrapper |
| `rhwp-shared/sw/download-interceptor-common.test.js` | 전체 결정표와 #198/DEXT5 보호 계약 |
| `rhwp-chrome/sw/download-interceptor.js` | created/finalized context 전달 |
| `rhwp-chrome/sw/download-interceptor.test.mjs` | XLSX 충돌, 보류→확정, 중복·설정·file 계약 |
| `rhwp-firefox/sw/download-interceptor.js` | Chrome과 동일한 단계 context 전달 |
| `rhwp-firefox/sw/download-interceptor.test.mjs` | 공통 정책의 Firefox adapter 교차 회귀 |
| `src/parser/hwpx/reader.rs` | 비압축 HWPX 필수 엔트리 helper |
| `src/parser/mod.rs` | ZIP 매직 후보의 구조 확정과 합성 ZIP unit test |
| `rhwp-studio/src/core/document-signature.ts` | `zip` 후보와 HWPX 확정 책임 설명 |
| `rhwp-studio/tests/document-signature.test.ts` | 일반 ZIP/OOXML을 `hwpx`로 부르지 않는 계약 |
| `tests/threat_scan_contract.rs` | 기존 합성 HWPX builder의 필수 package entry 보완 |
| `tests/cases/threat_scan_cli_contract.rs` | 기존 CLI 합성 HWPX builder의 필수 package entry 보완 |
| `crates/rhwp-contracts/src/schema_registry.rs` | 병렬 전체 회귀에서 불안정한 정책 문서 runtime lookup을 compile-time linkage로 고정 |
| `rhwp-chrome/e2e/download-interceptor.test.mjs` (신규) | 격리 packaged Chrome 실제 download/tabs 계약 |
| `rhwp-chrome/package.json` | 독립 download E2E 실행 script |
| `mydocs/manual/browser_extension_dev_guide.md` | metadata 우선순위와 ZIP 후보/Rust 확정 설명 |
| `mydocs/manual/chrome_edge_extension_build_deploy.md` | 새 packaged download E2E 실행법 |
| `mydocs/working/task_m100_6534_stage*.md` | 단계별 red/green/교차검증 증적 |
| `mydocs/report/task_m100_6534_report.md` | 최종 결과, 성능, 잔여 위험 |

`rhwp-shared/sw/download-observer-state.js`, manifest, 배포 버전, dist 산출물은 변경하지 않는다.
allowlist 밖 제품 파일이 필요해지면 먼저 원인과 영향도를 문서에 반영하고 계획 변경 승인을 받는다.

### 5.1 Stage 4 계획 변경 — threat-scan 합성 HWPX 정합성

Stage 4 전체 release-test에서 threat-scan 5건이 새 detector에 의해 `Unknown`으로 거부됐다. 두 기존
test helper가 HWPX라고 명명한 ZIP에 `Contents/header.xml`을 넣지 않았고, 일부 사례는
`Contents/content.hpf`도 생략한 것이 원인이다. 실제 추적 HWPX 525건과 비밀번호 표본은 두 엔트리를
모두 가진다는 Stage 3 감사와도 어긋난다.

제품 detector나 threat scanner를 확장자 기준으로 느슨하게 만들지 않는다. 위 두 기존 test helper가
호출될 때 누락된 필수 엔트리만 최소 XML로 보완해, 위협 payload·finding cap·출력 계약은 그대로 두고
합성 컨테이너를 유효한 HWPX 후보로 만든다. 테스트 수와 generated suite·manifest는 바꾸지 않는다.
메인테이너는 2026-08-31 이 allowlist 확장과 fixture 정정을 승인했다.

### 5.2 Stage 4 계획 변경 — schema policy 문서 연결 결정성

fixture 정정 뒤 전체 8,881건은 threat-scan 5건을 모두 회복했지만,
`rhwp-contracts::schema_registry::tests::policy_path_points_to_existing_document` 한 건이 전체 병렬 실행에서만
두 번 실패했다. 같은 release-test binary의 단독·crate 단위 실행은 모두 통과했고, 대상 파일의 inode,
mtime과 Git 상태도 변하지 않았다. #6534 제품·fixture 변경과 인과관계는 없다.

내부 비배포 crate의 이 테스트가 runtime `Path::exists()`에 의존할 이유가 없다. registry의 `policy` 값을
canonical 상대경로와 exact 비교하고, 같은 파일을 `include_str!`로 compile-time 연결한다. 문서가
누락·이동되면 컴파일이 실패하고 registry 문자열만 drift하면 assertion이 실패하므로 기존 불변식을 더
결정론적으로 유지한다. 테스트 수와 제품 API는 바꾸지 않는다. 메인테이너는 2026-09-01 이 두 번째
allowlist 확장과 정정을 승인했다.

## 6. 구현 순서와 단계별 중단점

### Stage 1 — red 계약

1. 공유 분류 테스트에 XLSX filename + `.hwp` URL, XLSX filename + HWP MIME, redirect 충돌,
   generic filename + HWP 보조 근거의 미확정/확정 조합을 추가한다.
2. Chrome/Firefox adapter 테스트에 `onCreated` 보류와 `onChanged` 확정 동작을 추가한다.
3. Rust 합성 ZIP 테스트에 HWPX 두 필수 엔트리, XLSX 엔트리, 필수 엔트리 하나만 있는 ZIP을 추가한다.
4. Studio 테스트에 ZIP=`zip` 후보 계약을 추가한다.
5. 현행 코드에서 의도한 red 실패만 발생하고 기존 보호 테스트는 green인지 기록한다.

제품 코드는 이 단계에서 바꾸지 않는다. red 결과와 테스트 설계 승인을 받은 뒤 Stage 2로 간다.

### Stage 2 — 다운로드 분류와 adapter

1. 공유 3상태 분류와 boolean wrapper를 구현한다.
2. Chrome/Firefox adapter에 단계 context를 연결한다.
3. 공유·observer·두 adapter focused test를 통과시킨다.
4. `.hwp` 즉시 열기, extensionless terminal fallback, XLSX 무간섭을 결과표로 남긴다.

### Stage 3 — ZIP 후보와 HWPX 최종 확정

1. Rust helper와 `detect_format()`을 최소 변경한다.
2. Studio의 `hwpx` 매직 단정을 `zip` 후보로 정정한다.
3. 합성 XLSX가 `Unknown`/`UNSUPPORTED_FILE_FORMAT`, 정상·비밀번호 HWPX가 `Hwpx`인지 확인한다.
4. 추적 ZIP HWPX 525개를 새 detector로 감사하고 실패 0을 요구한다.
5. 변경 전후 525개 구조 판정 시간을 같은 release binary·warm 조건에서 반복 측정해 median/p95를
   최종 보고서에 기초 데이터로 남긴다. payload 압축 해제나 파일 내용 공개는 하지 않는다.

### Stage 4 — packaged browser와 전체 변경 범위 검증

1. 확장 및 Studio focused test/build를 수행한다.
2. 새 E2E가 자체 HTTP fixture와 임시 profile/download directory만 사용하도록 만든다.
3. 실제 Chrome에서 다음을 확인한다.
   - 정상 `.xlsx`: 다운로드 유지, rhwp 탭 0
   - URL이 `.hwp`이나 filename/MIME이 XLSX: 다운로드 유지, rhwp 탭 0
   - 확정 `.hwp`: rhwp 탭 1
   - extensionless URL + HWP MIME/본문: terminal 뒤 rhwp 탭 1
   - 각 download id의 탭은 최대 1
4. Rust 필수 lint 묶음, Docker WASM build, dist 계약을 통과시킨다.

### Stage 5 — 문서·정산

1. canonical browser 문서의 “ZIP 매직=HWPX” 표현을 후보/확정 모델로 고친다.
2. 계획 대비 결과, 실제 공공기관 원응답 미확보 여부, Safari 후보 게이트 차이, 성능 측정을 보고한다.
3. 변경 allowlist와 generated artifact 미포함을 확인하고 code candidate를 준비한다.
4. push, PR 생성, self-review, merge는 각각 별도 승인 게이트로 둔다.

## 7. 검증 명령

### JavaScript/TypeScript와 확장

```bash
node --test rhwp-shared/sw/download-interceptor-common.test.js
node --test rhwp-shared/sw/download-observer-state.test.js
node --test rhwp-chrome/sw/download-interceptor.test.mjs
node --test rhwp-firefox/sw/download-interceptor.test.mjs
node --test rhwp-studio/tests/document-signature.test.ts
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
node --test scripts/frontend-extension-dist.test.mjs
npm --prefix rhwp-chrome run test:e2e:download
```

packaged E2E는 사용자 Chrome profile을 쓰지 않고 외부 네트워크도 사용하지 않는다. 테스트 종료 시
임시 profile·download를 `finally`에서 정리하고, 서버가 남지 않았는지 확인한다.

### Rust source 변경 필수 gate

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
node scripts/rust-unit-test-tiers.mjs --check
cargo test --locked --lib parser::tests::test_detect_format --target-dir target/pr-review
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
  --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
```

Docker WASM은 `mydocs/manual/dev_environment_guide.md`의 정본 명령을 사용한다. generated integration
suite·manifest와 진단용 unit tier inventory는 검증 뒤 복원하고 stage하지 않는다. 전체 nextest와 PR 제출
직전 gate는 Stage 4 결과 승인 뒤 별도 진행한다.

## 8. 커밋 경계

1. **Stage 1**: red 계약과 stage 보고서
2. **Stage 2**: 공유 다운로드 정책·Chrome/Firefox adapter와 green 결과
3. **Stage 3**: Rust HWPX 구조 확정·Studio 후보 명칭과 감사 결과
4. **Stage 4**: packaged E2E·canonical manual·최종 검증 보고서

각 단계는 해당 exact path만 stage한다. 원격 push와 PR 생성은 포함하지 않는다.

## 9. 완료 조건

1. #6534의 XLSX 메타데이터 충돌 두 종류가 Chrome/Firefox adapter와 packaged Chrome에서 탭 0으로 고정된다.
2. `.hwp/.hwpx/.hml` 확정 파일명과 extensionless HWP terminal fallback은 유지된다.
3. #198 DEXT5, #1498/#1515 신선도, #2656 설정 fail-closed, `file://` 억제, download id당 1회가 회귀하지 않는다.
4. 일반 OOXML ZIP은 `FileFormat::Hwpx`가 아니며 HWPX `MissingFile` 전에 비지원 형식으로 끝난다.
5. 추적된 실제 ZIP HWPX 525개와 비밀번호 보호 표본이 모두 새 구조 게이트를 통과한다.
6. focused, native/WASM lint, Docker WASM, 확장 build/dist, packaged browser gate가 모두 성공한다.

## 10. 승인 뒤 첫 작업

구현계획 승인 뒤 Stage 1의 테스트 파일만 수정해 현행 코드에서 red 증적을 만들고 보고한다. 제품 코드
수정은 red 계약 결과를 메인테이너가 확인한 다음 단계 승인 뒤 시작한다.
