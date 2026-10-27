# PR #8328 증적 대응 정정 검토

## 최종 판정

**수용 가능 — 문서·증적의 입력 대응과 대체 표시만 수정했다.**

## 검토 범위

통합 PR #8319의 잘못된 대표 이미지 연결을 실제 검증 입력·Print·페이지로 정정했다. 원 PR 보고 문서 사례 22건은 유사 문서 대체 19건과 파일 없는 단위 계약 3건으로 구분했다. 대체 자료를 원 보고 실물 문서의 직접 검증으로 표현하지 않는다.

39개 유효 재현 입력의 Native/fresh WASM 전쪽 TSV와 같은 페이지 review/overlay를 연결했다. #8033의 저장본 추가 진단 실패는 성공 자료로 사용하지 않는다. 공식 배포처에서 내용 대응을 확인한 문서도 작성자 파일과 바이트 동일성을 뜻하지 않는다. 새로 찾은 문서의 전 페이지 90% 보정은 작업지시 범위가 아니다.

## 실제 검증

- 문서 후보: `ad321308c3e3acfb6ca0bd8f27525a20dbea3e9d`.
- base: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [후보 CI](https://github.com/edwardkim/rhwp/actions/runs/40513485940): completed / success.
- preflight: `fast_pass=true`, `all-review-only-no-code-impact`, candidate는 위 base다. heavy Rust/Studio worker의 skip은 문서-only 판정이며 새 전체 회귀 실행 성공으로 표현하지 않는다.
- 45개 대응 원장, 대체 여부, 실제 입력·PDF·PNG 해시와 전체 TSV 페이지·수치 검증 통과.
- 전체 TSV 78개를 포함한 raw 자산 88개 재다운로드: 커밋된 원본과 바이트 동일.
- Markdown 97개 내부 링크·공백 검사 통과. source/test/fixture 등록/baseline/workflow/지침 변경 없음 확인.
- 최신 base와 `git merge-tree --write-tree` 종료 코드 0.

[최종 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json), [공식 출처·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md), [CI 영수증](../assets/pr8319_evidence_alignment_20261027/pr8328-candidate-ci.json)을 함께 확인한다.

## 후속 처리

이 기록은 후보 CI 성공 후의 review-only trailing 기록이다. trailing 최신 head의 필수 check 성공을 별도로 확인한 뒤 머지한다. 이미 완료된 #8319의 기록 정정이므로 오늘할일·이슈 종료·새 contributor 댓글을 중복하지 않는다.

자료가 devel에 반영된 뒤 기존 45개 원 PR 댓글과 #8319 본문·기존 결과 댓글을 직접 수정한다. 새 정정 댓글은 만들지 않는다. 실제 merge SHA로 입력·PDF·TSV·review/overlay 링크를 고정하고 게시 내용을 재조회한다. contributor fork·원본 branch는 보존한다.
