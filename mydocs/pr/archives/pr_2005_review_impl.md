# PR #2005 처리 계획 - 옵션 1

## 목적

#1999의 RowBreak 거대 셀 쪽수 정합 보정을 코드 PR 안에서 처리하고, collaborator self-merge 후보 예외 경로의
옵션 1에 따라 review 문서, 대표 visual asset, 오늘할일을 같은 PR head에 포함한다.

## 적용 커밋

| 순서 | SHA | 내용 |
|---|---|---|
| 1 | `b95ec77cd` | RowBreak 거대 셀 쪽수 정합 보정 |
| 2 | `d1ece7f5c` | HWP 저장본 115쪽 회귀 가드 추가 |
| 3 | `ae87a55a9` | 14~16쪽 그림 흐름 원인 분석 |
| 4 | `475d9528e` | RowBreak 그림 flow 고립 방지 |
| 5 | `06fde211b` | TopAndBottom 지연 flow 흐름 보정 |
| 6 | `cc472c2a4` | RowBreak hard break 완화 회귀 보정 |
| 7 | `f334099e0` | RowBreak TopAndBottom 앞 fragment 보존 |
| 8 | `5f896e00f` | 최종 회귀 검증 기록 |

위 SHA는 `upstream/devel` rebase 후 코드 검증 기준 커밋이다. 옵션 1 문서/asset/오늘할일 커밋 이후 PR head는
다시 갱신된다.

## Stage 구성

1. #1999 코드/샘플/테스트/스테이지 문서 작성
2. 최신 `upstream/devel` 기준 rebase
3. 로컬 검증
4. PR #2005 생성
5. 옵션 1 문서 묶음 추가
   - `mydocs/pr/archives/pr_2005_review.md`
   - `mydocs/pr/archives/pr_2005_review_impl.md`
   - `mydocs/pr/assets/pr_2005_issue1999_p16_hwpx_review.png`
   - `mydocs/orders/20260706.md`
6. remote push
7. 최신 PR head 기준 CI 대기
8. CI 통과 및 작업지시자 승인 후 merge
9. merge 후 #1999 auto-close 확인 및 후속 코멘트
10. `devel` sync, 로컬/원격 작업 브랜치 정리

## merge 전 조건

- GitHub Actions 최신 head 기준 통과
- PR diff에 review 문서, 대표 visual asset, 오늘할일 포함
- 작업지시자 승인

## merge 후 후속 처리

- #1999 상태를 확인한다.
- auto-close 여부와 무관하게 #1999에 merge commit, 검증 결과, 대표 visual asset 링크를 포함한 후속 코멘트를 남긴다.
- PR #2005에도 검증 요약과 대표 visual asset 링크를 남긴다.
- `devel`을 `upstream/devel`로 fast-forward sync하고 작업 브랜치를 정리한다.

