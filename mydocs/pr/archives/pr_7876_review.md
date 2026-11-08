# PR #7876 리뷰 — WASM 라이브러리 시작·export 선택

## 최종 판정

**메인터너 보정 후 수용 가능 — #7875 해결 범위의 로컬 검증 완료.**
원 기여를 최신 devel에 통합한 [PR #7981](https://github.com/edwardkim/rhwp/pull/7981)의 정확한 code candidate CI도 성공했다.
원 contributor head를 직접 merge 대상으로 삼지 않는다. 기존 git/path 라이브러리 소비 호환 개선으로 한정하며 신규 배포 채널·공식 parser-only SDK·외부 Rust API 안정성 계약을 추가하지 않는다.

검증 코드 producer는 `366654f29959c62088bee7206d77b8889e3ebcd4`, CI candidate는 문서만 뒤따른 `69f1b8e7ce2427fd642bec1b7a07a074f752264e`다.
다음 조건은 문서 trailing head의 실제 required checks·fast-pass 판정·MERGEABLE/CLEAN 재확인이다. 사용자는 PR 생성부터 CI 모니터링·merge·후속 처리까지 승인했다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 원 PR·작성자·base | [#7876](https://github.com/edwardkim/rhwp/pull/7876), `9ood2s`, devel |
| 첫 기여 | 작성자의 전체 상태 PR 조회에서 rhwp PR이 #7876 한 건임을 확인 |
| source fork·branch | `9ood2s/rhwp`, `feat/wasm-api-feature` |
| 원 head·규모 | `b273e3fc44dd713f33d22846faa6fb1d98f01d3b`, 1 commit·2 files·+14/−3 |
| reviewer | 원 PR은 `jangster77` 요청. 통합 PR은 작성자 self-review로 별도 reviewer 요청 없음 |
| 원 PR 상태 | 접수 및 통합 준비 시 OPEN/non-draft, maintainerCanModify=true, 현재 Cargo feature와 충돌 |
| 기준 devel | `515bd72dcde0eecdeece333666b675354b7fa91d` |
| 통합 경로 | collaborator_external_pr 9.1.1 + 첫 기여자·로컬 검증·review-only fast-pass·post_merge |
| 관련 이슈 | [#7875](https://github.com/edwardkim/rhwp/issues/7875), 통합 PR 본문에 Closes 명시 |

## 프로젝트 적합성과 원 기여·메인터너 보정

체리픽 전에 프로젝트의 Rust rlib 재사용 경계와 [#5446 메인터너 결정](https://github.com/edwardkim/rhwp/issues/5446#issuecomment-5327810241)을 대조했다.
[1단계 적합성·수정 전 재현](../../working/archives/task_m100_7875_stage1.md)에 독립 근거와 실제 입력·소비 코드를 기록했다.

- 원 기여: 기본 `wasm-api` feature로 facade와 JS export/start 속성을 선택하고 CLI의 required-features를 선언했다. 원 author·공동 저자와 `-x` 출처를 보존했다.
- Cargo 충돌 보정: 현재 기본 `pdf-import`와 관련 의존성을 보존했다.
- 설명 보정: 일반 Rust main의 자동 시작과 여러 명시적 wasm-bindgen start의 연결을 구별했다. 명시적 start끼리는 함께 실행되며 순서는 보장되지 않는다.
- 사용법 보정: PDF 제외 CLI는 `--no-default-features --features wasm-api`, Studio는 `--no-default-features --features console_error_panic_hook,wasm-api`로 바인딩을 유지한다.
- 기본값을 끈 Rust facade 소비자는 `wasm-api` opt-in이 필요하며 Cargo feature는 합쳐진다. 이 의도적인 선택 계약을 API 안정성 보장으로 확대하지 않는다.

## 구현 주장과 동작 검증

실제 경로는 `Cargo feature → module·attribute gate → wasm-bindgen 생성/init → 소비 앱 main·JS export`다.
같은 소비 앱 소스에서 일반 main과 명시 app start를 feature로 선택했고, rhwp의 실제 resolve feature가 빈 배열임을 확인했다.

| 실제 관측 | 기준 devel | 보정 후보 |
| --- | --- | --- |
| 일반 Rust main 실행 | false | true |
| 명시 app start 실행 | true | true |
| HwpDocument export | 492 | 0 |
| 일반 main 소비 앱 WASM | 16,753,644 bytes | 2,580,226 bytes |
| HWP5/HWPX 파싱 | 각각 1 section·1 paragraph | 동일 |

Node와 실제 Chrome web-target에서 확인했다. frozen 수정 전 바이너리를 같은 검사기로 재실행했을 때 의도한 main/export 계약으로 FAIL, 수정 후에는 PASS였다.
최소 4조건 대조군으로 일반 main과 명시적 start의 적용 경계도 검증했다. 수동 init_panic_hook/version Rust API는 console-only WASM 소비 앱의 타입 검사로 확인했으며 panic 로그의 runtime 검증이라고 쓰지 않는다.
기여자의 사유 앱 49.0→38.4MB 수치나 Trunk 자체 실행을 검증했다고 주장하지 않는다.

## 입력과 기본 동작 보존

실행 파일과 Git blob의 바이트 일치를 확인한 기존 공개 입력만 사용했다. 두 blank 입력의 bytes·SHA-256은 [1단계 입력 표](../../working/archives/task_m100_7875_stage1.md#최신-rhwp를-링크한-실제-소비-앱)에 고정했다.
추가 입력 `samples/hwpx/form-002.hwpx`는 131,571 bytes, SHA-256 `5ab8f7c368e02538f75f1cd2bd82bbd8de2f925a54ba7b38ec9395b2cdb804d4`다.
기준/후보의 실제 Chrome 기본 모드에서 3문서 전12쪽의 초기화·version·쪽수·페이지별 렌더 트리·SVG가 동일했다.

기본 JS·두 타입 선언·package.json·README·LICENSE 6파일, WASM import·522개 export도 동일했다.
기준 WASM 22,305,631 bytes와 후보 22,305,358 bytes의 차이는 후보 Cargo.toml을 유지하고 lib.rs만 gate 없는 기준 코드로 바꾼 대조군에서도 재현됐다.
대조군과 후보의 패키지 7파일이 WASM까지 바이트 동일했다. 대조 빌드 뒤 원본 lib.rs의 바이트와 Git clean을 복원·확인했다.

PDF 기본·제외 브라우저 계약은 기존 `scripts/tests/pdf-import-wasm.test.mjs`의 합성 PDF 생성기로 검증했다.
기본은 전체/선택 페이지와 대응 정보를 유지했고, 제외 모드는 PDF_IMPORT_DISABLED를 반환하면서 HWP/HWPX를 유지했다. 한컴 Print/시각 정확성의 증거로 해석하지 않는다.

## 완료한 로컬·CI 검증

[2단계 명령·전체 결과·도구·한계](../../working/archives/task_m100_7875_stage2.md)에 source SHA와 재현 명령을 기록했다.
공용 target/pr-review·jobs 8에서 Cargo를 순차 실행했다. 새 fixture·golden·기대값·CI 개별 호출은 추가하지 않았다.

| 로컬 검사 | 결과 |
| --- | --- |
| fmt·manifest·기본 Native/WASM lint·workspace build/all-target lint | PASS |
| 비기본 library·명시 API·console-only·Node/Chrome·fresh WASM 조합 | PASS |
| 전체 nextest | 10,808 PASS / 0 FAIL / 45 skipped |
| Native Skia lib | 4,043 PASS / 0 FAIL / 12 ignored |
| Skia placeholder·직접 PDF export | 각각 2 PASS / 14 PASS |
| PDF 제외 CLI | 4 PASS |

정확한 `69f1b8e7c`의 아래 workflow 6개와 check 36개를 모두 확인했다. Full CI의 lint·Native Skia·frontend package 및 archive A/B/C/D builder와 worker가 실제 성공했다.
별도 WASM Build·Frontend unit job의 skip과 frontend package의 fresh WASM/브라우저 gate 성공을 구분한다.

| workflow | 실제 결과 |
| --- | --- |
| [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/39030725781) | success |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39030725798) | success, 언어별 분석·GHAS check 확인 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39030725943) | success |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39030725786) | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39030725838) | success |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39030725830) | success |

