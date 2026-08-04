# PR #3360 검토 기록

## 라우팅·메타데이터

외부 collaborator 통합 검토 경로를 적용했다. 작성 시점 참고값: `kevin9327`의
`pr/fix-issue-3359-export-family-option-order` → `devel`, 최신 head `d8a5b08f6363`, 보류
comment/review 없음, 검토 branch `review/kevin9327-20260726`.

## 변경 검토

SVG/PNG/PDF/Markdown/render-tree/doclang export 명령의 파일 위치 인자와 옵션 해석을 통일한다.
#3352 `export-text`와 함께 적용했으므로 export 하위 명령 간 옵션 순서 UX가 일관된다.

## 검증·권고

통합 release-test 전수·clippy·fmt·diff check 통과. 구현은 CLI 인자 해석이며 새 fixture나
layout 변경이 없어 IR baseline·visual sweep 대상이 아니다. 세부는
[통합 구현 기록](pr_3345_review_impl.md)을 참조한다.

**수용 가능**. #3359는 통합 PR merge 뒤 종료 상태를 확인한다.
