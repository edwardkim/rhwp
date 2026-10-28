---
kind: working
status: active
canonical: mydocs/working/task_m100_6628_stage4.md
issue: 6628
last_verified: 2026-09-03
---

# #6628 Stage 4 — 전수 감사 판정

## 1. 결론

Stage 4의 세 축을 현행 `task_m100_6628@05a834692`에서 실행했다.

| 축 | 결과 | 판정 |
|---|---|---|
| positive baseline | 21 pack · 1,033/1,035 통과 | **미충족** — BO05·BO15 |
| discrimination | 1,035 task · 1,511 control · false-pass 0 | 충족 |
| trajectory | 239/239 load-bearing · theater/예외/tool 오류 0 | 충족 |

positive의 두 실패는 task/reference 결함이나 제출 누락이 아니다. 두 reference가 요구하는
`batch fill --verify`의 필드 편집 후 LineSeg 재조판 제품 결함이며, 이미 분리한
[#6641](https://github.com/edwardkim/rhwp/issues/6641)의 로컬 수정 후보로 2/2 통과함을
별도 바이너리에서 확인했다.

따라서 #6628의 task·reference·채점 조건을 완화하지 않는다. #6641 제품 수정을 먼저
정식 처리한 뒤 그 exact 제품 tree에서 Stage 4 전수 증적을 다시 고정해야 한다.
현재 Stage 4 종료 게이트는 아직 통과하지 않았다.

## 2. 실행 경계와 재현 신원

- source head: `05a834692c05c517281eb0f59766332a1da8355a`
- current binary: `rhwp v0.8.6`
- current binary SHA-256:
  `14d0e8ef71f762a062ecf436c88e7d9b0ea719ce9a22178c8e51c09567ddddec`
- 공개 저장소 자산만 사용
- 기존 `gym/submissions/claude-fable-5`와 다른 제출 폴더 비변경
- positive 제출물은 격리된 임시 루트에서 만들고 실행 종료와 함께 제거

Stage 3 전수 discrimination 이후 현재 head까지 바뀐 파일은 다음 세 개뿐이다.

- `scripts/tests/test_gym_checks.py`
- `mydocs/plans/task_m100_6628.md`
- `mydocs/working/task_m100_6628_stage3.md`

task/reference/check runner/discrimination tool의 실행 의미가 같으므로 Stage 3의 전수
discrimination 원문을 이 Stage의 음성 축 증적으로 재사용했다. Rust 또는 Gym 실행
source가 바뀌면 재사용하지 않는다.

## 3. positive baseline 전수 결과

`build_baseline.py`의 실제 `process_pack`/`process_one_task` 경로로 21 pack을 전건
실행했다. CLI 기본 제출 루트를 쓰지 않고 임시 `sub_root`만 주입했다.

```text
taskCount        1,035
built            1,033
failed           2
skipped          0
missingArtifact  0
failedScore      0
buildError       2
elapsed          1,722.16s (28m 42.16s)
```

실패는 정확히 두 건이다.

| task | reference command | 실제 결과 | 분류 |
|---|---|---|---|
| `batch-ops/BO05` | 3행 `batch fill --verify` | 채움 3/3 성공, verify 차이 3, exit 3 | reference build-error |
| `batch-ops/BO15` | 2행 `batch fill --name-field myMsg01 --verify` | 채움 2/2 성공, verify 차이 2, exit 3 | reference build-error |

두 reference는 정상 저장 왕복 검증을 요구하므로 허용 종료 코드는 `[0]`이다. exit 3을
허용하거나 `--verify`를 제거하면 positive 수치만 맞추고 제품 결함을 숨기므로 적용하지
않았다.

## 4. 두 실패의 authority와 원인

권위 원장에서 BO05·BO15는 모두 다음과 같다.

- `authority=self-live`
- `baselineSource=self-live`
- caveat: `current-rhwp-dependent; not an independent product oracle`

즉 외부 한컴 정답과의 불일치가 아니라 현재 rhwp가 만든 산출물을 현재 rhwp의 저장
왕복 검증이 불일치로 판정한 내부 계약 위반이다. task의 최종 content check까지 도달하기
전에 reference run이 실패했다.

#6641에 기록된 원인은 필드 문자열 교체 후 저장 LineSeg를 무효화하면서 실제 소유
문단(body/cell/textbox)의 폭으로 즉시 reflow하지 않는 것이다. 저장·재적재 때 줄이 다시
합성되어 메모리 상태와 재적재 상태가 달라진다.

## 5. #6641 후보의 표적 교차검증

로컬 `task_m100_6641@ce2fb30b8`은 현재 `upstream/devel@51043f5f8` 바로 위의 독립
커밋이며 다음 두 파일만 바꾼다.

- `src/document_core/queries/field_query.rs`
- `tests/batch_fill_contract.rs`

후보를 detached 임시 worktree에서 기존 `target/pr-review` 캐시로 진단 빌드했다.
worktree는 검증 뒤 제거했다.

- candidate binary: `rhwp v0.8.6`
- candidate binary SHA-256:
  `db1216d07727d150d171444f0ab76ead52dd90f1760315cc44f70a78d105adb1`
- diagnostic build: 성공
- BO05 targeted positive: 통과
- BO15 targeted positive: 통과
- targeted count: built 2, failed/skipped/missing/score/build error 0

이는 #6641 후보가 Stage 4의 두 실패를 해결한다는 종속성 증거다. 다만 진단 빌드와 두
task 통과만으로 #6641의 제품 영향도·Rust 필수 lint·전체 회귀 검증을 승인한 것은 아니다.

## 6. discrimination 축

Stage 3의 동일 실행 의미에서 얻은 원문을 재검산했다.

```text
taskCount          1,035
controlCount       1,511
discriminating     1,035
falsePass          0
falsePassControls  0
load/build/tool 오류 0
toolFailed         false
validate_report    issue 0
elapsed            446.53s
```

`scoreErrors` 116건은 Stage 3에서 정산한 58개 artifact task의 의도된 잘못된
`answer.json` 조기 거부다. `input-copy`와 `garbage` 각 58건이며 CLI·도구 중단이 아니다.

## 7. trajectory 축

```text
python3 gym/tools/trajectory.py --bin target/debug/rhwp --json

kind             gymTrajectoryNecessity
schemaVersion    1.0
ok / exit        true / 0
taskCount        239
loadBearing      239
theater          0
exceptions       0
toolErrors       0
missingBin       false
toolFailed       false
trusted          true
validate_report  issue 0
elapsed          151.90s
```

다단계 reference 239건의 마지막 의미 step이 모두 실제 채점에 필요했다. BO05·BO15는
단일 run reference이므로 trajectory 대상이 아니며 positive 실패를 상쇄하지 않는다.

## 8. 권위 원장 재확인

```text
task/reference/entry  1,035/1,035/1,035
issue                 0
authority             self-live 987 · contract-constant 28
                      independent-fixture 20 · external-oracle 0
baseline source       self-live 1,031 · contract-constant 4
```

전수 감사 결과를 제품 릴리스 정답으로 승격하지 않는 Stage 1 경계는 유지된다.

## 9. Stage 4 판정과 정상화 순서

현재 판정은 **Stage 4 미완료**다.

- discrimination false-pass 0: 충족
- trajectory 239/239, theater·예외·도구 오류 0: 충족
- positive baseline 1,035/1,035: **미충족(1,033/1,035)**
- 승인 예외 원장: 0 — 제품 결함을 예외로 만들지 않음

권장 순서:

1. 이 중간 증적을 #6628 로컬 history에 보존하고 #6628 진행을 잠시 멈춘다.
2. `task_m100_6641`로 전환해 제품 수정의 수행계획·영향 범위·Rust 필수 게이트를 정식
   처리한다.
3. #6641의 exact 후보 head에서 제품 검증과 Stage 4 전수 Gym을 한 번만 실행한다.
4. #6641 병합 뒤 #6628을 최신 `devel`과 동기화한다.
5. 제품·Gym 실행 tree hash가 검증한 후보와 동일함을 확인해 증적을 재사용한다. 달라졌으면
   해당 전수 축을 다시 실행한다.
6. positive 1,035/1,035가 확인된 뒤에만 Stage 4를 완료하고 Stage 5로 간다.

이 순서는 이미 확인한 실패를 task/reference 완화로 우회하지 않으면서, #6641 병합 뒤
같은 전수 실행을 불필요하게 반복하지 않기 위한 것이다.

## 10. #6641 병합 후 최종 판정

위 1~9절은 `05a834692`에서 확인한 중간 판정을 역사적 원인 증적으로 보존한다. 이후 #6641은
PR #6673의 정상 merge commit `edeaeb28910f1b84f005aabb4ec0d0f183adc2a1`로 devel에
반영됐고, #6628 브랜치는 merge commit `3950ca15738311ef23e87233d981dd2d6197b953`에서 그
devel을 충돌 없이 받아들였다.

#6641 검증에서는 다음 두 계보를 섞지 않고 분리했다.

- Gym runner: `374c7416a6c2d9abe7c2701969de5f377b71183f`
- product candidate: `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`
- candidate binary SHA-256:
  `4334e35e3bfb7e892416e663c7a52dd055a7d82040d2a89447f32d25ccd02f34`

이 조합의 전수 결과는 positive 1,035/1,035, discrimination 1,035 task·1,511 control·
false-pass 0, trajectory 239/239 load-bearing·theater/예외/tool 오류 0이다. BO05·BO15도
별도 canary 2/2를 통과했다. 원문 계측과 실행 시간은
[`#6641 최종 보고서`](../report/task_m100_6641_report.md#5-6628-gym-인계-전수)에 보존했다.

증적 재사용 전에 다음 동일성을 기계적으로 확인했다.

1. `374c7416a..3950ca157` 사이의 `gym/core`, `gym/packs`, `gym/tools`, Gym 계약 시험과
   Gym workflow 실행 의미 diff는 0이다.
2. `7f1174f1d..upstream/devel@edeaeb289` 사이의 `src`, `tests`, `gym`, `scripts`, workflow
   diff는 0이다.
3. 현재 병합 tree에서 Gym Python 계약 3,149건, 구조 audit, oracle structural/selftest와
   authority ledger를 다시 실행해 모두 통과했다.

따라서 제품 결함을 task/reference/allowExits 완화로 우회하지 않고 Stage 4 종료 게이트를
충족했다. 이 판정은 공개 Gym 내부의 정합 근거이며 external oracle 0개라는 한계나 제품 릴리스
비의존 경계를 바꾸지 않는다.
