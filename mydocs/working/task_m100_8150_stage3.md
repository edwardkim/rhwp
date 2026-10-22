# #8150 — 3단계 완료보고: CLI 검증과 사용 문서

- Issue: [#8150](https://github.com/edwardkim/rhwp/issues/8150).
- 승인: 2026-10-22 사용자의 「다음 단계 진행을 승인합니다」.
- 기준 devel: `2e087a5b4bc1515ad877e8020b30114639a59392`. 재확인 시 변경이 없어 rebase하지 않았다.
- 검증 checkout: `ac851c24e7da0a218d7cefd726fc5907f3658ab5`와 이 단계의 생성기·문서 보완.
  제품 Rust·회귀 test·Cargo·CI 소스는 1단계 `2d2d9b1921eaef5f586937739920dbee4689b75d` 이후 바뀌지 않았다.
- 중간 결과·명령·실행 로그: ignored `output/pr-review/issue8150/stage3/`.

## 적용 범위 정정

사용자가 「이번 작업은 rhwp cli 쪽만 해당되지 않나요?」라고 확인했다.
실제 변경을 다시 따라가면 `src/renderer/mod.rs`의
`cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))`로 제한된 Kitty 모듈과
Native CLI만 새 PNG 소비자다. 공통 줄 구성·paint·글꼴·PNG/Sixel producer와
Studio/WASM API는 변경하지 않는다.

[로컬 검증 4.3](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)의
CLI 행과 [실제 소비 경로](../manual/verification/renderer_backend_verification.md)를 기준으로
WASM 재빌드·Studio 실행·WASM Visual Sweep은 비해당으로 정정했다.
Rust 공통 lint 묶음의 WASM32 Clippy는 그대로 수행했다.
이전 단계의 포괄적인 Native/fresh WASM 필수 표현을 현재 구현의 실제 영향 범위로 바로잡은 것이다.
공통 렌더링 기능의 개선이나 WASM 시각 일치를 새로 주장하지 않는다.
준비만 했던 깨끗한 base-source 임시 worktree는 제거했다. 다른 worktree와 공유 target은 보존했다.

## 검증

| 항목 | 결과 |
| --- | --- |
| fmt·기본/WASM32/workspace/all-target/Native Clippy·workspace build | 모두 PASS |
| release-test 전체 nextest | **11,361 PASS·0 FAIL·45 skip** |
| Native lib와 관련 unit target | **4,041 PASS·0 FAIL·12 ignored** |
| Native missing-picture / 직접 PDF | 2 PASS / 14 PASS |
| Native Sixel / Kitty CI 호출 | 7 PASS / 8 PASS |
| suite manifest 정책 | 1,685 sources·48/48 targets PASS |
| 에이전트 생성기/스킬 Python 계약 / Kitty 독립 검사기 | 14 PASS / 2 PASS(13개 스트림) |
| CLI 글꼴 예시 / 프로필·호환 옵션 | 4 PASS / 8 PASS, 잘못된 값 2건 exit 2 |
| 문서 링크 / 매뉴얼 메타데이터 / 공백 | 10개 / 5개 / PASS |
| 최종 Native CLI 빌드 | PASS |

공유 `target/pr-review`에서 Cargo는 순차 실행했다. 잡 수 6개, test threads 4개를 사용했다.
전체 회귀는 다음 정식 nextest 경로로 실행했다.

```bash
CARGO_BUILD_JOBS=6 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --tests --test-threads 4 --no-fail-fast
```

상세 명령은 `rust-checks-results.json`, `nextest-result.json`, `manifest-early.log`에 있다.
마지막 Kitty 집중 실행은 로컬 helper 인자에 잘못 넣은 `--` 때문에 전체 target 빌드로 번져
중단했다(`native-kitty-ci-interrupted.log`). 이 실행은 검증에 포함하지 않았다.
CI와 같은 올바른 helper 호출에 `RUST_TEST_THREADS=4`를 적용해 8건을 다시 실행하고 통과했다.
제품·CI 코드를 변경할 사유는 없었다.
기존 렌더링 회귀는 실패하지 않았으며 검사·기대값·golden·허용치를 완화하지 않았다.
변경된 workflow는 Native 검사 호출 2건만 추가하며 trigger·권한·concurrency·required job 이름은 보존한다.
같은 파일 SHA의 1단계 classifier 45건·workflow 35건 PASS를 재사용했고 구문과 diff를 다시 확인했다.

## CLI 문서와 폰트 해결 방법

최초 focused 계약 검사에서 `agent_codex_contract::every_capability_command_has_a_codex_chapter`가
`export-kitty` 누락으로 실패했다. 생성기의 `FAMILIES`에 명령을 등록하고 바이너리의
capabilities·명령별 상세 help를 읽어 변환·렌더 장을 생성했다.
그 뒤 전체 회귀에서 agent codex 계약 2건이 통과했다(`agent-codex-red.log`, `nextest-all.log`).

사용자의 매뉴얼·폰트 옵션 지적에 따라 아래 문서를 함께 보완했다.

- [CLI 매뉴얼](../manual/cli_commands.md): 명령 계약과 Sixel/Kitty 공통 글자 누락 처방.
- [Kitty 가이드](../manual/export_kitty_command.md): Native 빌드·직접 출력·저장·SSH·한글 폰트·문제 해결.
- [Sixel 가이드](../manual/export_sixel_command.md): 글꼴 디렉터리/개별 파일·반복 옵션·원격 경로·재생성 예시.
- [문서 지도](../manual/README.md), [생성형 명령 장](../manual/agent_codex/40_변환과_렌더.md): 두 가이드의 탐색 경로.

글꼴은 렌더링 호스트에 공급한다. SSH에서는 원격 경로를 지정하며 로컬 터미널의 폰트를
바꿔 이미 전송된 이미지의 누락 글자를 복원할 수 없다. `.six`·`.kitty`는 공급 뒤 재생성한다.
생성형 문서는 생성기의 계약 사유도 고쳐 두 명령의 사용 가이드 연결을 유지했다.
그 밖의 기존 장 본문은 바이트가 같음을 확인했다(`codex-regeneration.json`).

`/tmp`에서 같은 공개 표 문서를 실행하며 두 명령 각각 절대 경로 글꼴 폴더와 반복 지정한
개별 글꼴 파일을 사용했다. 4건 모두 성공했고 Kitty PNG를 독립 복원해 한글 표시를 직접 확인했다.
명령·stdout·stderr·결과는 `manual-examples/`에 있다. 프로필 4종×호환 옵션 2종의 8건도
같은 조건 PNG와 픽셀이 같았고 잘못된 프로필/호환 값은 exit 2·빈 stdout이었다(`options/`).

## Native 출력 보존의 보조 비교

2단계의 4개 공개 문서 11쪽을 그대로 사용했다. 독립 한컴 Print PDF의 출처·대응·쪽수와
입력/PDF SHA는 `inputs.json`에 있다. 기준/후보 Native 전쪽 TSV를 먼저 산출했다.

| 입력 | 쪽수 | Native 전쪽 raw 최저 | base 대비 최대 하락 |
| --- | ---: | ---: | ---: |
| table-core | 3 | 96.64066% | 0.00000%p |
| footnote | 6 | 98.95047% | 0.00000%p |
| image-crop | 1 | 99.95079% | 0.00000%p |
| compound | 1 | 100.00000% | 0.00000%p |

각 독립 PDF와 전체 쪽수가 같고 누락 쪽이 없었다. base/head TSV는 11쪽 모두 같으며
Decimal로 하락을 판정했다. 전쪽 render tree의 구조·내용·좌표도 동일했다.
`scripts/visual_sweep.py --silhouette-only --rhwp-bin ... --compat 2022 --dpi 96`으로
각 4개 `--file-target`을 실행했다. 원본 TSV·명령은 `../{base,head}/native/`에 있다.
대표 첫 쪽 4개는 review·overlay·compare를 추가 생성해 직접 열었고 각 visual gate가 `passed`였다.
이때 silhouette-only의 `not_evaluated`를 최종 gate로 대체하지 않았다.

표 외곽·셀/뒤 본문·각주 표지와 선·crop·화살표·색과 한글을 대조했다.
표 글꼴·획 굵기, 각주선 농도와 복합 개체 하단 작은 표지의 기존 차이는 남아 있으며
한컴과의 완전한 일치로 보고하지 않는다. 이 비교는 Native 전송 경로의 기존 출력 보존 근거다.
WASM 결과나 조판 개선의 근거로 승격하지 않는다.

최종 Native CLI는 이번 단계에서 다시 빌드했으며 SHA-256은
`1b8801c19f7f3518ba548beaf72fdb830d4a4c6bbe5001a79ac4735d631d7cdb`다.
2단계 실제 터미널 검증 바이너리는 `9cd196a82f728271a51099b5a9721a92589575c8e4eea784c60d70a2d574101e`다.
바이너리 해시가 달라 새 실행 파일로 22건의 독립 복원·기존 PNG/Sixel 보존과 Native 전쪽 TSV를
다시 실행했다(`fresh-documents/`, `fresh-native-visual/`).
새 11쪽 PNG 픽셀·render tree·TSV 값이 이전 후보와 같고 22개 Kitty 스트림도
실제 터미널 검증에 사용한 스트림과 바이트가 같았다(`fresh-producer-equivalence.json`).
대표 review의 직접 판독은 같은 픽셀의 기존 후보에 대한 판독으로 구분해 연결한다.
제품·CI·계약·검사기 14개 파일의 SHA 불변도 확인했다(`final-source-identity.json`).

[2단계](task_m100_8150_stage2.md)의 독립 복원 22건·기존 PNG/Sixel 보존과 실제 kitty 로컬/SSH,
사용자의 Windows 11 WezTerm 직접 SSH 한글 표시 증거를 같은 제품 소스로 연결한다.
시스템 폰트·사용자 WezTerm 창·다른 작업의 파일은 바꾸지 않았다.

## 판정

CLI 범위의 승인된 3단계 검증과 문서화를 충족했다.
[최종 결과보고](../report/task_m100_8150_report.md)로 연결한다.
remote push·PR 생성·GitHub comment·merge·이슈 close는 수행하지 않았다.
