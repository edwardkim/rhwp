---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7163_review.md
last_verified: 2026-09-15
---

# PR #7163 검토

## 판정과 적용 범위

판정은 **승인**이다. 메인터너 보정으로 **기존 P2 머지 보류 사유를 해소**했다. 실제 prefix의 배치 좌표를 suffix가
이어받게 바꿨고, 14쪽에서 빠졌던 prefix 첫 줄도 복구했다. 사용자가 발견한 Studio 본문 글꼴 깨짐은
별도 Canvas 글꼴 체인 병합 오류로 확인해 함께 보정했다. 검토 증적은 사진이 들어 있는 전체 원본으로 교체했다.

원 contributor head를 보존한 채 보정 두 커밋을 원 PR 브랜치에 직접 push했다.
코드 candidate는 `7bef9d6ecb6aca50bee1b57f8f8759a9fd73d7ec`다. 원 head의 CI와 구분해
이 candidate의 Full CI·CodeQL·Render Diff·Adapter·Proptest 성공을 확인했다.
그 뒤 review·오늘할일을 단일 문서 trailing commit으로 반영한다.
reviewer `jangster77` 할당은 완료했으며, 이 기록은 GitHub approve 제출이나 merge 완료를 뜻하지 않는다.

## 검토 대상과 출처

