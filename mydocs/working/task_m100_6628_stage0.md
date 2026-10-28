---
kind: working
status: active
canonical: mydocs/working/task_m100_6628_stage0.md
issue: 6628
last_verified: 2026-09-02
---

# #6628 Stage 0 — WIP 계보 감사와 제출 구조 정상화

## 1. 감사 기준선

- branch: `task_m100_6628`
- base: `upstream/devel@51043f5f8d0453b9bc929233de443fa60cb3df4b`
- WIP head: `c4acec2f8c38d2d24c11cff8f6310be1488c3392`
- WIP tree: `0c0c91c1524a7ed6d0ad2434ae0ad033dbaf6060`
- 관계: base보다 7 commits ahead, 0 behind
- 원격 동명 branch: 없음

2026-09-02에 `git fetch upstream devel`로 다시 확인했으며 base는 변하지 않았다.
계획서는 메인테이너 승인 뒤 `active`로 전환했다.

## 2. 최종 WIP 트리 분류

일곱 커밋의 중간 이력이 아니라 `upstream/devel...HEAD` 최종 diff를 기준으로 분류했다.

| 범위 | 파일 수 | 추가/삭제 | 처분 |
|---|---:|---:|---|
| `gym/packs/**` | 122 | +1,608/-105 | 유지 후보. task/reference별 Stage 3 대사 필요 |
| Gym 문서 | 13 | +139/-114 | 유지 후보. 권위 용어를 Stage 1 모델과 맞춤 |
| Gym runtime/tool | 4 | +167/-8 | 유지 후보. 연산자·전수 실행 계약 재검증 |
| Gym Python test | 9 | +228/-250 | 유지 후보. Stage 2 전수 실패 정산 필요 |
| workflow | 2 | +96/-80 | hunk별 분리 |
| CI impact classifier | 6 | +78/-22 | 유지. Gym-only PR의 제품 worker 생략 계약 |
| 기타 CI test | 1 | +5/-0 | 제외. Native Skia 60분 상향만 검증 |
| `mydocs/**` | 4 | +198/-4 | 문서별 분리·재작성 |
| Rust 제품 코드·integration test | 2 | +88/-8 | #6628에서 제외하고 별도 제품 이슈로 이동 |

합계는 163개 파일, +2,607/-591이다. 아직 추가되지 않은 현 수행계획은 이 수치에
포함하지 않았다.

## 3. 릴리스 결합 잔존 검사

최종 트리에는 중간 커밋에서 도입했던 producer/consumer가 남지 않았다.

- `.github/workflows/npm-publish.yml`: `upstream/devel`과 차이 없음
- `.github/workflows/release-binary.yml`: `upstream/devel`과 차이 없음
- `gym/tools/release_certification.py`: 최종 diff 없음
- `scripts/verify_gym_release_certification.py`: 최종 diff 없음
- 관련 certification test와 release-channel policy test: 최종 diff 없음
- `.github`, `scripts`, `gym`의 `gym-certification`, `Gym certification`,
  `verify_gym_release_certification`, `release_certification` 잔존 참조: 각 0건

따라서 최종 **트리**는 릴리스 producer/consumer와 분리돼 있다. 다만 앞선 여섯
커밋을 그대로 PR에 싣는다면 철회된 결합 코드가 영구 Git 계보에 남는다. 일반 merge
commit 정책과 결합하면 과도기 커밋도 `devel` 역사에 들어가므로 논리 replay가 필요하다.

## 4. 파일·hunk 처분표

| 대상 | 처분 | 근거 |
|---|---|---|
| `.github/workflows/gym-release-gate.yml` | 유지·보강 | 파일 경로는 보존하고 Gym 관련 PR의 빠른 계약과 수동 전건 benchmark만 담당 |
| `.github/workflows/ci.yml`의 Gym test 제거 | 유지 | 일반 제품 CI에서 Gym runner 비용을 분리 |
| `.github/workflows/ci.yml`의 Native Skia 60분 상향 | 제외 | v0.8.6 릴리스 사고 대응이며 #6628 벤치마크 경계와 별개 |
| `scripts/tests/test_ci_impact_workflow.py`의 60분 test | 제외 | 위 timeout 변경의 파생 계약 |
| `scripts/ci-impact-classifier.cjs`와 관련 6개 test/fixture | 유지 | Gym-only 변경을 제품 Rust·frontend·render·CodeQL worker에서 분리 |
| `gym/**`, `scripts/tests/test_gym_*.py` | 유지 후보 | 약한 오라클 정산의 실체. Stage 1~4에서 task별 재검증 |
| `mydocs/manual/publish_guide.md` | 유지 | 제품 릴리스 비의존 운영 경계의 정본 |
| `mydocs/working/gym_release_gate.md` | 유지 | #5259 역사 기록에 현행 비차단 해석을 명시 |
| `mydocs/plans/task_m100_6584.md` | 제외 | 완료된 릴리스 계획을 #6628에서 소급 수정하지 않음 |
| `mydocs/working/task_m100_6584_gym_release_gate_normalization.md` | 경로·내용 재작성 | 유효한 원인 계보는 보존하되 issue/canonical/수치가 #6584와 구 기준선에 묶여 있음 |
| `mydocs/plans/task_m100_6628.md` | 유지 | 승인된 현 작업 정본 |
| Rust 2개 파일 | 별도 이슈·브랜치 | Gym이 발견했지만 제품 저장 왕복 결함이며 Rust 필수 검증 대상 |

## 5. 별도 제품 결함 후보

### 제안 제목

`[field] batch fill --verify가 필드 편집 후 LineSeg 재조판 누락으로 저장 왕복 불일치를 보고한다`

### 확인된 원인

