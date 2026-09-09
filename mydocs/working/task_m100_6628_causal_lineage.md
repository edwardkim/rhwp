---
kind: working
status: active
canonical: mydocs/working/task_m100_6628_causal_lineage.md
issue: 6628
last_verified: 2026-09-02
---

# #6628 Gym 운영 경계 정정의 원인 계보

이 문서는 v0.8.6 릴리스 중 Gym을 제품 게이트로 오인했다가 철회한 과정과 당시 WIP
계측을 보존한다. 현 작업의 수행 범위와 승인 단계는
[`../plans/task_m100_6628.md`](../plans/task_m100_6628.md), Stage 0의 파일별 처분은
[`task_m100_6628_stage0.md`](task_m100_6628_stage0.md)가 정본이다.

## 1. 정정 결론

태그 후 실행된 `Gym Release Gate`가 약한 오라클을 발견했을 때 처음에는 “릴리즈 전에
Gym 인증을 하지 않았다”를 운영 결함으로 판단했다. 이에 exact `devel` 인증과 Release
Binary·npm 게시의 증적 소비 경로를 로컬에서 구현했으나, 전수 검증 중 정답지의 권위까지
추적한 결과 이 판단은 잘못된 책임 경계에 기반했다.

Gym은 AI 에이전트가 rhwp CLI/API를 조합해 과제를 수행하는 기술을 학습·평가하는
벤치마크다. 제품의 HWP/HWPX 해석·조판·저장 결과가 외부 정답과 맞는지를 판정하는
릴리즈 오라클이 아니다. 따라서 Gym 실패는 벤치마크 결함으로 고치되 main 승격, 일반
devel/PR CI, tag, Release Binary, npm·extension 게시를 차단하지 않는다.

## 2. 정답지 권위 계측

`gym/packs/*/reference/*.json`과 task를 전수 계측했다.

| 항목 | 수치 | 의미 |
|---|---:|---|
| task/reference | 1,035 / 1,035 | 모든 과제에 기준풀이 존재 |
| reference의 `run` step | 993 | 기준풀이 대부분이 rhwp 자체를 실행 |
| answer spec | 686 | 제출 답안 생성 좌표 |
| 현재 rhwp live command로 계산 | 682 | 99.4%가 채점 대상과 같은 구현을 오라클로 사용 |
| 상수 answer | 4 | 외부 정답이 아니라 task 작성자가 박은 계약값 |

이 구조에서 baseline 100%는 “현재 rhwp로 만든 기준 제출물이 현재 rhwp 채점기를
통과한다”를 뜻한다. discrimination 100%는 “준비된 음성 대조가 탈락한다”를 뜻하고,
trajectory 100%는 “다단계 기준풀이의 마지막 의미 step이 채점 결과에 필요하다”를
뜻한다. 셋 모두 벤치마크 판별력에는 유효하지만 다음은 증명하지 않는다.

- 한컴 편집기·명세·독립 구현과의 결과 일치
- 같은 버그를 기준풀이와 채점기가 공유하지 않음
- 제품 보안·성능·패키징·플랫폼 실행 적합성
- 릴리즈 후보가 사용자 문서를 더 정확하게 처리함

구·신 rhwp를 비교하는 `release_diff.py`도 변화 위치는 보여주지만 어느 쪽이 옳은지는
판정하지 못한다. 독립 오라클이 없는 self-differential 결과를 릴리즈 승인으로 승격하지 않는다.

## 3. 약한 오라클과 기준풀이 결함

태그 후 최초 실행은 19개 task, 28개 음성 대조 false-pass를 발견했다.

- live-only answer: 제출 `answer.json`을 읽지 않고 현재 rhwp 값만 검사
- artifact 형식·내용 미검사: 파일 존재·크기·동일 해시만 확인
- 실패 봉투 허용: 정상 결과가 필요한 과제에서 exit 3 오류 봉투도 허용

전수 positive baseline을 처음 실행하자 19건 밖에도 stale command, 안전하지 않은
제출 경로, 누락된 기준풀이 step, 오래된 결과 좌표가 여러 pack에 남아 있음이 확인됐다.
이것은 Gym을 구현할 때 “기준풀이 통과 + 음성 대조 거부 + 경로 필요성”을 함께 전수
검증하지 않았다는 증거다.

경계 정정 WIP 당시 로컬 실측은 다음과 같다. Stage 1~4에서 권위 원장과 현행
전수 계약을 적용해 다시 측정하기 전에는 최종 #6628 결과로 인용하지 않는다.

| 감사 | 결과 |
|---|---|
| positive baseline | 21 pack, 1,035/1,035 통과 |
| discrimination | 1,035 task, 1,511/1,511 음성 대조 거부, false-pass 0 |
| trajectory | 감사 대상 239/239 load-bearing, theater·예외·도구 오류 0 |
| work-receipt 계약 | 25건 통과, 56/56 기준풀이 통과 |
| 관련 Python 단위 모듈 | 318건 통과 |
| Rust generated regression suite 015 | 160/160 통과 |

## 4. Gym이 발견한 실제 제품 결함