| 항목 | 검토값 |
| --- | --- |
| 원 PR / 작성자 | [#7163](https://github.com/edwardkim/rhwp/pull/7163), planet6897, 기존 기여자 |
| 제목 | 수정: 표 옆에서 이어지는 문단이 표 높이를 한 번 더 타지 않는다 (#7158) |
| 관련 이슈 | [#7158](https://github.com/edwardkim/rhwp/issues/7158), 원 PR의 `Fixes #7158` |
| 최초 contributor head | `4ca6cf993ccdd822522a335cbd03b6b28ef0ed31` |
| source base | `4fddb1bb7244fb478ea18f927aca273f93ac7322` |
| 검토 기준 upstream/devel | `bee316e246929b2587034dfda17ae2927dee8eb4` |
| branch | `codex/pr7163-review-20260915` |
| 초기 로컬 검토 체리픽 | `06c4a721a554218e791598d6385a8cd2ba5b50c7`, `-x` 출처 보존, 충돌 없음 |
| 체리픽 직후 tree | `03a587b50803764fda83e3cd537ef3b3c2591a43`, 현재 base와 원 PR의 merge tree와 동일 |
| 원 PR Studio 보정 | `459162ca92e201abb80e331baa7247a0d1356df9` |
| 원 PR 레이아웃·입력 보정 / code candidate | `7bef9d6ecb6aca50bee1b57f8f8759a9fd73d7ec` |
| 최초 contributor head CI | 31 성공(30 check run + CI Impact Policy status), 3 skip; 보정 candidate 검증과 구분 |
| 코드 검증 상태 | OPEN, non-draft, MERGEABLE; 아래 보정 candidate의 5개 workflow 성공 |
| fork 권한 | `maintainerCanModify=true`, repo API `permissions.push=false`; PR source에 실제 non-force push 성공, 원격/PR head 일치 확인 |

base route: `collaborator_external_pr.md`.
modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`.
loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 자식 문서와
`verification/visual_sweep_guide.md`. 현재 정본과 사용자의 CI 회귀 재사용 지시를 적용했다.

초기 검증은 최신 base 위 체리픽에 보정한 `92b14b689`·`3e6c1b36c4292d4c1809f0db02b68bfa46fc3e34`에서 수행했다.
직접 push 단계에서는 기여자 원 head 위로 메인터너 소유의 미게시 커밋만 재배치했다.
기여자 history는 재작성하지 않았으며 force push나 devel merge도 하지 않았다.
현재 base와 candidate의 자동 merge tree `d886a0f700b5a0eae92b63bdc35d9e71ef42c429`는
검증 당시 `3e6c1b36c`와 `src/`, `crates/`, `tests/`, `rhwp-studio/`, `Cargo.toml`, `Cargo.lock`의
차이가 없다. `.gitattributes`의 입력 보호 항목 위치만 충돌 없는 기존 section 경계로 옮겼다.

## 원인과 메인터너 보정

원 PR은 Square 표 뒤 `PartialParagraph`가 동일 표의 `WrapAroundPara` prefix와 연결되면
`col_area.y + stored_vpos`로 돌아갔다. 그러나 앞줄은 실제 표 앵커에 상대적으로 그려지므로
선행 여백 변화가 있을 때 두 조각의 기준이 달라졌다.

기존 최소본의 선행 제목 표 pi=75, ci=0에 `outer_margin_bottom += 1500 HWPUNIT`만 적용한
메모리 내 반례에서 첫 줄은 20px 내려가지만 suffix는 제자리에 남았다. 원 PR 적용 상태의 첫 줄 간격은
30.02667→10.02667px가 되어 높이 20px 글줄이 겹쳤다. 같은 반례의 수정 전 devel은 모든 줄이 함께
20px 이동했으므로, 이는 원 PR의 새 결함이었다. 문서 사본이나 조작한 한컴 기준 PDF를 만들지 않았다.

보정은 현재 Column에 이미 출력한 동일 section/paragraph의 직전 TextLine을 찾아,
**실제 y + 저장된 두 줄 사이의 vpos 차이**를 소비한다. 표 식별자와 `prefix.end_line == suffix.start_line`
소유 컷이 맞는 경우에만 동작한다. 직전 줄이 없거나 vpos가 유효하지 않으면 기존 흐름을 유지한다.
좌표를 문서별 상수로 맞추거나 별도 측정 추측을 추가하지 않았다.

14쪽 pi=156은 유한 prefix가 typeset에서 이 단에 배정됐는데, 표의 실제 아래 경계 밖이라는 이유로
paint에서 버려졌다. suffix가 line 1부터 시작해 line 0을 다시 그리지 않으므로 첫 문장이 누락됐다.
첫 fragment의 명시적 유한 prefix는 소유한 줄을 배치하도록 바꿨다. 전체 어울림 문단과 후속 fragment의
범위 필터는 유지했다. “이러한 문제를 해결하기 위해 강원도는”이 한컴 PDF와 같이 나타난다.

### 호출 경로와 경계

- `typeset.rs`의 Square wrap 분기는 저장 레인 prefix와 전폭 suffix를 나누고, formatted 줄 수·저장 seg 수·
  가용 높이를 확인해 prefix를 wrap 목록, suffix를 일반 PageItem에 배정한다. 이 컷과 측정은 변경하지 않았다.
- `layout_wrap_around_paras`가 소유 prefix를 배치한 실제 TextLine 결과를 `layout.rs`의 Square 후속 항목 처리에서
  소비한다. suffix와 다음 문단이 동일한 앵커 이동을 이어받는다. 기존 right-lane 전체 문단 처리와는 별도 분기다.
- 이번 변경은 같은 단에서의 prefix/suffix handoff다. rowspan 분할 컷·셀 높이 예약·각주 예산은 변경하지 않았다.
- 반례는 10px/20px 선행 여백 이동, 후속 문단 보존, 13개 handoff 문단의 정확한 줄 수·소유·간격으로 고정했다.
  기존 #6778의 오른쪽 레인/일반 들여쓰기/왼쪽 레인 경계도 focused test로 확인했다.

### Studio 글꼴 깨짐

실제 Chrome `localhost:7700`에서 본문 `fillText`에는 정상 한글이 전달됐지만,
font-chain은 `HY신명조, 휴먼명조, Batang, ...`였다. 표시 정책이 원 face를 제외하고 대체 체인을 만들었는데
`wasm-bridge.ts::substituteCssFontFamily`가 엔진의 원 face를 다시 앞에 삽입했다. 이 환경에서 legacy
휴먼명조가 선택되면 한글이 물음표 상자로 그려졌다. 원 face의 로컬 접근 record는 `null`이었다.

명시적 치환이 있고 표시 체인에서도 제외한 원 face는 다시 넣지 않게 했다. 엔진 전용 별칭,
치환 규칙이 없는 이름, 표시 정책이 승인한 원 face는 보존한다. 원 문서 글꼴명·문자 데이터는 바꾸지 않았다.
같은 사용자 문서를 실제 Chrome에서 다시 열어 본문 한글의 정상 표시를 직접 확인했다.

Visual Sweep의 WASM 모드는 **WASM SVG export**이며 Studio Canvas2D 실사용 검증을 대체하지 않는다.
SVG에는 legacy face의 별도 fallback이 있어 기존 sweep만으로 이 문제를 놓쳤다. 따라서 이번에는
SVG 비교와 실제 Studio 확인을 별도 증거로 기록한다.

## 입력 커밋 확인

**충족**. 전체 원본은 원 PR의 `7bef9d6ecb6aca50bee1b57f8f8759a9fd73d7ec`에 추가했고 기존 최소본·기준 PDF는 재사용했다.
검증한 파일과 커밋된 파일의 SHA-256이 일치한다.

| 입력 | SHA-256 | 역할 |
| --- | --- | --- |
| `tests/fixtures/issue_7158/156492236-regulatory-sandbox-full.hwpx` | `af198ad2dff0c1ad0458d7147ca8fcf42b45b0d2e5ceb2e10353debc8250a5b9` | 23,230,621 bytes, 사진 포함 원본 |
| `samples/issue4090/156492236_규제샌드박스_min.hwpx` | `d6f4d431b9a4d934b3b4e4330546ef61768c953c2e1328010d2f75440fefa070` | 기존 구조 회귀·메모리 반례 입력 |
| `pdf/156492236-regulatory-sandbox-full-2020.pdf` | `54139a4760f911f57861f33920f5755e07eee85c1cf2cfedfd6c272e25a5c3e9` | 기존 한컴 PDF, 926,826 bytes, 17쪽 |

최소본은 BinData/Preview가 1×1 스텁이므로 전체 PDF의 사진과 비교할 수 없었다. 전체 원본은
Mac의 기여자 자료에서 찾았고, Git 추적 HWP/HWPX에 동일 바이트 파일이 없는 것을 SHA-256으로 확인했다.
Contents/header.xml·section0.xml은 최소본과 동일하다. 전체 사진 비교에는 원본을 사용하며
같은 layout의 samples census를 중복시키지 않도록 tests/fixtures에 보존했다.
[입력 README](../../../tests/fixtures/issue_7158/README.md)와 [manifest](../../../tests/fixtures/issue_7158/MANIFEST.json)에 출처를 기록했다.

원본 저장 정보는 hancom-office-2018 10.0.0.11131, printMethod=4다. N-up 저장 설정을 별도로 기록하며
새 oracle page-count 행을 추가하지 않았다. 기준 PDF는 Creator `Hwp 2022 0.0.0.0`,
Producer `Hancom PDF 1.3.0.550`, 595×841pt, PDF 1.6이다. 다시 변환하거나 이름만 바꿔 추가하지 않았다.

## 검증 결과

- #7158 focused: 3/3 통과. 원본 ladder, 10px/20px 선행 여백과 후속 문단, 13개 문단의 줄 누락·중복·간격 확인.
- #6778 focused: 7/7 통과.
- 전체 원본 신규 입력 보안: 1/1 통과; 원본 1개에 hidden text·prompt injection·unicode deception 검사 적용.
- Studio 글꼴/실제 setter 계약: 20/20 통과. 제외한 face 재삽입·엔진 별칭 유실을 주입하면 각각 실패하는 음성 대조 포함.
- TypeScript `npx tsc --noEmit` 통과.
- `cargo fmt --all -- --check`, suite manifest `--prepare` 후 `--check` 통과.
- Native 및 WASM Clippy `-- -D warnings`, workspace build,
  workspace/all-targets Clippy `-- -D warnings` 통과.
- 새 Native CLI, 새 WASM package 빌드 완료. 전체 17쪽 Native/WASM render-tree JSON이 모두 일치.
- 전체 회귀·Native Skia는 사용자 지시에 따라 별도 반복하지 않았으며 원 PR CI와 보정 focused 검증을 구분했다.

최초 focused 실행은 파생 suite 배정 불일치로 0건/exit 4였고 성공으로 집계하지 않았다.
`--prepare`로 파생 harness를 재생성한 뒤 현재 suite에서 실제 3건을 실행했다.
처음 manifest check의 drift도 재생성 후 해결했으며 생성물은 커밋하지 않았다.
보안 하네스는 `samples/` 경로만 허용해 `tests/fixtures/` 직접 입력이 거부됐다. 동일 원본을 가리키는
임시 `samples/pr7163-full-security-review.hwpx` 심볼릭 링크로 실제 검사를 실행해 통과했고 링크를 제거했다.
검사기나 allowlist를 변경하지 않았으며 파일 사본도 추가하지 않았다.

### 원 PR에 직접 push한 code candidate CI

아래 실행은 모두 `7bef9d6ecb6aca50bee1b57f8f8759a9fd73d7ec`,
`planet6897/rhwp`, `fix/7158-square-lane-continuation`, `pull_request` event와 일치한다.
PR #7163의 현재 head check rollup에 연결된 실행임을 확인했다. Fork run API의 `pull_requests` 배열은
비어 있어 그 배열만으로 PR identity를 추정하지 않았다.

| Workflow | 결과 | 실행 증거 |
| --- | --- | --- |
| Full CI | 성공; 네 Rust archive 테스트·Native Skia·frontend package·lint·Build & Test 포함 | [34965979834](https://github.com/edwardkim/rhwp/actions/runs/34965979834) |
| CodeQL | JavaScript/TypeScript·Python·Rust 성공 | [34965979867](https://github.com/edwardkim/rhwp/actions/runs/34965979867) |
| Render Diff | Canvas visual diff 성공 | [34965979652](https://github.com/edwardkim/rhwp/actions/runs/34965979652) |
| Adapter inter-diff | 성공 | [34965979781](https://github.com/edwardkim/rhwp/actions/runs/34965979781) |
| Proptest roundtrip | 성공 | [34965979912](https://github.com/edwardkim/rhwp/actions/runs/34965979912) |

재사용할 집계 check는 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34965979834/job/104374306726)다.
최종 trailing commit은 이 녹색 candidate의 후속 문서 변경만 포함한다. 후속 head의 fast-pass·required check
성공 여부는 별도이며 위 표의 코드 CI 성공과 혼동하지 않는다.

## 전체 원본 Visual Sweep

Native는 1·5·9–15·17쪽 10개, 새 WASM은 1·9·14쪽 3개를 96dpi/기본 threshold 32로 비교했다.
각 페이지의 compare/overlay/review를 직접 읽었으며 자동 flagged는 각각 0/10, 0/3이었다.
17쪽 전체 원장도 `fidelity_compare.py --text-only --export-all-svg --layout-ledger ... 0 16`으로 확인했다.

before CLI는 `65df76e6195e0b87471d82359eb11c437d0bf1a0` 빌드
(`/private/tmp/rhwp-pr7141-7149-review-20260915/rhwp-after`)를 재사용했다.
해당 SHA와 검토 base 사이 `src`, `crates`, `Cargo.toml`, `Cargo.lock` 차이가 없음을 재확인했다.

전체 문서의 이미지 23개가 before/after에 유지된다. 영향 없는 8쪽의 render tree는 좌표·텍스트·구조가
같고, 나머지 9쪽에서는 배치가 개선됐다. 14쪽에서만 누락됐던 21문자와 TextLine/TextRun이 복구됐으며
다른 쪽의 텍스트는 동일하다. pageCount=17, offCanvas/textOverlap/overlap/emptyPage/storedLineEscape는 0이다.
기존 일반 overflow 신호 36건은 남아 있으며 0건이라고 주장하지 않는다.

| 실제 쪽 | pixel match | 내용 픽셀 중심 자동 일치율 보조값 |
| --- | ---: | ---: |
| 1 | 82.82903% | 34.06017% |
| 5 | 84.38545% | 40.72713% |
| 9 | 85.77185% | 35.84638% |
| 10 | 87.55403% | 44.07495% |
| 11 | 90.28926% | 43.54813% |
| 12 | 86.73466% | 41.60846% |
| 13 | 85.93895% | 41.20987% |
| 14 | 84.75173% | 39.85907% |
| 15 | 85.92953% | 34.62236% |
| 17 | 86.15170% | 43.28834% |

전체 원본 기준 9쪽은 before 79.06595%/21.74593%에서 85.77185%/35.84638%로,
14쪽은 76.19210%/24.60792%에서 84.75173%/39.85907%로 개선됐다.
값은 사람 판정 정확도가 아니다. 기존 글꼴 굵기·이미지 위치/크기의 세부 차이는 남아 있으며,
이번 보정은 문서 전체가 PDF와 픽셀 단위로 동일하다는 주장이 아니다.

- [전체 원본 before 9쪽](../assets/pr7163_before_p009.png)
- [전체 원본 WASM 9쪽](../assets/pr7163_wasm_p009.png)
- [전체 원본 WASM 14쪽, 첫 줄 복구](../assets/pr7163_wasm_p014.png)

기존 최소본 PNG는 위 전체 원본 증적으로 교체했다. 재현 scratch는
`/private/tmp/rhwp-pr7163-review-20260915/`이며 `full-before-sweep`, `full-native-sweep`,
`full-wasm-sweep`, `full-fidelity`, `maintainer-validate.log`, `maintainer-focused-retry.log`,
`studio-font-tests.log`를 사용했다. 중간 JSON·로그·전체 페이지 PNG를 저장소에 무차별 추가하지 않았다.

## 공통 조판 원칙 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거·일반성 | 충족 | 동일 소유 컷의 실제 prefix 배치 소비, 문서 ID/좌표 상수 없음 |
| 측정·배치 일관성 | 충족 | 기존 typeset 컷 재사용, paint에서 다른 앵커를 추측하지 않음 |
| 분할·이어받기 계약 | 충족 | 유한 prefix 소유 줄 보존, 직전 TextLine과 suffix 연결, 여백 반례 해소 |
| 줄 소속·점유 높이 | 충족 | 13개 문단 정확한 line index/줄 수/간격, 이동한 후속 문단 보존 |
| 사례·독립 근거 | 충족 | 기존 한컴 PDF 전체 원본 비교와 별도 메모리 계약 반례, #6778 반례 |
| 기준값 변경 | 충족 | 원 PR의 한 행 23→0 강화 유지; 추가 baseline/threshold 완화 없음 |
| 주장·검증 범위 | 충족 | 원 CI와 로컬 보정, SVG sweep와 Studio Canvas, 잔여 차이를 구분 |

## 이후 처리

원 PR source `planet6897:fix/7158-square-lane-continuation`에 코드 보정은 직접 반영했다.
code candidate의 위 5개 workflow 성공을 확인한 뒤 review·오늘할일·대표 PNG만 단일 trailing commit으로 추가한다.
최신 base와 trailing head의 merge simulation, 문서 링크와 기존 오늘할일 보존을 push 전에 검사한다.
최종 trailing head의 required checks와 fast-pass 판정은 원격 push 뒤 별도로 확인한다.
이번 요청 범위는 원 PR push와 기록 반영이며 merge·이슈 종료는 아직 수행하지 않았다.

검증 바이너리 SHA-256:

- Native CLI: `124b4123b2d1d6c2c76141bf5df73006601858c07098ef1395a339d8b692b43d`
- WASM JS: `0ef35380618d4088db768ad067c39fb89bdb9367402f61c2574e8fe4535a3899`
- WASM binary: `88fa194649ad4294749580411f61add066d4fc006c90f87f18d80b8efb2ce406`

재현 명령의 핵심 입력:

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs issue_7158_square_wrap_continuation -- \
  --cargo-profile release-test --target-dir target/pr7163-review-20260915
venv/bin/python scripts/visual_sweep.py \
  --file-target pr7163-full tests/fixtures/issue_7158/156492236-regulatory-sandbox-full.hwpx \
  pdf/156492236-regulatory-sandbox-full-2020.pdf \
  --rhwp-bin target/pr7163-review-20260915/release-test/rhwp \
  --pages 1,5,9-15,17 --dpi 96 \
  --out /private/tmp/rhwp-pr7163-review-20260915/full-native-sweep
# WASM: 새 --target web package를 --wasm-pkg로 지정하고 --pages 1,9,14 사용.
```
