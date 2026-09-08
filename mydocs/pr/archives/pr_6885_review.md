# PR #6885 검토

## 대상과 판정

- 현재 판정: **머지 보류 — 원 코드 P2는 보정·로컬 검증 완료, 새 PR head CI 대기**
- PR: [#6885](https://github.com/edwardkim/rhwp/pull/6885)
- 이슈: [#6884](https://github.com/edwardkim/rhwp/issues/6884)
- 작성자: `zunstudio` (저장소 PR 이력 조회상 첫 외부 PR)
- 원 PR head: `ce6bca037b8e971a6818fef01a30acf495c28ff8`
- 동기화한 `upstream/devel` 및 로컬 `devel`: `5111c24c745863fde1f6a7db4b61ed48d2269195`
- 최초 통합 검토 이력 보존 브랜치: `codex/review-pr6885-integration-20260908`
- 최신 devel 위 cherry-pick 결과: `d73aa5d42` (원 commit 1개, 충돌 없음)
- current-base merge tree: `4e29614bc7ac270d427041660afce52fdae42051`; 검토 HEAD tree와 일치
- 규모: 3개 파일, +154/-75
- 조회 시점 상태: OPEN, non-draft, base `devel`, MERGEABLE/CLEAN
- 최초 검토에서는 원격 조치를 수행하지 않았다. 후속 사용자 승인에 따라 원 PR에 code·review를
  push하며, GitHub 댓글은 초안 승인 전 게시하지 않는다. merge는 이번 요청 범위 밖이다.

라우팅은 `collaborator_external_pr`이며 `intake_and_review`, `local_validation`,
`first_time_contributor`를 함께 읽었다. 아래 발견·전후 비교는 최초 검토 기록이며,
후속 직접 보정과 검증은 문서 마지막 절에 구분한다.

## 변경 이해와 댓글 검토

기존 lenient CFB reader는 마지막 이름만 비교해 `/BodyText/Section0` 요청에도 앞에 있는
`/ViewText/Section0`을 반환했다. PR은 directory ID를 보존한 트리 탐색을 사용하고, 트리 탐색에
실패하면 같은 이름이 유일한 경우만 복구한다. BodyText와 BinData 호출부에도 명시적 경로를 전달한다.
스트림 크기 제한과 복호화 알고리즘은 유지하며 #5169의 정상 ViewText 우선 규칙도 바꾸지 않는다.

[작성자 댓글](https://github.com/edwardkim/rhwp/pull/6885#issuecomment-5582151638)은 Node 설치 후
manifest 준비와 해당 suite 169건 통과를 추가 보고했다. 이 댓글을 참조했으며 작성자의 169건과
아래 로컬 검증 수를 혼동하지 않았다. inline review comment와 review event는 조회 당시 없었다.

댓글의 CRLF 문제는 이 PR의 코드 결함과 별개다. 현재 checkout은 `core.autocrlf=false`,
`Cargo.toml` LF이며 manifest 준비·검사가 통과했다. `.gitattributes`에서 `Cargo.toml`의 eol은
명시되지 않았다. 작성자의 CRLF checkout 재현은 별도로 수행하지 않았다.

## 발견 사항

### [P2] 트리에서 찾은 삭제 슬롯을 검증하지 않아 기존 복구가 실패한다

위치: `src/parser/cfb_reader.rs:822-823`의 즉시 반환.

`find_child_entry_by_name`은 이름만 일치하면 `obj_type=0`인 미사용/삭제 슬롯도 반환한다.
새 `find_entry_id`는 그 결과를 유효성 검사 없이 반환하므로, 손상된 child/sibling 링크가
이름이 남은 삭제 슬롯을 가리키면 아래의 유일 이름 fallback에 도달하지 않는다.

재현 CFB에는 다음 조건을 만들었다.

1. `/FileHeader`에 12바이트 `valid header`를 기록한 CFB v3를 만든다.
2. 미사용 directory slot에 해당 entry를 복사하고 `obj_type`만 0으로 설정한다.
3. root child pointer를 이 미사용 슬롯으로 바꾼다. 기존 유효 스트림과 데이터는 유지한다.

동일한 바이트에 최신 devel의 원본 reader와 PR reader를 각각 실행한 결과:

| 구현 | 유효한 FileHeader entry 수 | 읽기 결과 | 검증 exit |
| --- | ---: | --- | ---: |
| devel의 기존 reader | 1 | `Ok("valid header")` | 0 |
| PR reader | 1 | `StreamError("FileHeader: 스트림이 아님 (type=0)")` | 101 |

이는 reader 경계의 합성 CFB 재현이며 실제 사용자 HWP에서 발생했다고 주장하지 않는다. 다만
손상된 디렉터리를 복구하는 lenient 경로에서 기존에 복구하던 유효 스트림을 새 코드가 거절하는
회귀다. 실제 문서 파서도 `parse_hwp_with_lenient` 시작 시 `read_file_header()` 실패를 반환한다.

보완 방향은 트리 탐색에서 부모가 storage이고 최종 entry가 유효한 종류인지 확인하고, 무효 항목을
발견하면 안전한 유일 이름 fallback으로 진행하는 것이다. 미사용 슬롯, 끊긴 링크, 중복 이름의
모호성 거절을 함께 회귀로 고정해야 한다. 현재 PR의 정상 실물 fixture 테스트 2건은 이 경계를
포함하지 않는다.

### 재현 핵심 코드

아래는 진단 executable이 구성한 입력과 같은 내용이다. 원본 문서는 수정하지 않았다.

```rust
use std::io::{Cursor, Write};
use rhwp::parser::cfb_reader::LenientCfbReader;

let mut c = cfb::CompoundFile::create_with_version(
    cfb::Version::V3, Cursor::new(Vec::new()),
).unwrap();
c.create_stream("/FileHeader").unwrap().write_all(b"valid header").unwrap();
let mut bytes = c.into_inner().into_inner();
let sector = 1usize << u16::from_le_bytes(bytes[30..32].try_into().unwrap());
let sid = u32::from_le_bytes(bytes[48..52].try_into().unwrap()) as usize;
let dir = 512 + sid * sector;
let valid_id = u32::from_le_bytes(bytes[dir + 76..dir + 80].try_into().unwrap()) as usize;
let invalid_id = if valid_id == 2 { 3 } else { 2 };
let entry = bytes[dir + valid_id * 128..dir + (valid_id + 1) * 128].to_vec();
bytes[dir + invalid_id * 128..dir + (invalid_id + 1) * 128].copy_from_slice(&entry);
bytes[dir + invalid_id * 128 + 66] = 0;
bytes[dir + 76..dir + 80].copy_from_slice(&(invalid_id as u32).to_le_bytes());
let r = LenientCfbReader::open(&bytes).unwrap();
assert_eq!(r.read_file_header().unwrap(), b"valid header");
```

진단 소스와 실행 파일은 ignored `target/pr-review/pr6885-probe*`에 보존했다. standalone rustc가
실제 `cfb_reader.rs`를 모듈로 포함했고, 비교본은 `git show upstream/devel:src/parser/cfb_reader.rs`를
그대로 사용했다. 동일한 `cfb`/`flate2` rlib을 링크해 알고리즘 재작성 없이 전후 비교했다.

## 검증 결과

환경: Windows native PowerShell, Rust 1.93.1, Node 24.19.0, cargo-nextest 0.9.143,
검토 target `target/pr-review`. 기존 target과 다른 작업의 산출물은 삭제하지 않았다.

- `node scripts/rust-test-suite-manifest.mjs --prepare`: 통과.
- `cargo fmt --all -- --check`: exit 0.
- `node scripts/rust-test-suite-manifest.mjs --check`: 통과. 1,208 sources,
  28 suites + 20 exceptions = 48 integration targets.
- `node scripts/run-rust-test.mjs lenient_cfb_path_lookup -- --cargo-profile release-test --target-dir target/pr-review`:
  새 회귀 **2/2 통과**. 첫 compile 7분, test 0.061초.
- `node scripts/run-rust-test.mjs issue_5169_prefer_viewtext -- --cargo-profile release-test --target-dir target/pr-review`:
  기존 ViewText 우선 회귀 **1/1 통과**.
- current-base merge simulation 및 `git diff --check upstream/devel...HEAD`: 통과.
- 추가 합성 CFB 전후 비교: 위 P2 회귀를 재현했다.

원 code head `ce6bca0`의 [Full CI run 34204676159](https://github.com/edwardkim/rhwp/actions/runs/34204676159)는
success이며 lint, archive A-D test, Build & Test가 성공했다. CodeQL 및 proptest도 조회 시점에
성공했다. 제품 보정 없이 clean한 current-base merge와 focused 검증을 수행했으므로 전체 회귀와
Clippy 묶음은 기존 code head의 CI 증적을 참조했다. 해당 GitHub CI를 로컬 통합 head에서 실행한
결과로 표시하지 않는다.

이번 PR은 CFB 경로 조회 변경이고 renderer/layout/pagination 및 신규 sample을 바꾸지 않는다.
새 테스트는 원시 스트림 선택을 검사하며 한컴 PDF나 특정 시각 개선을 주장하지 않으므로 PDF
변환·visual sweep은 수행하지 않았다. 위 reader 회귀만으로 보류 사유가 성립한다.

## 최초 보류 해제 조건

무효 directory entry를 즉시 채택하지 않도록 보완하고, 위 실패 입력 및 유일/중복 fallback 테스트와
기존 경로·ViewText 회귀를 통과해야 한다. 수정된 code head의 CI도 다시 확인해야 한다. 현재 GitHub
CI가 녹색이라는 사실만으로 이 회귀를 승인하지 않는다.

## 2026-09-08 직접 보정 및 push 준비

사용자가 보류 조건의 코드 보정과 **원 PR source branch 직접 push**를 승인했다. 이에 따라
최초 devel 기반 통합 이력은 위 보존 브랜치에 남기고, 가시성 브랜치
`codex/review-pr6885-20260908`을 원 head `ce6bca037b8e971a6818fef01a30acf495c28ff8`
위에서 시작했다. contributor history를 rewrite하지 않고 별도 보정 commit을 추가했다.

- 보정 code SHA: `4b1f1179bd4c42e0a87e88f9accb063a34943a88`.
- 원 contributor 변경: 경로 기반 CFB 조회와 실물 fixture 회귀 2개.
- collaborator 보정: 경로 부모는 storage(type 1)만 허용하며, 최종 무효 entry(type 0/255 등)는
  즉시 반환하지 않고 기존의 유일 이름 fallback으로 진행한다.
- 합성 CFB 회귀 4개 추가: 무효 이름 슬롯 복구, 끊긴 링크의 유일 이름 복구/중복 이름 거절,
  stream의 부모 storage 오인 거절, 순환 sibling 링크의 종료와 복구.
- 변경 sample·fixture·baseline 없음. 파생 harness와 manifest는 stage하지 않는다.
- push 직전 조회에서 PR head와 contributor remote SHA가 모두 원 head와 같고
  `maintainerCanModify=true`임을 확인했다.
- 재조회한 `upstream/devel`: `e7e9785893e7dffdaa600859cb1f28cca18621cb`.
  `git merge-tree --write-tree upstream/devel HEAD`는 충돌 없이
  `4673109d63abe5caf0964cb453141b99e18a38b2`를 생성했다. 로컬 검증은 이 merge tree가
  아닌 원 source 위 보정 code SHA에서 수행했으며, 새 current-base 검증은 push 후 CI로 확인한다.

### 보정 코드 로컬 검증

모든 명령은 Windows native PowerShell에서 `target/pr-review`를 사용해 순차 실행했다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| native root Clippy (`-D warnings`) | exit 0 |
| WASM32 lib Clippy (`-D warnings`) | exit 0 |
| `cargo build --locked --workspace --target-dir target/pr-review` | exit 0 |
| workspace all-target Clippy (`-D warnings`) | exit 0 |
| manifest `--prepare`, `--check` | 통과; 1,197 sources, 48/48 integration targets |
| `lenient_cfb_path_lookup` focused nextest | **6/6 통과**, exit 0 |
| `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast` | **9,222/9,222 통과**, 46 skipped, exit 0 |
| `git diff --check` | 통과 |

전체 nextest 실행 시간은 665.848초(별도 compile 8분 30초), run ID는
`63aded46-580e-4d7e-ada5-0c294bffad59`다. 원 PR의 이전 녹색 CI를 이번 로컬 검증의 대체로
사용하지 않았다. focused 첫 시도는 fmt 이후 source 크기 변화로 harness 배정이 달라져 0개 실행으로
실패했으며 성공으로 집계하지 않았다. fmt 후 `--prepare`를 다시 실행해 6개 모두 실행·통과했다.

### 작성자 댓글의 CRLF 지적 처리

댓글 원문을 재조회했다. 생성기의 `renderCargoTestBlock`은 LF로 조립한 문자열을 반환하고,
검증 코드는 Cargo marker 블록과 이를 개행 정규화 없이 직접 비교한다. 메모리에서만 기존
`Cargo.toml`을 CRLF로 변환한 비교 결과는 다음과 같다. 실제 checkout이나 Cargo 파일은 수정하지 않았다.

- 기존 LF 내용에 expected block 포함: `true`.
- CRLF 변환 내용에 expected block 포함: `false`.
- CRLF를 LF로 정규화한 내용에 expected block 포함: `true`.

따라서 줄바꿈만으로 drift 판정이 달라질 수 있다는 지적에는 코드·진단 근거가 있다. 다만
작성자의 clean CRLF checkout 전체 실행과 `49 > 48` 연쇄 오류는 독립 재현하지 않았다.
`Cargo.toml text eol=lf` 지정, LF/CRLF 회귀와 필요 시 비교 정규화는 parser 수정과 분리한
후속 검토를 제안한다. 이번 PR에 `.gitattributes`나 생성기 변경을 섞지 않는다.
답변 초안은 사용자 승인 전이며 GitHub에 게시하지 않는다. 새 head CI 확인 후 최종 판정을 갱신한다.