## 조판 원칙·시각 범위와 한계

조판·좌표/높이·측정/배치 공유·분할/이어받기·저장 조판 정보 변경은 **비해당**이다.
Cargo feature와 lib gate만 바뀌며 renderer/parser/model/paint/font/resource 본문·선택 조건을 바꾸지 않는다.
기본 선택 소스의 동일성·실제 feature-only 대조 빌드·기본 브라우저 12쪽 동등성·기존 회귀로 무영향을 확인했다.
독립 Print와의 Visual Sweep·TSV·90/98% px 회귀 추가는 비해당이다. 쪽수나 SVG 해시를 한컴 시각 일치의 증거로 승격하지 않았다.
Docker 없는 Linux 호스트의 --no-opt 진단 빌드였으며 최적화된 Docker 배포·Trunk 본체·Windows 직접 실행은 검증 범위에 넣지 않는다.

## Merge 후 contributor PR comment 계획

원 PR에는 merge 전 검토 의견과 merge 후 결과를 한국어 존댓말로 남긴다. 첫 rhwp 기여에 감사·환영을 명시하고 원 구현과 PDF 보존·설명·사용법 보정의 이유를 구별한다.
실제 통합 PR·merge SHA·위 code CI와 trailing CI 결과·검증 수치·한계를 안내하고 이 리뷰/구현 기록은 merge SHA에 고정한 링크로 게시한다.
본문은 UTF-8 LF 파일을 --body-file로 전달하고 API로 한글·BOM·줄바꿈과 게시 결과를 확인한다.
#7875 종료 상태·후속 comment를 확인한 뒤 통합 완료한 원 PR을 close한다. contributor fork는 유지한다.

최종 증적은 이 archive와 stage 기록·기존 Git 입력/생성기·GitHub 실행 로그에 보존한다.
merge 뒤 duration 갱신 결과를 확인하고 devel sync·이번 임시 branch·사용 종료한 root output 전체를 정리한다. 공유 target/pr-review는 유지한다.