`batch fill --verify`는 필드 문자열을 바꾼 뒤 해당 문단의 LineSeg를 무효화했지만,
저장·재로딩 시 LineSeg가 다시 합성되어 verify에서 줄 수 불일치가 발생했다. 검증을
완화하지 않고 필드 편집 직후 실제 body/cell/textbox 문단을 reflow하고 vpos를 다시
계산하는 제품 수정 후보를 만들었다. 이 patch는 #6628에서 분리해 제품 이슈
[#6641](https://github.com/edwardkim/rhwp/issues/6641)과 로컬 `task_m100_6641`에
보존했다. 기존 integration source의 강화 후보는 exit 0, `identical=true`,
`diffCount=0`을 요구한다.

이 결함은 Gym이 발견한 유용한 부산물이지만, Gym 전체를 제품 릴리즈 게이트로 삼아야
한다는 근거는 아니다. 제품 수정과 벤치마크 판정은 서로 다른 증적과 검증 절차를 따른다.

## 5. 수정된 운영 경계

| 변경 종류 | 일반 CI | Gym workflow | 릴리즈 의존 |
|---|---|---|---|
| 제품 Rust/WASM/Studio/packaging | 기존 영향도별 lane | 자동 실행하지 않음 | 제품 검증 결과 사용 |
| Gym task/reference/tool/test | 제품 worker 생략 | PR에서 빠른 Gym 계약 | 없음 |
| Gym 전건 평가 | 자동 실행하지 않음 | 메인테이너 수동 실행 | 없음 |
| devel/main push·v* tag | Gym 미실행 | 미트리거 | 없음 |

기존 workflow 파일 경로 `.github/workflows/gym-release-gate.yml`은 GitHub workflow ID를
고아로 만들지 않기 위해 유지하되 표시 이름을 `Gym Benchmark Validation`으로 바꾼다.
Gym 관련 PR은 경계·audit·baseline·discrimination·oracle·trajectory·work-receipt의
명시된 빠른 계약 모듈만 실행하고, 수동 실행에서만 baseline, discrimination,
trajectory와 oracle probe를 전건 실행한다. 전건 결과 artifact는 30일 보존하며 Gym
자체의 판별력 증적으로만 사용한다.

## 6. 철회한 로컬 구현

원격에 push되지 않은 다음 경로는 운영 경계 정정에 따라 제거·복원했다.

- exact `devel` Gym release certification producer
- Release Binary와 npm 게시의 Gym artifact consumer
- tag push 기반 Gym 자동 실행
- release candidate SHA/tree와 Gym 결과 결합
- `gym/tools/release_certification.py`
- `scripts/verify_gym_release_certification.py`와 관련 테스트

`.github/workflows/release-binary.yml`과 `.github/workflows/npm-publish.yml`은 Gym 결합 전
`upstream/devel` 내용으로 복원했다. 이 철회는 실패를 우회하는 것이 아니라 벤치마크
증적을 제품 품질 증적으로 오인한 설계를 제거하는 조치다.

## 7. 비용 계측

- full positive baseline: 약 23분 30초
- full discrimination: 약 6분
- full trajectory: 약 2분
- exact v0.8.4 빌드와 release diff: 비교가 10분 이상 진행됐으나 독립 정답을 제공하지
  못하므로 운영 경계 정정 후 중단

전건 Gym을 일반 PR·devel·main·release에 넣으면 캐시 상태와 무관하게 상당한 반복 비용이
발생한다. 60분 timeout은 수동 전건 벤치마크 workflow에만 적용한다.

## 8. 전수 Python 탐색에서 확인한 비차단 부채

2026-09-02 replay 직전 `python3 -m unittest discover -s scripts/tests -p
'test_gym_*.py'`는 약 8초에 3,121건을 실행했지만 5 failure, 4 error, 1 skip이었다.
실패는 오래된 전체 저장소 스냅샷과 환경 가정, 그리고 실제 pack-health 위반을 함께
수집한 결과다.

- `python` 실행 파일 권한·경로 가정: coverage CLI 3건
- profile/source 정문 문자열 스냅샷: 2건
- 16개 파일 연산자의 `CHECK_FIELD_HINTS` 누락
- pack-health 196건과 tutorial front matter 날짜 정문 snapshot

이 불일치를 숨기기 위해 시험을 느슨하게 만들지 않는다. Gym PR마다 이미 깨진 역사적
전수 묶음을 자동 실행하지도 않는다. Stage 2에서 환경 가정·낡은 source snapshot·실제
계약 위반을 분리해 전수 unittest를 정상화한다.

## 9. 경계 정정 WIP의 당시 검증

아래 표는 replay 전 WIP에서 통과했던 범위다. #6628 최종 검증이 아니며, Rust 항목은
#6641로 분리됐다. Stage 6에서 정리된 exact head를 다시 측정한다.

| 검증 | 결과 |
|---|---|
| workflow YAML BaseLoader 파싱 4개 | 통과 |
| CI classifier·policy | 78/78 통과 |
| workflow wiring·release policy | 67/67 통과 |
| Gym 빠른 contract job과 동일 묶음 | 686/686 통과 |
| 변경 Python 모듈 묶음 | 425/425 통과 |
| `gym/tools/audit.py --json` | 21 pack, 1,035 task/reference, issue 0 |
| `cargo fmt --all -- --check` | 통과 |
| native Clippy `-D warnings` | 통과 |
| WASM lib Clippy `-D warnings` | 통과 |
| workspace build | 통과 |
| workspace all-target Clippy `-D warnings` | 통과 |
| generated integration manifest | 1,110 source, 4,777 attrs, 48/48 target 통과 |
| regression suite 015 | 160/160 통과 |
| batch fill 핵심 회귀 단건 재확인 | 1/1 통과 |
| 변경 문서 link/redirect 검사 | 620개 문서, 이상 없음 |

문서 메타데이터 전수 검사는 이번 변경과 무관한 기존 4개 문서에서 16개 누락을
보고했다. 변경 문서의 front matter는 유효하며, 기존 부채를 이 작업에 섞어 고치지 않는다.

이 표 이후 정답 권위 원장과 Python 전수 계약이 미완료임을 확인했다. 따라서 #6628은
완료 상태가 아니며 Stage 1~6을 계속 수행한다. 원격 push와 PR은 아직 수행하지 않았다.
