# Task M100 #6381 최종 보고 — `test-caption` false-pass 제거

- **이슈**: [#6381](https://github.com/edwardkim/rhwp/issues/6381)
- **브랜치**: `task_m100_6381-test-caption-false-pass`
- **보고일**: 2026-08-30 KST
- **착수 기준**: `upstream/devel@2bcf9b261c3b761d114bc2b3a35ed85ccd1e461e`
- **최신 기준**: `upstream/devel@2deb3dd6163d83d2932ab58ac5a0bf61bfce6d31`
- **검증 기준 HEAD**: `d8ab820b065618966dfb67969cf2c1b1ba26992a`
- **결과 상태**: PR #6391 review 보정·최신 기준 전체 로컬 검증 완료

## 1. 해결 결과

내부 진단 명령 `test-caption`이 고정 fixture 좌표의 캡션 변경을 하나도 검증하지 못해도 원본 SVG와
`완료`를 남기고 exit 0으로 종료하던 false-pass를 제거했다.

- 네 caption mutation의 성공 여부를 개별 추적하고 실패 원인을 stderr에 기록한다.
- mutation에 성공한 대상도 Picture 종류, caption 존재, 방향·세로 정렬·폭 8504·간격 850을 정확히
  확인한다.
- mutation 또는 verification이 하나라도 실패하면 출력 폴더 생성과 SVG 렌더 전에 exit 1로 종료한다.
- 0-page 입력도 성공으로 오인하지 않고 exit 1로 종료한다.
- 네 대상이 모두 통과한 경우에만 기존 stdout, SVG 파일명과 `완료`를 유지한다.
- verification은 setter와 같은 범위로 Endnote 가상 문단과 `Shape(Picture)` 표현을 해석한다.

## 2. 회귀 계약

subprocess 통합 test가 실제 HWP parse→mutation→render 경계를 통과하도록 세 입력을 고정했다.

| 시나리오 | 기대 계약 | 결과 |
| --- | --- | --- |
| 고정 대상이 없는 임의 실문서 | exit 1, stderr 진단, `완료`·SVG 없음, panic 없음 | 통과 |
| 일부 대상만 있는 합성 HWP | exit 1, 일부 성공 뒤에도 `완료`·SVG 없음 | 통과 |
| 네 대상이 모두 있는 합성 HWP | exit 0, 기존 성공 stdout, SVG 1개 이상 | 통과 |
| 성공 경로 verification 증적 | `caption=Some(...)` 정확히 4건 | 통과 |
| `Shape(Picture)` setter/getter | 방향·세로 정렬·폭·간격 보존 | 통과 |

합성 fixture는 공개 `HwpDocument` API와 `assets/logo/logo-16.png`를 사용하며 별도 binary fixture를
repository에 추가하지 않는다.

## 3. 문서와 범위

`mydocs/manual/cli_commands.md`에 `test-caption`이 일반 문서 변환 명령이 아니라 고정 fixture의 네 그림을
검증하는 내부 명령임을 명시하고, runtime/validation failure가 exit 1이라는 계약을 반영했다.

다음은 의도적으로 변경하지 않았다.

- 고정 좌표를 자동 그림 탐색으로 일반화하는 대상 선택 정책
- caption setter와 document model
- renderer·layout·pagination·SVG 의미
- Render Diff workflow와 #3789의 move-only 경계
- 공개 command·option·JSON schema

따라서 renderer·layout 결과를 바꾸는 작업이 아니며 별도 PDF/SVG visual sweep은 적용 대상이 아니다.

## 4. 최신 devel 정합

장기 검증 전에 `upstream/devel`이 5개 commit 이동한 것을 확인했다. picture edit module 관련 upstream
변경과 직접 경로 충돌이 없음을 확인하고 dry merge 뒤 merge commit `49d0a61ea`로
`upstream/devel@97c4d7155`를 반영했다. PR 게시 승인 뒤 장기 baseline test 분할 3개 commit이 더
반영되어 `upstream/devel@f54408110`을 merge commit `143e3032d`로 추가 반영했다. review 보정 직전에는
장기 baseline 분할과 PR 전 Rust lint 절차가 포함된 `upstream/devel@2deb3dd61`을 merge commit
`0240e043e`로 반영했다. 직접 충돌은 없었고 최신 기준에서 새 lint 묶음과 전체 integration을 다시 실행했다.

## 5. 검증 결과

| 게이트 | 결과 |
| --- | --- |
| focused `test-caption` nextest | 5/5 pass, run `bd1bcaa0-dd48-415d-ab11-5a325cdd718d` |
| focused CLI catalog nextest | 20/20 pass, run `c7ba0e7a-ec8f-4b5b-a9ca-4643d1a6078e` |
| native·WASM32·workspace all-target Clippy | 모두 `-D warnings` 통과 |
| workspace build | `cargo build --locked --workspace --target-dir target/pr-review` 통과 |
| 전체 integration nextest | 8,686/8,686 pass, 43 skipped, 1 slow |
| 전체 nextest run | `554b5740-99fd-450e-982d-62c9c8810420` |
| integration manifest | 1,032 sources, 4,535 attrs, 48/48 targets |
| source-side unit tier | 4,221 tests, 299 modules, 정책 검사 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

generated suite·manifest와 `target/`, `output/`은 ignored 검증 산출물이며 제출 diff에 포함하지 않았다.

## 6. 계획 대비 결론

계획한 all-fail·partial-fail·all-pass 경계, fail-closed 구현, CLI 문서와 최신 devel 전체 회귀를 모두
완료했다. review 보정으로 setter와 verifier의 해석 불일치와 verification을 증명하지 못하던 테스트 공백도
닫았다. 구현 범위 밖의 renderer·layout·workflow 변경 없이 실제 false-pass만 차단했다.

작업지시자 승인 뒤 code candidate `988b9c85f`를 원격 branch에 push하고
[PR #6391](https://github.com/edwardkim/rhwp/pull/6391)을 생성했다. review 보정 code candidate
`d8ab820b0`과 trailing 기록을 push한 뒤 최신 PR head의 required GitHub Actions와 mergeability를 다시
확인하는 것이 남은 merge 조건이다.
