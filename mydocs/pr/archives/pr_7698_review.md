---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-11
---

# PR #7698 — 암호 HWP5의 ViewText 본문 선택

## 최종 판정

**병합 완료 — 제출 head `34e1276015eb892a199cfa576b1d90fdd194135b`의 CI·CodeQL 및 로컬 보정 검증 성공 후 원 기여 commit을 보존하는 merge로 devel에 반영했다. merge SHA `4020c0428defba3459c22879f38facd8019da2d7`, 병합 시각 `2026-10-11T06:42:52Z`. #7674는 `2026-10-11T06:43:20Z`에 completed로 자동 종료됐다.**

scari님의 원 기여 commit `1489ca1d95e3d6e6b888870d3b14cf240866ef00`은 ancestor로 보존한다. 사용자가 지시한 공개 #5169 변경 추적 표시 결함도 이번 보정 범위에 포함했다. HWP/HWPX 모두 독립 한컴 Print와 같은 6쪽이고 Native/fresh WASM 각각 HWP 최저 91.69413%, HWPX 및 XML 형식 공백 대조군 최저 90.87964%다. 전체 쪽수 일치, 누락·90% 미달 없음. 새 대표 2·3·4쪽 review gate는 passed이고 직접 판독에서 숨긴 내용 제외·빈 줄/제목의 새 쪽 소속·본문 및 표 관계를 확인했다.

저장 표시 설정을 공통 visible IR에 적용하며, 바뀌지 않은 문단의 유효한 저장 LineSeg를 보존한다. 미편집 HWP 저장본의 한컴 Print는 정상 6쪽이며 원본 PDF와 전쪽 raster가 동일하다. 보기 전환 UI, 변경 수락/거부, 서식 변경의 원본 복원, HWPX 이력 전체 저장·재열기는 완료 범위에 포함하지 않는다. 비공개 기업 원본은 제공되지 않아 공개 실제 암호 대조군과 합성 Section1 재현의 한계를 구분했다. [보정·검증 기록](pr_7698_review_impl.md)과 [대표 PNG](../assets/pr7698/README.md)를 연결한다.

원 head의 CodeQL high 경고 69건 실패는 아래 최초 진단으로 보존한다. 새 분석의 Rust SARIF 결과는 0건이며 `rust/cleartext-logging`의 실제 평가·해석 로그를 확인했다. 쿼리 제외·일괄 dismissal 없이 새 head의 CodeQL 보안 gate가 성공했다. 아래 접수 당시 상태와 마지막 절의 최종 원격 결과를 구분한다.

## 최초 접수 당시 트리아지

