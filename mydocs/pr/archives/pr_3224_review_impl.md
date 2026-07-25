# PR #3224 통합 적용 계획 — r22 10k 보고서

## 커밋과 적용 범위

| 원 SHA | 통합 SHA | 제목 |
| --- | --- | --- |
| `650dbbe1` | `4761738` | r22 10k 서베이 최초 기록 |
| `e65ca4a` | `f7057ac` | ERR 실체 재분류 |
| `a83c33d` | `6bc39f7` | makeup 종결·집계 보정 |

기준은 `upstream/devel@973de548faedc6709ef862a1a12aa7146c225ac5`, 가시성 브랜치는
`integrate/planet6897-20260724`다. 세 커밋은 #3241보다 먼저 `git cherry-pick -x`로 적용했고 충돌은 없었다.

## 단계

1. #3241과 함께 로컬 통합 검증·개별 review 문서·오늘할일·검증 asset을 준비한다.
2. 작업지시자 승인 뒤 `integrate/planet6897-20260724`를 원본 저장소 임시 head로 push하고 `devel` 대상
   통합 PR을 만든다.
3. 통합 PR의 최신 head CI가 green인지 확인한 뒤 승인된 방식으로 merge한다.
4. merge SHA와 #3239의 실제 close 상태를 확인한다. #3224 원 PR close/감사 코멘트는 merge 후 별도
   승인 없이는 수행하지 않는다.
5. 원격·로컬 통합 브랜치 및 review fetch 브랜치는 후속 처리 완료 뒤에만 정리한다.

## 롤백과 확인 사항

- 이 PR은 보고서만 추가하므로 문제 발생 시 통합 PR에서 해당 문서 커밋만 되돌릴 수 있다. #3241 코드와
  결합된 Rust 변경은 포함하지 않는다.
- 통합 PR 제목·본문, push, GitHub comment, merge 및 close는 작업지시자 승인 범위다.
