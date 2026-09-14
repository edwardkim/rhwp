# #3790 정상화 PR 제출 초안

상태: 로컬 준비, 원격 push·Open PR 생성 승인 대기. PR 번호 미확정.
제목: `fix(ci): devel 전용 impact controller 경계와 trusted reuse v6 계약 정상화`

아래 본문은 PR 생성 시 사용한다. 제출 직전 최신 base와 실제 제출 HEAD를 재확인한다.

## 변경 요약

- Controller를 base=devel인 PR로 제한하고, main 대상 PR 및 비PR 완료 이벤트의 감사·게시를 차단합니다.
- live PR 신원과 게시 직전 base/head를 검증하며, 비대상·stale 사유와 내부 실패를 구별합니다.
- CI·CodeQL·Render Diff의 trusted reuse 소비 버전을 실제 발행 v6에 맞추고 중복/누락 필드를 거부합니다.
- 기존 classifier, advisory, main·release 검증, 승격 preflight, 취소·증거 수렴·병합 후 중복 CI 제거를 유지합니다.

## 관련 이슈

Refs #3790. 이번 PR 병합만으로 이슈를 닫지 않습니다. Controller YAML은 정상 main 승격 후
활성화되므로 실제 운영 확인까지 남아 있습니다.

## 테스트

- 범위: CI workflow·Node/Python 계약 테스트·운영 문서. Rust·Studio·sample 변경 없음.
- 통합 검증 commit: `adadcb4e212fe487de5f366e4ad4a2c8c90a3f70` (구현 `822a1f76d`).
- Node 437개, Python workflow 180개, promotion 33개 통과.
- YAML 4개, inline JS syntax 11개 통과; 기존 trigger·권한·concurrency·보호 job 유지 확인.
- 원격 devel `922946438fd89346a022df82e76648f88472f0cc`와 병합 시뮬레이션 충돌 없음.
- 실행 명령·증적·제한: [N3 보고서](https://github.com/edwardkim/rhwp/blob/task_m100_3790_normalize/mydocs/working/task_m100_3790_normalize_n3.md).
- Rust lint/회귀·WASM·시각 검증은 제품 및 해당 검증 입력 변경이 없어 비해당입니다.
- actionlint 미실행(기존 로컬 설치 없음); GitHub Actions 실제 실행은 PR 생성 후 확인합니다.
- [x] 변경 범위별 로컬 검증 수행. 후속 제출 기록은 검증한 실행 파일과 바이트 동일성을 확인합니다.
- [x] generated Rust suite·manifest, 임시 output·로그·private 자료를 제출하지 않습니다.
- [ ] 제출 직전 exact HEAD/base 재확인, 공백·변경 문서 링크 검사 결과를 최종 본문에 반영합니다.

## 효과와 운영 제한

일반 devel PR의 선택 실행을 보존하고 workflow 변경 PR의 증명된 후행 문서 commit만 재사용합니다.
이번 PR 자체는 `fail-closed:workflow-contract`의 Full CI 대상입니다.
빈/복수 PR 연결에서는 live 신원 조회가 남을 수 있으며 main run 기록 자체를 없애는 변경이 아닙니다.
실제 runner 시간 절감률은 아직 측정하지 않았습니다. 실패 시 이번 구현 범위의 revert PR로 복구합니다.

## 승인 후 실행할 명령 — 아직 미실행

```bash
git fetch upstream devel main
git merge-tree --write-tree upstream/devel HEAD
git diff --check upstream/devel...HEAD
git push --set-upstream upstream task_m100_3790_normalize
gh pr create --repo edwardkim/rhwp --base devel --head task_m100_3790_normalize \
  --title 'fix(ci): devel 전용 impact controller 경계와 trusted reuse v6 계약 정상화' \
  --body-file <위 본문을 추출하고 최종 검증 SHA를 기록한 UTF-8 Markdown 파일>
```

이 문서 전체를 PR 본문으로 보내지 않는다. 번호를 받은 뒤 기존 metadata 분류와 self-review 절차를
수행하며, 새 PR의 현재 SHA·CI와 별도 병합 승인을 확인한다.