| 항목 | 확인 결과 |
| --- | --- |
| PR / 작성자 | [#7698](https://github.com/edwardkim/rhwp/pull/7698) / scari, 기존 기여자 |
| 관련 이슈 | [#7674](https://github.com/edwardkim/rhwp/issues/7674), OPEN; 본문 `Closes #7674`. #3405·#5169·#6884 참조 |
| 원 head | `1489ca1d95e3d6e6b888870d3b14cf240866ef00` |
| PR base | devel / `79abd9e49675f4286eaed38197f7e837afb744e7` |
| 최신 검토 base | `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47`; 검토 종료 전 fetch로 동일성 재확인 |
| 로컬 검증 후보 | `a47c49a6560c686c890800b6efc66a095ea6dcc8`; 최신 base에 원 기여 commit을 cherry-pick. 두 변경 파일의 내용은 원 head와 동일 |
| 작업공간 | 주 작업공간 `review/scari-pr7698-20261011`; 별도 worktree 없음 |
| 트리아지 | assignee=scari / reviewer=edwardkim / milestone=v1.0.0 / labels=bug, hwp5, rust; REST 재조회 확인 |
| 변경 | `src/parser/mod.rs`, `tests/cases/issue_7674_encrypted_viewtext.rs`; +285/-28 |
| 접수 당시 상태 | OPEN, non-draft, mergeable=true / unstable. 최신 base와 원 head merge-tree 성공: `cee0d60ab6f145fdb5acdf6c7432393a5002c135` |

maintainer 일반 경로와 intake·local validation·visual fixture·post-merge 가이드를 적용했다. 로컬 통합 후보는 원 PR head나 원격 CI 대상과 구분한다. 기여자 fork를 갱신하지 않았다.

## 신고와 실제 호출 경로

신고자는 rhwp-core WASM 0.8.7의 `openWithPassword`에서 기업 내부 문서를 열 때 `StreamNotFound("Section1")`을 확인했다. FileHeader 속성 `0x00024003`은 압축·암호·변경 추적·색인 필드를 포함하지만 배포용 비트는 없다. `BodyText/Section0`과 `ViewText/Section0,1`이 있으며 `BodyText/Section1`은 없다. 한컴에서 변경 추적을 모두 승인하고 저장하면 ViewText가 없어지고 BodyText/Section1이 생성되어 열렸다는 신고다. 비공개 실제 원본은 제공되지 않았다.

base의 strict/lenient 파서는 section count에 ViewText를 포함하지만, 암호 분기에서는 `read_body_text_section_raw_limited`만 호출한다. 따라서 열어야 할 Section1이 있다는 판단과 해당 본문을 찾는 저장소가 어긋난다. 비암호 경로에 이미 있던 #5169의 ViewText 우선 정책도 암호 분기에는 적용되지 않는다.

원 head의 실제 소비 경로는 다음과 같다.

1. `src/parser/mod.rs`의 `decode_password_rendered_viewtext`(호출 856·985행)가 ViewText 스트림을 기존 암호 복호화·제한 해제 경로로 읽고, 첫 레코드가 PARA_HEADER일 때만 본문으로 반환한다.
2. strict/lenient 각 암호 분기는 이 결과를 사용하고, 스트림 없음·stub·WrongPassword 결과이면 기존 BodyText 복호화 경로를 사용한다. 그 밖의 CFB 오류 및 출력 크기 제한 오류는 전파한다.
3. 선택한 section data는 기존 레코드 파싱·문단 IR·document 생성 경로로 전달된다. DocInfo 단계의 암호 검증을 우회하지 않는다. 배포용 분기·CFB 경로 조회·암호 알고리즘·공개 API는 변경하지 않는다.
4. 이 PR은 조판 측정값·배치값을 직접 변경하지 않지만 선택하는 본문 IR이 달라진다. 실제 문서의 내용·쪽 구성에 영향을 줄 수 있으므로, 구조 검사 통과만으로 한컴 출력 일치를 주장하지 않는다.

특정 문서 ID나 속성 수치 예외는 없다. 기존 비암호 정책의 PARA_HEADER 판별을 공유하며, 암호 스트림 처리라는 형식 계약에 분기를 둔다. 첫 레코드 판별은 전체 저장 조판 정보가 유효하다는 증명이 아니다.

## 원 head 접수 당시 검증 입력과 결과

원문 실행 증적은 ignored `output/pr-review/pr7698-20261011/`에 보존한다. Rust 산출물은 공용 `/home/edward/mygithub/rhwp/target/pr-review`를 재사용했다. 테스트·baseline·허용치는 수정하지 않았다.

| 검증 | 결과와 증적 |
| --- | --- |
| 최신 base 음성 대조 | 원 PR의 테스트 파일을 그대로 복사해 base 라이브러리에 연결한 `rustc --test` 실행: 3 PASS / 3 FAIL. Section1 누락, ViewText 대신 오래된 BodyText 선택, ViewText 제한 오류 미노출을 확인. `base-build.log`, `base-test-build.log`, `base-tests.log` |
| 후보 집중 회귀 | `cargo test --locked --profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test regression_suite_020 issue_7674_encrypted_viewtext -- --nocapture`: 6 PASS / 0 FAIL (`head-tests.log`) |
| 기존 실제 암호 문서 대조군 | 같은 명령의 `--test regression_suite_001 hwp5_password_fixture`: 2 PASS. 비밀번호 오류·내용/개체 보존·저장 왕복 및 CLI 종료 계약 (`hwp5_password_fixture.log`) |
| 기존 CFB 경로 대조군 | `--test regression_suite_005 lenient_cfb_path_lookup`: 6 PASS (`lenient_cfb_path_lookup.log`) |
| 기존 비암호 ViewText 대조군 | `--test regression_suite_024 issue_5169_prefer_viewtext`: 1 PASS (`issue_5169_prefer_viewtext.log`) |
| build / manifest | base 및 후보 `cargo build --locked --profile release-test --lib` PASS. `node scripts/rust-test-suite-manifest.mjs --prepare`, `--check --base-ref 90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47` PASS; `git diff --check` PASS |
| 원 head CI | [Build & Test 37917363490](https://github.com/edwardkim/rhwp/actions/runs/37917363490) success.Rust lint 및 archive shard 모두 성공. 별도 WASM Build는 skipped |
| 미실행 범위 | 로컬 전수 Rust 회귀·lint 묶음, 새 Docker WASM 빌드와 실제 브라우저 openWithPassword, 비공개 원본, 독립 한컴 Print PDF/Native/fresh WASM Visual Sweep |

새 집중 검사는 런타임 생성 CFB를 사용한다. 암호·압축 여부, strict/lenient, BodyText 공존/누락, 잘못된 암호, ViewText 없음/stub, 일반 저장 왕복, 256 MiB 출력 상한을 검사한다. lenient 입력은 FAT를 의도적으로 손상한 합성 대조군이며 정상 한컴 생성본으로 간주하지 않는다. 공개 재현용 비밀번호와 단순 본문만 사용하고 기업 문서 내용은 사용하지 않았다.

검증 입력 커밋 확인:

| 실제 입력 | SHA-256 / 출처와 commit 확인 |
| --- | --- |
| `tests/cases/issue_7674_encrypted_viewtext.rs` | `09ac954bbc6f01346278fce320d64ea20e26d16b4781627ec2c2b49a594b7258`; 원 head·후보·base 재현용 ignored 복사본 동일. CFB 바이트는 이 코드가 메모리에서 생성 |
| `samples/hwp3-sample16-hwp5-2024-password-123456.hwp` | `59d4bed335b9552fe78fa68d2a56f7cfa3d586bcdeaaba839af80df13f3e08dc`; 기존 공개 한컴 EncryptVersion 4 대조군, 원 head 포함 파일과 실제 파일 일치 |
| `samples/issue5169_viewtext_changetracking.hwp` | `c003cdb0dd9aac5fda9c82d4f3be20a327522cf2deb680dc25134c7c6c8c3462`; 기존 PRISM 공개 변경 추적 대조군, 원 head 포함 파일과 실제 파일 일치 |

## CI 실패 원인

[실패한 CodeQL check 113782152132](https://github.com/edwardkim/rhwp/runs/113782152132)의 정확한 head는 원 head `1489ca…`다. [CodeQL Actions run 37917363606](https://github.com/edwardkim/rhwp/actions/runs/37917363606)의 분석 작업 4개는 모두 성공했지만, 업로드한 결과를 판정하는 별도 GHAS check가 **`rust/cleartext-logging` high 69건**으로 실패했다. 분석 작업 성공과 보안 gate 통과는 서로 다른 결과다. JS/TS·Python의 devel 기준 configuration 누락 경고 2건은 별도 coverage 문제이며 69건의 실패 사유와 구분한다.

SARIF 분석 ID `1922558100`, ref `refs/pull/7698/merge`, 분석 당시 merge SHA `c38e0e5e38826e2b48e0a42ccff8ea1a4e9170e9`를 확인했다. CodeQL 2.27.2 / rust-queries 0.1.44 / rust-all 0.2.23이다. 이 merge SHA는 최신 base와의 로컬 통합 후보가 아니다.

69건의 207개 code flow 모두 새 `decode_password_rendered_viewtext(...)` **반환값**에서 시작한다. strict 호출 856행 109개, lenient 호출 985행 98개이며 동일 IR을 거쳐 기존 로그로 퍼진다. 새 helper는 `1489ca…`에서 처음 도입됐고 로그 sink는 그 이전 코드다. 경고 최초 생성 시각은 `2026-10-09T10:40:17Z`다. sink 원 commit·저자·시각은 `production-sink-origins.txt`, 현재 head 흐름은 `codeql-flow-summary.json`에 남겼다.

| 경고 범위 | 실제 확인 내용 / 판정 한계 |
| --- | --- |
| 기존 테스트 로그 60건 | wasm_api 33, serializer/cfb_writer 19, parser/control 5, parser/body_text 3. 모듈 선언 모두 `#[cfg(test)]`. 문서 본문 일부를 출력하는 테스트도 있어 전부 숫자 로그라고 단정하지 않는다. 테스트 경로라는 사실과 실제 민감 문서 logging 여부는 구분 |
| production 로그 9건 | #210·#221·#230–236. 페이지 수·시간·줄/표/그림 좌표·높이·인덱스·bool만 출력. `run_tacs`는 `(usize, f64, usize)`, `table_meta`는 인덱스 tuple. 암호 문자열·키·원문 본문 출력은 이 9개 sink에서 확인되지 않음 |
| 공통 source | `src/parser/crypto.rs`에서 기존 decrypt 함수로 변환한 문서 바이트. `crates/rhwp-password-crypto/src/lib.rs`의 password는 키 파생에 쓰며 반환값에 password 자체를 넣지 않음 |

CodeQL의 [공식 규칙](https://codeql.github.com/codeql-query-help/rust/rust-cleartext-logging/)과 [민감 이름 heuristic](https://codeql.github.com/codeql-standard-libraries/rust/codeql/concepts/internal/SensitiveDataHeuristics.qll/module.SensitiveDataHeuristics%24HeuristicNames.html)을 대조하면 password 이름을 포함한 helper의 반환값에 광범위한 민감도 추적이 생긴 **오탐 가능성이 높다**. 다만 이는 SARIF·코드·공식 문서에 근거한 추론이며, 이름 변경 실험이나 새 CodeQL 분석으로 확정한 결과는 아니다. 암호 문서의 복호화 본문도 민감할 수 있으므로 이름만 바꿔 경고를 숨기는 방법을 해결책으로 확정하지 않는다.

다음 보완은 source 의미와 실제 logger 인자를 분리해 설명하고, 필요한 코드/모델 보정 후 새 head에서 CodeQL을 다시 확인하는 것이다. 실제 시험 경로는 근거를 붙여 별도 분류할 수 있지만 test 전체 제외·규칙 제외·언어 제외·69건 일괄 dismissal은 수행하지 않는다. 재분류가 필요한 경우 [GitHub 운영 지침 9.4](../../manual/github_operations.md)의 승인 절차를 따른다.

증적: `codeql-check.json`, `codeql-alerts.json`, `codeql-rust.sarif.json`, `alert-classification-summary.json`, `production-sink-review.txt`, `test-sink-review.txt`, `production-sink-origins.txt`, `exact-head-checks-final.jsonl`.

## 원 head 최초 조판 원칙 준수 판정

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 실제 신고의 저장소 구조와 기존 #5169 선택 계약; 최신 base FAIL / 후보 PASS. 문서 ID 예외 없음 |
| 측정·배치 공통 결과 | 비해당 | 줄 측정·배치 helper 변경 없음.실제 선택 본문 출력은 별도 미검증 |
| 최종 출력 소비 계약 | 미검증 | 선택된 문단 IR이 기존 renderer에 전달되지만 암호 ViewText 실제 한컴 출력 대조 없음 |
| 분할·이어받기 계약 | 비해당 | 컷·예약 높이·rowspan·clipping 계산을 변경하지 않음 |
| 줄 소속·점유 높이 | 비해당 | LineSeg 수용 조건·줄 구성·높이 계산을 변경하지 않음 |
| 사례·증거 독립성 | 충족 | 신고 구조 및 기존 공개 대조군과 합성 재현을 구분. 비공개 원본 검증으로 승격하지 않음 |
| 기준값 변경 | 비해당 | 기존 기대값·baseline·golden 수정 없음 |
| 렌더링 회귀 추가 선행 조건 | 비해당 | 새 검사는 section 선택·본문 보존·오류/상한 계약이며 배치 golden을 고정하지 않음 |
| 90% 후 확대 판독 | 미검증 | 실제 암호 ViewText 원본/독립 Print PDF·TSV 미제공, 점수·시각 통과 주장 없음 |
| 주장·검증 범위 | 미검증 | 파서 계약 6건과 대조군 9건 확인. 실제 WASM API·한컴 출력 일치는 미검증 |

## 접수 당시 후속 계획 (이후 보정은 아래 최신 결과 참고)

1. 기여자에게 결함 재현 및 CodeQL 69건의 공통 source와 sink 분류를 전달한다. 초안은 ignored `output/pr-review/pr7698-20261011/contributor-comment-ko.md`; 게시 승인 전에는 게시하지 않는다.
2. 개인정보를 제거한 정상 한컴 저장 암호/변경 추적 재현본을 요청한다. 실제 기업 원본 공개는 요구하지 않는다. 해당 본문 선택에 대응하는 독립 한컴 Print PDF, 최신 head Native/fresh WASM 출력·전쪽 TSV·대표 PNG로 실제 동작을 확인한다.
3. CodeQL 보완/정확한 분류와 새 head CI를 확인하고 필요한 로컬 lint·회귀·Docker WASM 검증을 완료한 뒤 최종 수용 여부를 판정한다. 현재 단계에서는 #7674를 닫지 않는다.

## 최신 통합 보정의 조판 원칙·검증 판정

| 항목 | 판정 | 근거와 한계 |
| --- | --- | --- |
| 입력 → IR → 공통 출력 | 충족 | DocInfo/RangeTag 및 header type/hide/범위/merged → parser visible IR → 공통 typeset/layout → Native/fresh WASM. 텍스트와 소유 개체를 함께 투영하며 paint에서만 숨기지 않음 |
| 저장 줄·중앙 정렬 | 충족 | 축 변경/문단 병합에서만 LineSeg 무효화. 유효한 저장 중간 줄 폭은 유지하고 재조판 줄의 비가시 말미 공백을 정렬에서 제외. #3837 겹침 4→3 복원·전수 래칫 통과 |
| 빈 줄의 쪽 소속 | 충족 | 저장 꼬리와 현재 흐름 일치 근거를 HWPX에도 적용. 문단 19·20의 두 빈 줄 및 제목은 같은 2쪽, 관계형 정식 검사와 한컴 Print 직접 대조. 빈 글자 없음으로 점유를 제거하지 않음 |
| 표 조각/rowspan 컷 | 비해당 | 표 컷·예약·clipping 구현은 바꾸지 않음. 변경된 visible IR에 소속된 표 4개와 앞뒤 내용은 정식 회귀 및 대표 PNG로 검증 |
| 수정 전 FAIL / 수정 후 PASS | 충족 | 원 parser 검사: 최신 base 3 FAIL → 원 기여/보정 PASS. HWPX 같은 최종 테스트: 이전 lib 3 FAIL → 보정 3 PASS. XML 공백 12쪽/164문단 및 빈 줄 p2 27.65746%의 독립 진단은 분리 기록 |
| 렌더링 회귀 선행 조건 | 충족 | 원본 HWP/HWPX 및 의미 동등 XML 공백 대조군의 Native/fresh WASM 전쪽 90% 이상·직접 판독 뒤 3건 추가 및 기존 검사 재검토. 절대 px/전체 SVG hash/baseline 완화로 실물 배치 고정하지 않음 |
| 미편집 HWP 저장 | 충족 | raw ViewText 보존·visible BodyText 생성; production 저장본 한컴 Print와 원본의 6쪽 raster 동일, Native/fresh WASM 재열기 내용 동등. 편집 전체 보존으로 확대하지 않음 |
| 최종 source 검증 | 충족 | fmt·세 Clippy·workspace·정책·집중 13건·전수 10,565 PASS. Native Skia lib rhwp 3,927 PASS/13 IGNORE 및 내부 crate 182 PASS, placeholder 2 PASS, 직접 PDF 4 PASS |
| 직접 시각 판정 | 충족 | 메인터너 HWP 해결 확인 및 HWPX Studio 시각 통과. 최종 18장 WASM PNG는 승인 당시와 byte-identical; 자형·미세 간격 잔차 별도 기록 |
| 비공개 기업 원본 / 전체 보기·편집 | 미검증 | 원본 미제공. 전체 모드 전환·서식 복원·수락/거부·HWPX 이력 저장 범위 미구현/미검증 |
| 원격 CI/CodeQL 및 merge | 충족 | head `34e1276015`의 필수 CI·CodeQL·CI 정책 성공, 최신 base merge-tree 성공 뒤 merge `4020c0428d`. Rust SARIF 0건이며 해당 쿼리 실행 확인 |


## 승인 후 원격 반영·병합 결과

사용자가 보정 검증 통과 뒤 다음 절차 진행을 승인했다. 원 fork `scari/rhwp`의 `fix/7674-encrypted-viewtext`에 원 기여 이력을 보존한 head `34e1276015eb892a199cfa576b1d90fdd194135b`를 fast-forward push하고 한국어 제목·본문을 갱신했다. 본문 일치·UTF-8과 정확한 head SHA의 대표 PNG 4개 raw URL을 확인했다. 신규 LFS pointer는 0개이며 fork lock 인증 실패에는 해당 push 한 번의 `lfs.locksverify=false`만 적용했다. hook을 우회하거나 영구 설정을 변경하지 않았다.

- [CI 38117884014](https://github.com/edwardkim/rhwp/actions/runs/38117884014) success. Lint·Native Skia·Archive A/B/C/D·필수 Build & Test가 성공했다. CI WASM Build는 skipped이며 실제 WASM 증거는 로컬 Docker 빌드·브라우저 검사다.
- [CodeQL 38117884066](https://github.com/edwardkim/rhwp/actions/runs/38117884066)와 별도 GHAS check success. CI merge SHA `96bc1e2471fe93b229af73e9c73c90dc832ef8cd`의 네 언어 SARIF 결과는 각각 0건, Rust 분석 ID `1930807311`이다. CodeQL 2.27.2 / rust-queries 0.1.44에서 `CleartextLogging.ql`의 로드·평가·해석을 실제 로그로 확인했다. 이름·반환 계약 보정의 새 분석 결과이며 문서 본문 일반의 비민감성을 주장하지 않는다.
- merge 직전 base `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47`를 fetch하고 head·필수 checks·MERGEABLE/CLEAN 및 merge-tree `e7f587ea2d9f75a7012575136ca65b162d906470`을 재확인했다. 총 37개 check가 종료됐고 failure/pending은 없었다.
- [PR #7698](https://github.com/edwardkim/rhwp/pull/7698)을 `--merge --admin --match-head-commit`으로 병합했다. merge `4020c0428defba3459c22879f38facd8019da2d7`, 시각 `2026-10-11T06:42:52Z`. 원 기여 `1489ca1d95e3d6e6b888870d3b14cf240866ef00`과 제출 head가 upstream/devel의 ancestor인 것을 확인했다.
- [#7674](https://github.com/edwardkim/rhwp/issues/7674)는 [devel push 종료 작업](https://github.com/edwardkim/rhwp/actions/runs/38118957938)으로 `2026-10-11T06:43:20Z`에 completed 종료됐다. #5169 등 참조 이슈를 함께 닫지 않았다.
- [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/38118957946)은 success, `refreshed_targets=46`으로 metrics branch에 반영됐다. 병합 후 검증 CI를 재실행하지 않았다.

후속 운영 기록은 archive review 두 파일과 오늘할일만 devel에 직접 반영한다. PR 전용 clean worktree `/tmp/rhwp-pr7698-before-hwpx-20261011`와 local review branch는 clean·ancestor·활성 프로세스 부재 확인 뒤 제거 완료했다. contributor fork branch·다른 검토 worktree·공용 `target/pr-review`는 보존한다. 기본 작업공간의 Studio 7700은 메인터너가 확인 중인 환경이므로 유지한다. 초기 진단 전용 ref는 merged source에 없는 로컬 기록을 포함하므로 유지한다. ignored 원문 검증 자료도 댓글용 상세 검증 근거 보존을 위해 유지한다. 이번 승인 범위의 push·본문·merge·이슈 종료 확인은 완료했으며 별도 PR/issue 댓글은 게시하지 않았다.
