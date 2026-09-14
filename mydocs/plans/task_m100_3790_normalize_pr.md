# #3790 정상화 PR 제출 초안

상태: [PR #7129](https://github.com/edwardkim/rhwp/pull/7129) 제출 완료.
후속 최소 보정은 로컬 검증 완료, push·최신 CI·병합 미완료.
제목: `fix(ci): devel 전용 impact controller 경계와 trusted reuse v6 계약 정상화`

아래 본문은 기존 PR의 현행화 초안이다. 원격 본문은 별도 승인된 게시 시 갱신한다.

## 변경 요약

- Controller를 base=devel인 PR로 제한하고, main 대상 PR 및 비PR 완료 이벤트의 감사·게시를 차단합니다.
- live PR 신원과 게시 직전 base/head를 검증하며, 비대상·stale 사유와 내부 실패를 구별합니다.
- CI·CodeQL·Render Diff의 trusted reuse 소비 버전을 실제 발행 v6에 맞추고 중복/누락 필드를 거부합니다.
- 기존 classifier, advisory, main·release 검증, 승격 preflight, 취소·증거 수렴·병합 후 중복 CI 제거를 유지합니다.

## 관련 이슈

Refs #3790. 2026-09-14 최종 결정은 advisory 유지와 v6 계약 정합성 복원입니다.
최신 CI 성공·self-review·devel 병합 후 결정과 결과를 기록하고 이슈를 종료합니다.
Controller YAML의 main 활성화는 정상 승격 시점에 따르며 이슈 종료와 구분합니다.
실행 순서·빈도 재설계는 이번 범위에 추가하지 않습니다.

## 테스트

- 범위: CI workflow·Node/Python 계약 테스트·운영 문서. Rust·Studio·sample 변경 없음.
- 최초 통합 검증 commit: `adadcb4e212fe487de5f366e4ad4a2c8c90a3f70` (구현 `822a1f76d`).
- Node 437개, Python workflow 180개, promotion 33개 통과.
- YAML 4개, inline JS syntax 11개 통과; 기존 trigger·권한·concurrency·보호 job 유지 확인.
- 원격 devel `922946438fd89346a022df82e76648f88472f0cc`와 병합 시뮬레이션 충돌 없음.
- 실행 명령·증적·제한: [N3 보고서](https://github.com/edwardkim/rhwp/blob/task_m100_3790_normalize/mydocs/working/task_m100_3790_normalize_n3.md).
- Rust lint/회귀·WASM·시각 검증은 제품 및 해당 검증 입력 변경이 없어 비해당입니다.
- actionlint 미실행(기존 로컬 설치 없음).
- 제출 head `b0b150081`의 CI에서 `test_workflow_contract_wiring.py` 기대 목록 누락이 검출되었습니다.
  새 연결 테스트를 기대 목록에 추가했고, CI YAML의 `Validate CI impact classifier`와
  `Validate workflow contracts` 실행 명령 전체를 로컬에서 그대로 재실행하여 모두 통과했습니다.
  이전 Python discover의 파일명 패턴이 이 검사를 누락했음을 N3 보고서에 정정했습니다.
  보정 후 원격 CI 성공은 아직 확인하지 않았습니다.
- [x] 변경 범위별 로컬 검증 수행. 후속 제출 기록은 검증한 실행 파일과 바이트 동일성을 확인합니다.
- [x] generated Rust suite·manifest, 임시 output·로그·private 자료를 제출하지 않습니다.
- [ ] 제출 직전 exact HEAD/base 재확인, 공백·변경 문서 링크 검사 결과를 최종 본문에 반영합니다.

## 효과와 운영 제한

일반 devel PR의 선택 실행을 보존하고 workflow 변경 PR의 증명된 후행 문서 commit만 재사용합니다.
이번 PR 자체는 `fail-closed:workflow-contract`의 Full CI 대상입니다.
빈/복수 PR 연결에서는 live 신원 조회가 남을 수 있으며 main run 기록 자체를 없애는 변경이 아닙니다.
실제 runner 시간 절감률은 아직 측정하지 않았습니다. 실패 시 이번 구현 범위의 revert PR로 복구합니다.

## 최초 제출 시 사용한 절차 — 보존 기록

```bash
git fetch upstream devel main
git merge-tree --write-tree upstream/devel HEAD
git diff --check upstream/devel...HEAD
git push --set-upstream upstream task_m100_3790_normalize
gh pr create --repo edwardkim/rhwp --base devel --head task_m100_3790_normalize \
  --title 'fix(ci): devel 전용 impact controller 경계와 trusted reuse v6 계약 정상화' \
  --body-file <위 본문을 추출하고 최종 검증 SHA를 기록한 UTF-8 Markdown 파일>
```

최초 제출 절차는 완료되어 #7129가 존재한다. 위 `gh pr create`를 다시 실행하지 않는다.
후속 절차는 최신 base 확인 → 보정 push·기존 PR 본문 현행화 → 최신 SHA CI 확인 → self-review
→ 승인된 병합·이슈 결정 게시 및 종료 순서다. 이 문서 전체를 PR 본문으로 보내지 않는다.
