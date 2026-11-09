# Dependabot 19건 최종 검증 원장

기준 devel `024d610633ede166ebcbb915f9c90e7a9d0649d8`, 제품·의존성·workflow 검증 source `2320351d2325a41d900e91f4e07f559d4a924a7d`, 추가 하네스·고지 보정 `42f82843ec5c957e4e06c414e7f9d0ade7cf052b`. 원 head 19개를 보존했다. 공유 결합 graph를 검증한 결과이며 원 PR마다 검사를 반복한 것으로 집계하지 않는다. 원격 승인·새 head CI는 별도다.

- [19건 판정](../../dependabot_20261109_review.md), [원 head·reviewer·CI 실패 원인](source-review.md).
- [실제 명령·종료 코드·최소 글꼴·하네스 보정](validation-summary.md): 전체 회귀 13,005 PASS/기존 45 skip, 집중 83 PASS, Native 4,077 PASS/기존 12 ignored, Studio 2,778 PASS/기존 2 skip. 모든 적용 lint/build와 실제 AI·확장 브라우저 검증 통과.
- [Native/fresh WASM·입력/PDF/producer 해시·전쪽 TSV·확대 판독](visual-review.md): 각 전 9쪽 raw 최저 100.00%, base/후보 PNG 동일. 편람 Native 383쪽 최저 99.96%로 gate 통과.
- [실제 의존성 소비·원 CI 원인](dependency-review.md), [공급망·공식 archive·Action 계약·HTTPS 다운로드](supply-chain-review.md).
- [원격 처리안](remote-action-plan.md): 승인 뒤 통합 Open PR → exact-head CI·self-review·merge → 옵션 2의 별도 후속 기록 PR → 원 PR 19건 완료 댓글·동기화·종료 정리.

실행 중간 JSON·로그·원시 raster는 커밋하지 않는다. 정식 원본/독립 PDF는 기존 경로를 재사용하고 최종 TSV·대표 PNG와 이 Markdown 검토만 보존한다. 글꼴 프로그램·생성 JS/WASM·임시 profile은 제외한다. 대표 이미지는 [댓글 초안](comment-drafts/)과 PR 본문에 실제 사용한다.