`set_field_value_by_id`와 `set_field_value_by_name_at`은 필드 문자열을 바꾼 뒤 저장
LineSeg를 무효화하지만 실제 문단 소유자(body/cell/textbox)의 폭으로 즉시 reflow하지
않는다. 메모리 상태에서는 빈 줄 배열이 남고, 저장·재적재 경로는 줄을 다시 합성하므로
`batch fill --verify`가 동일한 편집 결과를 비교하면서도 종료 3과
`identical=false`를 보고할 수 있다.

WIP 수정은 편집 직전 본문 흐름 끝을 보존하고, 필드 편집 직후 실제 소유 문단을
reflow한 뒤 vpos를 다시 잇는다. 기존 `tests/batch_fill_contract.rs`의 계약은 정상
필드 채움에 exit 0, `identical=true`, `diffCount=0`을 요구하도록 강화했다.

### 별도 완료 조건

1. `samples/field-01.hwp` 기반 정상 필드 채움이 저장 왕복 뒤 동일 판정을 낸다.
2. body, table cell, textbox 경로가 각 소유 폭으로 reflow된다.
3. 검증기 예외나 diff 무시 규칙을 추가하지 않는다.
4. Rust 필수 lint 묶음과 기존 integration 계약을 모두 통과한다.
5. #6628 diff에는 해당 Rust source와 test가 남지 않는다.

제품 결함은 [#6641](https://github.com/edwardkim/rhwp/issues/6641)로 등록했다. 기존
WIP의 두 파일 patch는 로컬 `task_m100_6641@ce2fb30b868f`에만 보존했으며 원본 WIP와
patch SHA-256 `e24390a58045aad1f8823308c5f60085a2ce2d72e53f2797a7ea24267b21b283`가
일치한다. assignee·milestone과 원격 branch는 추가하지 않았다.

## 6. replay 실행 결과

메인테이너 승인 뒤 다음 순서로 과도기 이력을 제거했다.

1. `refs/safety/task_m100_6628-pre-replay`를 WIP head에 만들었다.
2. 제품 결함 #6641과 로컬 `task_m100_6641` branch가 Rust 2개 파일의 patch를
   보존하는지 확인했다.
3. 현재 Gym branch를 `upstream/devel` 기준으로 mixed reset해 파일은 보존하고
   commit 계보만 되돌렸다. `--hard`는 사용하지 않았다.
4. 처분표의 제외 대상을 제거하고 다음 논리 단위로 다시 commit했다.
   - 승인 계획과 원인 계보
   - 약한 task/reference·검사 연산자 정산
   - baseline/discrimination/trajectory 도구·계약
   - Gym 전용 workflow와 CI impact 경계
5. 정리된 tree를 WIP tree와 비교했다.
6. 제품 patch 보존과 Gym replay를 검증한 뒤 safety ref를 삭제했다.

replay head는 `b6e5b5400868`, tree는
`717c84b877c5f6cbd29ebac1831f80105691f26e`다. 기능 replay 계보는 다음 다섯
커밋이며, 이 표를 고정하는 Stage 0 종료 증적 커밋 1개가 뒤따른다.

| commit | 역할 |
|---|---|
| `0718a4612` | 승인 계획, 원인 계보, Stage 0 감사, 운영 경계 |
| `84d89dc50` | 약한 answer/artifact 오라클 정산 |
| `ccb163ca2` | reference 실행·baseline·trajectory WIP 정산 |
| `d147cc6a0` | Gym 증적의 의미와 릴리스 비의존 문서 |
| `b6e5b5400` | Gym 전용 workflow·CI impact 경계 WIP |

원래 WIP와 replay tree의 endpoint 차이는 다음 여덟 경로뿐이다.

- 승인된 제외: `.github/workflows/ci.yml`의 timeout hunk,
  `scripts/tests/test_ci_impact_workflow.py`, `mydocs/plans/task_m100_6584.md`
- #6641 분리: `src/document_core/queries/field_query.rs`,
  `tests/batch_fill_contract.rs`
- #6628 신규 정본: 계획서와 Stage 0 보고서
- 경로·내용 정정: #6584 이름의 WIP 보고서를 #6628 원인 계보로 변경

다른 Gym 변경의 손실은 0건이다. `refs/safety/task_m100_6628-pre-replay`는 검증 뒤
삭제했으며 원래 Rust patch는 `task_m100_6641`에 남아 있다.

replay 중 직접 관련된 검사는 다음 결과를 냈다.

- 약한 오라클 관련 Python: 119/119 통과
- reference·trajectory 관련 Python: 317/317 통과
- `gym/tools/audit.py --json`: 21 pack, 1,035 task/reference, issue 0
- CI classifier·policy: 78/78 통과
- Gym benchmark workflow 계약: 8/8 통과
- workflow wiring: 3/3 통과
- workflow YAML 4개: 파싱 통과
- tutorial 전체 계약: 76건 중 날짜 정문 snapshot 1건 실패
- `test_gym_*.py` 전수 기준선: 3,121건, failure 5·error 4·skip 1로 replay 전과 동일

마지막 실패는 계획에 이미 분류한 Stage 2 대상이다. replay 승인 범위를 넘어 기대값을
바꾸지 않았으며 Stage 2에서 날짜 값이 아니라 front matter 형식·정본 관계를 검증하게 한다.

## 7. Stage 0 현재 판정

- 최신 base 확인: 통과
- 최종 트리 release producer/consumer 비의존: 통과
- WIP disposition 표: 완료
- Rust 제품 수정 분리: #6641과 로컬 branch에 보존 완료
- 논리 replay: 완료

Stage 0 종료 게이트를 충족했다. 다음 승인 지점은 Stage 1의 정답 권위 분류 모델과
1,035개 task 전수 원장 설계·구현이다.
