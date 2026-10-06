---
kind: report
status: active
canonical: mydocs/pr/archives/pr_6948_review.md
last_verified: 2026-09-09
---

# PR #6948 자체 검토

## 판정: 승인

로컬 검증 범위에서 수용 가능하다. 최신 PR head의 원격 CI와 병합 승인, 후속 fork PR의
실제 재사용 및 CodeQL 시간 단축은 별도 조건이며 아직 완료하지 않았다.

- PR: https://github.com/edwardkim/rhwp/pull/6948
- Issue: [#6947](https://github.com/edwardkim/rhwp/issues/6947), 상위 잔여 추적 [#6901](https://github.com/edwardkim/rhwp/issues/6901)
- 코드 후보: `93f309230`
- 기준 devel: `b6d2d39f9fd9012012ac69e4c70d223241b8cd56`
- 브랜치: `fix/fork-review-only-base-advance-20260909`
- 경로: collaborator self-merge 후보. owner에게 자동 review를 요청하지 않는다.

## 변경과 신뢰 경계

최종 fork head가 그대로인 상태에서 base에 허용된 Markdown만 추가된 경우를 지원한다.
신뢰된 pre-merge verifier가 immutable Git 객체의 최대 64개 first-parent 단계 및 최종
병합 tree 차이를 확인한다. 단순히 API의 파일명이나 PR CI 성공만으로 허용하지 않는다.

- 허용: `mydocs/` 아래 일반 Markdown 파일. rename은 삭제/추가로 양쪽 경로를 확인한다.
- 거부: source/test/workflow/sample, JSON, symlink/executable, CI 소비 기술 규약 2개,
  source 변경 후 되돌림, 이력/객체 누락, 한도 초과, 최종 코드 충돌 보정.
- 유지: upstream/fork 저장소와 PR/head/workflow/attempt 신원, 최종 head 성공, 병합 이전 완료,
  필수 worker 및 duration artifact 증거. privileged 경로에서 fork 코드를 실행하지 않는다.
- 기존 같은 저장소 및 과거 trailing candidate의 base 계약은 완화하지 않는다.

CodeQL은 devel push에서 내부 metric 3개만 제외한다. PR의 제품 경로 제한은 복사하지 않으며
전체 Rust 분석 경로와 기본 보안 쿼리 및 추출 진단은 유지한다. main push, 정기 및 수동 스캔은
기존 전체 분석을 유지한다. main CI Impact Policy Controller는 변경하지 않는다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| JS verifier 및 실제 workflow 하네스 | 134 통과, 실패/skip 0 |
| Python workflow 계약 | 233 통과 |
| 변경 workflow 2개의 actionlint | 통과 |
| 코드 변경 diff 공백 검사 | 통과 |
| 실제 #6933/#6945 Git 객체의 base 전진 증명 | 통과 |

실제 증명은 검사 base `74d0a68b74919761cc30343f7511dbc5d0fe32d3`, 최종 base
`9a96eef92458112d5d7998f4b3390261c0e3c811`, tested merge
`d1bcfa7e833e565cf267507a2ed6dd80c45d5e71`, 최종 merge
`b6d2d39f9fd9012012ac69e4c70d223241b8cd56`에 적용했다.

새 테스트는 문서 이동/shallow 객체 허용과 비허용 경로, mode, source 되돌림, 이력 한도,
다른 head, 위조 proof/attempt, 실패/pending 및 worker 증거 누락 거부를 확인했다.
CodeQL은 event/ref/language/선택 여부별 초기화의 상호 배타성과 필터 3개, 경로 및 기본
보안 쿼리 override 부재를 검사했다. 실제 CodeQL CLI의 query resolution과 단축 시간은
측정하지 않았다. Rust/제품 변경이 없어 Cargo/WASM/시각 검증은 수행하지 않았다.

## 잔여와 적용 조건

이 PR은 CI enforcement 변경이므로 자체 CI는 full 검증이 예상된다. 변경된 verifier는
이 PR 자체의 검증을 생략하는 근거로 사용하지 않는다. 원격 적용 후 후속 fork PR에서
실제 재사용 사유 및 CodeQL 선택 쿼리를 확인해야 한다.

#6933의 PR CI에는 duration artifact가 없었다. 따라서 base 전진 증명을 통과하더라도
CI 전체 재사용은 별도 duration 요건 때문에 거부될 수 있다. 이를 허위 성공으로 처리하지 않으며
duration publisher 및 Adapter/Proptest 등 #6901 잔여 전체를 해결했다고 주장하지 않는다.

## 병합 후 이슈 코멘트 계획

최신 head 및 병합 SHA의 실제 CI 결과를 확인한 뒤, #6947에 merge SHA, 실제 CI URL,
이 리뷰의 merge-SHA 고정 링크와 적용 범위를 UTF-8 body-file로 기록한다. 기존 동일 merge
댓글이 있으면 수정하고 중복 게시하지 않으며 API로 본문을 확인한다. 시각 변경이 아니므로
존재하지 않는 PNG/PDF를 게시하지 않는다. 실제 후속 fork 검증이 남으면 미완료로 명시하고,
#6901의 잔여를 임의로 닫지 않는다. 로컬/원격 변경 정리는 사용자 승인 및 post_merge 절차를 따른다.
