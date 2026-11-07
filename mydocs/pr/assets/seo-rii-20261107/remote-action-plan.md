# seo-rii 29건 원격 수용안 — 로컬 검증 완료·승인 대기

현재 공개 댓글·원격 push·Open PR·merge·close는 승인받거나 실행하지 않았다. [최종 17개 검증 묶음](local-gates.json)을 모두 통과했고 전체 nextest 12,825건·Native 집중 194건·Native 공식 3종 4,077건·Studio 2,745건을 확인했다. Native/fresh WASM 각 24쪽은 모두 raw 90% 이상이며 최신 devel과의 PNG 48쌍이 같다. 테스트 주석 보완은 [실행 본문 동일성](test-header-metadata-receipt.json)으로 검증했다.

## 대상과 원 history

대상은 #8666·#8667·#8668·#8669·#8670·#8671·#8672·#8673·#8674·#8675·#8702·#8703·#8704·#8705·#8706·#8707·#8712·#8713·#8714·#8716·#8718·#8720·#8721·#8722·#8723·#8724·#8726·#8727·#8728이다. 원 head와 source commit은 [접수 원장](intake.json)에 고정돼 있다. 마지막 Ready tip은 다른 28개 head를 모두 조상으로 포함한다.

원 fork head를 고치거나 반복 체리픽하지 않고, upstream의 `review/integration-536-seo-rii-native-hosts`에 검증한 통합 후보를 push한다. devel 대상의 Open 수용 PR 하나에서 현재 base·원 기여 history·메인테이너 보정·플랫폼별 검증 한계를 함께 검토한다. 원 commit을 보존하는 merge commit으로 통합하며 squash/rebase merge를 쓰지 않는다.

## 승인안에 포함해야 하는 두 효과

1. **Draft #8646의 선행 P73 코드 간접 수용:** 해당 두 commit이 Ready 29개 history 모두에 포함돼 있다. 검증 범위에는 포함했지만 Draft를 별도 승인·Ready 전환하지 않았다. 원 history를 merge하면 #8646이 간접 MERGED로 표시될 수 있다. 이 효과까지 작업지시자가 허용하기 전 원격 통합을 진행하지 않는다. fork branch는 보존한다.
2. **기존 대조군 시각 차이에 한정한 예외:** `eq-01.hwp`의 본문 굵기·수식 간격/분수 모양과 `pic-crop-01.hwp`의 가는 테두리는 독립 Print와 차이가 남지만 최신 devel/후보의 같은 PNG가 바이트 단위로 같다. 전쪽 Native/fresh WASM 90% 이상만으로 차이를 면제하지 않는다. 이 기존 범위에 한정한 예외를 수용안에 명시하고 승인을 기다린다. 새 숫자 의미 오류·새 회귀·필수 검증 실패를 예외에 넣지 않는다.

Metal은 이미 승인한 default-disabled 실험 기반 예외만 사용한다. Windows D3D의 실제 compile/link·ABI·하드웨어 producer 성공과 최종 Flutter consumer sampling·지속 표시 미검증을 구분한다. [검증 범위](validation-scope.md)를 따른다.

## 승인 뒤 순차 실행

1. 최신 upstream/devel·대상 head·본인 계정 열린 PR·reviewer를 재조회한다. 검증 이후 source가 달라졌으면 새 변경을 대조하고 영향 범위를 재검증한다. 현재 base와 제출 후보의 `git merge-tree --write-tree`를 확인한다.
2. 승인한 upstream branch에 후보를 push하고 실제 remote SHA를 확인한다. 같은 head의 원본 TSV·대표 PNG raw URL을 다운로드해 MIME·바이트·hash를 확인한 뒤 Open PR 본문에 연결한다. 원 기여 SHA·보정 SHA·빌드/글꼴 환경·입력/Print 출처를 기록한다.
3. 새 head의 required CI·실제 로그·보안 검사를 확인한다. 실패 원인을 조사하며 workflow 취소·재실행·보안 metadata 분류·검증 기준 변경이 별도 승인 대상이면 현행 운영 규칙을 따른다. 로컬 성공으로 GitHub 실패를 덮지 않는다.
4. 정확한 CI head에서 self-review를 기록한다. 최신 devel과 mergeability·required check를 다시 확인하고 `--match-head-commit`으로 원 기여를 보존하는 merge를 실행한다.
5. merge SHA와 원 PR 29개·Draft의 간접 MERGED 상태를 API로 확인한다. 자동 반영되지 않은 Ready 원 PR은 통합 수용의 commit 연결·설명을 남기고 완료 처리한다. Draft의 추가 상태 변경은 승인한 간접 수용 범위와 구분한다.
6. [후속 처리 정본](../../../manual/pr_review/post_merge.md)에 따라 review archive·오늘할일·증적의 merge 확정값을 운영 기록으로 반영하고 devel을 fast-forward한다. comment가 가리키는 자산의 실제 devel 존재와 raw hash를 확인한다.
7. #8337은 실제 원본의 AB·CD·4·3 복원과 정상 글꼴 대조를 설명하고 완료 반영한다. #536·#8701 전체 로드맵은 OPEN을 유지한다. 원 PR마다 해당 단계의 검증·제한·메인테이너 보정과 감사 댓글을 게시하고 API 본문·이미지 표시를 확인한다.
8. fork branch·기본 Studio 산출물·다른 작업은 보존한다. 작업 전용 local/upstream branch·worktree/ref·Windows 임시 도구·작업 전용 Docker 이미지와 사용 종료된 공유 debug를 정리한다. 저장소 루트 `output/` 전체를 정본에 따라 제거하고 잔여 여부를 확인한다. 추가 백업·압축·이관은 만들지 않는다.

## 원격 승인과 최종 실행 기록

- 로컬 보정·필수 검증: 완료. source `263b1e753c`와 최종 제품 `cda7c8cb33`, source 동등성·actual SDK/runtime·전쪽 시각 증거를 보존했다.
- 위 원격 수용·기존 차이 예외·Draft 간접 수용 효과: 아직 승인 전.
- 최종 제출 SHA·필수 로컬 검증·GitHub PR 번호·CI run·merge SHA: 실제 완료 뒤에만 기록한다.

## 원격 승인·실행 확정

[통합 PR #8739](https://github.com/edwardkim/rhwp/pull/8739)의 head `0d2548fd88ea76b10e53eec841376b4bcbecbc4a`에서 필수 CI·self-review를 완료했고, 원 기여 이력을 보존하는 merge `f5f0fbee95b5fb0e32e99f8f639d228ae55389dd`로 수용했다. 작업지시자는 기존 eq-01 본문 굵기·수식 간격/분수 모양 및 picture-crop 가는 테두리 차이에 한정한 예외와 Draft #8646의 간접 MERGED 효과를 승인했다. 새 숫자 오류·회귀·필수 검사 실패는 예외에 포함하지 않았다. #536·#8701은 OPEN을 유지하며 #8337 완료·원 PR별 후속 댓글·정리는 정본 순서에 따라 이어 간다.

위의 승인 대기·미수행 표현은 당시 로컬 단계의 기록이며 현재 상태는 [최종 운영 기록](remote/completion.md)을 따른다.
