# PR #3241 통합 적용 계획 — #3239 적응식 그림 crop 폴백

## 커밋과 적용 범위

| 원 SHA | 통합 SHA | 제목 |
| --- | --- | --- |
| `3847a548` | `6ce59e4` | imgDim 없는 그림 crop 적응식 폴백 복원 |

기준은 `upstream/devel@973de548faedc6709ef862a1a12aa7146c225ac5`, 가시성 브랜치는
`integrate/planet6897-20260724`다. #3224의 문서 커밋 뒤에 `git cherry-pick -x`로 적용했다.
`src/renderer/svg.rs`는 3-way 자동 병합됐고 사람이 해결할 충돌은 없었다.

## 단계

1. 실제 `samples/issue3239` HWP, HWP 2020 MCP 기준 PDF, Native Skia PNG 비교와 renderer 공통 gate를
   통합 tree에서 완료한다.
2. #3224와 개별 review/impl 문서, 검증 asset, 오늘할일을 같은 통합 PR에 포함한다.
3. 작업지시자 승인 뒤 원본 저장소 임시 head 브랜치에 push하고 `devel` 대상 PR을 연다.
4. 통합 PR의 최신 head가 green이면 승인된 방식으로 merge한다. 그 뒤 merge SHA와 #3239 해결 상태를
   확인한다.
5. 원 PR close/감사 코멘트, #3239 close, 원격·로컬 브랜치 정리는 merge 후 별도 승인 범위에서 처리한다.

## 롤백과 확인 사항

- 문제가 발견되면 이 커밋 하나를 통합 PR에서 revert해 #3224 문서 기록과 분리할 수 있다.
- 새 코드 보정은 필요하지 않았다. `tools/verify_issue3239.py`의 기본 `.exe` 경로는 사용 시 `--exe`로
  명시하는 운영상 주의점으로 기록했으며, 이번 통합 범위에서 변경하지 않는다.
- GitHub push, PR 생성, merge, close, comment는 작업지시자 승인이 필요하다.
