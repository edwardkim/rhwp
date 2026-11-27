# PR #9445 리뷰 — HWP 표가 든 문단의 가져오기

## 최종 판정

**승인.** 최신 `devel`의 CLI에서 재현한 #9225 오류가 제출 커밋에서 해소됐다. 공개 API와 CLI의 집중 검사 39건, 필수 CI와 최신 base 병합 시뮬레이션을 통과했다. [승인 리뷰](https://github.com/edwardkim/rhwp/pull/9445#pullrequestreview-6085909514)를 게시한 뒤 원 PR을 일반 병합했다. [병합 커밋](https://github.com/edwardkim/rhwp/commit/29a9d9d11e2fd67609e0c387dd0f76458ef4d960)

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 원 PR·작성자 | [#9445](https://github.com/edwardkim/rhwp/pull/9445) · `donggyun112` |
| 관련 이슈 | [#9225](https://github.com/edwardkim/rhwp/issues/9225) · PR 본문의 종료 키워드 `Closes #9225` |
| 원격 브랜치·제출 커밋 | `donggyun112/rhwp:task/9225-table-common-tail-import` · `c5dbd34cce6d0029f56582b7880e721c92b4195d` |
| 지정 리뷰어 | `edwardkim` · 2026-11-27 17:56 KST에 요청 API로 확인 |
| 검토 기준 | `upstream/devel` `f941c1df6743d7ca94c980fa9d0e9c791fdd5bae` · 병합 시뮬레이션 성공, 결과 트리 `9b20d442d5726aae64b4dbc5d1e3d7b989410460` |
| 변경 범위 | Rust 가져오기 검증 1개 파일, 기존 통합 회귀 1개 파일 |

작성자 식별자를 정정하기 전 커밋 `8e7d06944aa69adc876cefd1374c53ebfefa40a9`와 제출 커밋의 부모와 파일 트리가 같다. 작성자가 기록한 macOS 14,213건은 동일 파일 트리의 결과다. 아래 메인테이너 실행은 제출 커밋에서 별도로 수행했다.

## 변경과 검토 범위

[#3587 문단 가져오기](https://github.com/edwardkim/rhwp/issues/3587)는 [#7101](https://github.com/edwardkim/rhwp/pull/7101)로 반영됐다. [당시 병합 커밋](https://github.com/edwardkim/rhwp/commit/ad6174255e6aabfaa48131f3f5f2f6287a93e48a). 원본을 다시 읽고 문단·표 소유와 리소스를 검사한 뒤 복사하는 계약을 [당시 검토](pr_7101_review.md)와 대조했다. #9225 제보에서는 설명문 길이가 0인 48바이트 `CTRL_HEADER`의 끝 두 바이트가 `00 00`이었다. 공통 속성 본문은 control ID 4바이트를 제외한 44바이트다.

[HWP5 표 69](../../tech/한글문서파일형식_5.0_revision1.3.md#표-69-개체-공통-속성)는 설명문 길이와 해당 길이만큼의 UTF-16 문자를 정의한다. [참고 커밋](https://github.com/edwardkim/rhwp/commit/0368a59724af704641c0231f51e280f62313b9f1)은 0으로 채운 꼬리를 공통 속성 전체에서 허용하는 접근이었다. 제출 커밋은 표 분기 안에서 원시 헤더 끝의 길이와 값, 빈 설명문, 파서가 별도로 보존한 두 바이트가 모두 일치하는 경계를 사용한다. 가져오기 검증에서만 설명 길이까지 검사하며 저장기는 기존 원시 헤더 보존 경로를 소비한다.

조판 원칙 검토는 비해당이다. 가져오기 허용 조건이 바뀌며 표의 너비나 위치, 줄 구성과 그리기에는 기존 모델과 호출 경로를 사용한다. 신규 회귀의 기대값은 문단·표 소속, 행·열과 셀 내용, 공개 API와 CLI 실행 결과, HWP 꼬리 보존이다.

## 검증 입력과 결과

검증 입력 커밋 확인은 충족이다. 실제 사용한 다음 파일의 SHA-256을 제출 커밋의 내용을 읽어 각각 대조했다. 제보된 비공개 원본 형태는 첫 파일을 읽어 메모리에서 구성한 뒤 HWP로 저장하고 재파싱했다. CLI 검사는 같은 바이트를 임시 파일로 전달했다. 검증 근거는 저장값과 공개 API·CLI의 실행 결과다.

| 기존 입력 | SHA-256 |
| --- | --- |
| [표 HWP](../../../samples/hwp_table_test.hwp) | `dc3e57d7577447ae4b47f59bdb0f499a5ac29066b62af8a650bcdd8d70eee32a` |
| [글상자 속 표 HWP](../../../samples/table-in-tbox.hwp) | `02b0ced8b96cec5b6e1cad6dd1926b701a2875ea9591c589e57edf7653daf4af` |
| [연구노트 원본](../../../samples/rnote/labnote-001.hwp) | `8401e778edc386f0a87a0df4120a2cf0733aee36c84070ddc864499bde73a1af` |
| [한컴 복사본 1](../../../samples/rnote/labnote-001-cp-01.hwp) | `83f3f604192cca91bf15475f0e1de98219b6e65edd3229260fb8e83ebb0bbe79` |
| [한컴 복사본 2](../../../samples/rnote/labnote-001-cp-02.hwp) | `e2f939584a357f1776fa57a1bc16e696094b236154a419ac687fee70655c8e2d` |

### 메인테이너 실행

WSL/Linux x86_64, Rust 1.93.1에서 `release-test` 프로필을 사용했다. 기본 작업공간의 최신 `devel`은 `cargo build --locked --offline --profile release-test --bin rhwp`로 빌드했다. 원 head에서는 `cargo test --locked --offline --profile release-test --no-run`으로 통합 대상 `regression_suite_011`, `025`, `027`, `002`, `022`를 만들었다. `bwrap`에서 시스템·사용자 글꼴 경로와 네트워크를 격리했다. Fontconfig 목록에서 `ttfs/opensource`의 Noto Sans KR 2종, Noto Sans Symbols 2, RHWP 옛한글과 RHWP 공개 바탕 5개만 확인했다.

| 검사 | 결과 |
| --- | --- |
| 동일 HWP 입력으로 최신 `devel` CLI 대조 | `run --dry-run`과 `run`이 각각 exit 2, `uninterpreted common tail` 반환 |
| 제출 커밋 CLI | 두 명령 모두 exit 0. 미리보기와 적용 결과가 같고 입력·원본은 보존됐다. 미리보기 동안 출력 파일은 생성되지 않았다. |
| 공개 API와 반복 저장 | 미리보기·가져오기 범위가 같았다. HWP·HWPX 각 두 번 저장과 재열기에서 표 소속, 행·열, 셀 내용을 보존했다. HWP는 `00 00`도 두 번 모두 보존했다. |
| 정상 입력과 경계 | 꼬리가 없는 기존 표는 통과했다. 0이 아닌 꼬리, 한 바이트 및 네 바이트 꼬리는 대상 변경 전에 거부했다. |
| 집중 회귀 | 가져오기 계획 8건, 블록 가져오기 14건, 사전 검사 9건, 중첩 표 5건, 반복 표 3건이 통과했다. 기존 진단 전용 3건은 ignore 상태였다. |
| 형식·테스트 편성·공백 | `cargo fmt --all -- --check`, 테스트 매니페스트 검사, `git diff --check` 통과 |

실행 파일 SHA-256은 기준 `7a4d55131b0acde7e37284bce7a13f54bbfa1343ea5ad64da75f82637aa20a99`, 제출 커밋 `6ecfd67043194ff09acfcebafe4eaca9894e72fc8403c918eb12f02da8d4aab2`다.

### GitHub Actions

제출 커밋의 GitHub Actions에서 Rust Lint와 회귀 Archive A–E, Build & Test, Rust CodeQL, Adapter inter-diff, Proptest 및 최종 CI Impact Policy가 성공했다. CI preflight 로그의 `fast_pass=false`를 확인했다. 입력 검증 변경의 CI 분류에 따라 Native Skia, Frontend 및 Render Diff의 렌더 작업은 실행 범위에서 제외됐다. 코드 스캐닝 보조 상태는 변경에 해당하지 않는 세 언어 구성의 생략을 표시해 중립이었다. 소스·검사·fixture를 추가 보정하지 않았고 최신 base 병합 시뮬레이션도 통과했으므로 전체 Rust 회귀는 이 실행을 재사용했다. [전체 CI](https://github.com/edwardkim/rhwp/actions/runs/45954486635) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45954486759) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/45954486712) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/45954486702) · [최종 정책 검사](https://github.com/edwardkim/rhwp/actions/runs/45957857636)

## Merge 후 contributor PR comment 계획

원 PR의 병합 커밋과 재현 결과를 남긴다. #9225의 자동 종료를 확인하고, 해당 표의 가져오기와 반복 저장 검증을 설명한다. 원 기여자에게 제보된 바이트 형태로 변경을 좁혀 검증한 점에 감사를 전한다.

## 병합 결과

병합 트리가 사전 시뮬레이션과 일치했다. nextest 실행 시간 갱신 작업이 성공했다. #9225는 GitHub에서 `completed`로 닫힌 것을 확인했다. [병합 후 실행](https://github.com/edwardkim/rhwp/actions/runs/45972403830)
