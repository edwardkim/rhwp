# #9154 Ubuntu runner 기준·Native ABI 보완 결과

## 구현과 기준

[이슈 #9154](https://github.com/edwardkim/rhwp/issues/9154), [수행 계획](../plans/task_m100_9154.md).
기준 devel은 `1e2488ebd432deb7c326be532f8a91945f260231`, 최초 로컬 구현·실행 source는 `9815b65c3606f42ae57b68df66293f921a1db254`다.

GitHub의 `ubuntu-latest` 26.04 전환에 따라 제품 빌드·검사와 Native host Linux 패키지의
기준을 Ubuntu 24.04로 고정했다. archive producer/consumer도 같은 OS를 사용한다.
기존 CLI Linux 24.04/24.04-arm, macOS·Windows와 metadata 제어 작업은 유지했다.
Oracle·layout advisory는 실제 제품 CLI를 빌드하므로 포함했고, Gym benchmark·skill-router는
별도 도구 경로로 유지했다.

Fuzz·Full Renderer의 기존 수동 입력에 `runner_image`를 추가했다. 기본·정기 Fuzz는 24.04이며
26.04는 명시적 대조만 허용한다. 임의 label·입력 불일치는 checkout 전에 실패한다.
cache key·restore prefix에 OS 버전을 포함해 이전 Linux namespace를 복원하지 않는다.
새 workflow·자동 26.04 job이나 테스트 개별 호출은 추가하지 않았다.

Native 패키징은 실제 ELF의 version needs를 검사해 `GLIBC_2.39` 상한을 넘으면 로드 전에
실패한다. exported version·GLIBCXX를 glibc 요구 버전으로 세지 않으며 분석 실패도 거절한다.
이후 기존 capability·packaging·headless 검증을 실행한다. 제품 Rust·JS·조판·저장 로직과
패키저 source는 기준 devel과 같다. [source 동일성](../working/assets/issue9154/product-source-equivalence.json)을 확인했다.

## 실제 검증

[검증 영수증](../working/assets/issue9154/validation.json)과
[Native 실행 결과](../working/assets/issue9154/native-runtime-results.json)를 보존했다.

| 검사 | 실제 결과 |
| --- | --- |
| workflow 계약 | 262 PASS, ABI·수동 OS 선택 검사 포함 |
| CI 영향·정책·신뢰 재사용 | 339 PASS |
| 기존 Native package 계약 | 14 PASS |
| actionlint 1.7.12 | 변경 workflow 14개 PASS |
| Native FFI 실제 release build | Ubuntu 24.04.5 / Rust 1.93.1, PASS |
| 실제 `.so` 최대 glibc 요구 | **2.35**, 상한 2.39 이내 |
| Ubuntu 24.04 소비 | glibc 2.39, capability·IME·Undo/Redo·저장·재열기 PASS |
| Ubuntu 26.04 소비 | glibc 2.43, 같은 package로 같은 검사 PASS |
| 독립적인 상한 초과 대조 | 26.04에서 만든 GLIBC_2.43 `.so`를 gate와 24.04 loader가 각각 거절 |

실제 라이브러리 SHA-256: `1cd1e0bfc769cdb6666c5b8706907a69c0350e60ba6594f25d930c7f21fbd125`.
최초 cold build는 18분 54초였으며 구현 commit에서 같은 명령을 재확인한 결과는 0.24초였다.
최초 빌드 시작 commit과 최종 실행 source를 [빌드 영수증](../working/assets/issue9154/native-build-result.json)에
구분했고 두 source의 제품 경로가 같음을 확인했다. 새 OS의 GitHub cold build 시간은 이 로컬 값으로 예측하지 않는다.

두 소비 환경은 공식 Ubuntu image에서 공개 `fonts-noto-cjk`와 동적 의존성을 설치했다.
서로 다른 Noto package 버전도 원문 로그에 남겼다. 패키지는 읽기 전용으로 옮기고 checkout·
네트워크·개인 글꼴 없이 `/tmp`에서 실행했다. 편집 후 재열기한 `각`의 렌더 버퍼도 blank와
다르며 양 OS의 SHA가 같았다. [추가 렌더 확인](../working/assets/issue9154/reopened-render-results.json),
[24.04 원문](../working/assets/issue9154/runtime-24.04.log),
[26.04 원문](../working/assets/issue9154/runtime-26.04.log)을 연결한다.
이는 단일 headless ABI·기능 대조이며 독립 한컴 조판 정확도나 Qt·GPU·macOS·Windows 검증이 아니다.

## 기존 검사·린트의 선행 근거

Render Diff cache 계약은 원 해결 [#7474](https://github.com/edwardkim/rhwp/pull/7474),
commit `def947c2149`에서 build mode 간 복원 금지와 primary key 재사용을 보호한다.
그 계약을 보존하면서 OS prefix만 추가하고 이전 OS key가 없음을 함께 검사했다.

Oracle 안내문은 원 commit `8b684ac27b5`와 통합 [#5425](https://github.com/edwardkim/rhwp/pull/5425),
layout 안내문은 통합 [#5530](https://github.com/edwardkim/rhwp/pull/5530)의 `bef280b001b`를 추적했다.
기존 Markdown backtick의 의도적인 literal 출력에 생긴 SC2016은 quoted heredoc으로 표현했다.
실제 shell 실행의 수정 전후 안내문 196·189바이트가 각각 같음을
[독립 실행으로 확인](../working/assets/issue9154/advisory-note-equivalence.json)했다.
검사 비활성화나 허용치 완화는 하지 않았다.

## 재현과 원격 처리 결과

빌드·패키징 명령은 [Native 패키징 안내](../../bindings/Native/examples/PACKAGING.md)에 있다.
Docker base digest와 설치 recipe는 [이미지 기록](../working/assets/issue9154/docker-runtime-build-results.json),
[Dockerfile](../working/assets/issue9154/Dockerfile.runtime.txt)에 보존했다.
상한 초과 대조는 [공개 C source](../working/assets/issue9154/newer-glibc-control.c.txt)를
26.04에서 `gcc -x c -shared -fPIC -fno-builtin -o control.so <source> -lm`으로 빌드한 뒤
`python3 scripts/verify-linux-abi.py --artifact control.so --max-glibc 2.39`로 재현한다.
[거절 결과](../working/assets/issue9154/newer-glibc-control-result.json)와
[24.04 loader 오류](../working/assets/issue9154/newer-glibc-24-loader.log)는 실제 실행 결과다.

PR [#9155](https://github.com/edwardkim/rhwp/pull/9155)의 최종 code head는 `d9868f4ef1e506918bb3eceece5453333b047e64`이며
devel merge는 `1ea5d6df80eebcdc16cf451b4e76d4a48d5edc2a`다. 새 head Full CI·CodeQL·Render Diff·Adapter·Proptest·정책과
Fuzz 두 OS 12개 build/run이 모두 성공했다. Archive 13,523건·선택 Native 245건·Studio 2,805건이 통과했다.
실제 Native package 3플랫폼·Full Renderer 두 OS는 producer source `de94e7cc23`의 증거를
사용자 승인에 따라 동일 실행 정의의 컴포넌트 증거로 유지했다. 최신 devel 병합의 새 바이너리와 혼동하지 않는다.

Fuzz cold build의 기본 단일 codegen unit에서 로컬 compiler peak 16.64 GiB를 관측했다.
같은 SDK/source/lockfile의 16개 단위 target build는 13.18 GiB였으며 양 구성의 실제 퍼징·느린 입력이 통과했다.
이후 실제 GitHub 두 OS 12개 빌드의 peak 최대는 8.092 GiB였다.
원격 OOM은 직접 확인되지 않았으므로 이전 exit 143의 원인을 확정하지 않는다.
60분 job·60초 퍼징·입력별 30초·RSS 2 GiB·ASan·coverage·6개 타깃을 유지했다.

[최종 검토](../pr/archives/pr_9155_review.md)와 [원격 검증 원장](../pr/assets/pr9155/README.md)에
개별 source·run·job·digest·실제 소비 결과와 제한을 보존했다. Full Renderer browser 378 PNG·105 SVG는
OS 간 같지만 Native 34 PNG에는 실제 차이가 있었고 report-only parity는 22/72 통과였다.
이를 Native 전체 동일성이나 독립 한컴 조판 개선으로 승격하지 않는다.
cache budget/read-only 경고도 유지하며 cache 삭제·예산·과금 설정을 변경하지 않았다.
CI 변경의 옵션 2 별도 기록 PR로 review·오늘할일·실제 merge SHA를 반영한다.

정기 Fuzz는 main workflow를 사용하므로 devel 반영만으로 적용 완료가 아니다.
main 적용 범위·검증 후보를 별도로 준비해 승인받고 실제 GitHub runner의 수동 선택·cache·
Native artifact 동작을 확인하기 전에는 #9154를 자동 종료하지 않는다. PR은 `Refs #9154`를 쓴다.
기존 Fuzz exit 143의 종료 계기는 미확정이며, 이번 변경을 그 근본 원인의 입증·영구 해결로
기록하지 않는다. 부모 #536은 OPEN을 유지한다.
