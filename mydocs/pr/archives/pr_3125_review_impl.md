# PR #3125 통합 적용 기록 — resumable pagination

## 적용 순서와 보존 원칙

| 단계 | 내용 |
| --- | --- |
| 기준 | `upstream/devel@1b5950a95` |
| 가시성 브랜치 | `integrate/postmelee-20260724` |
| 적용 순서 | #3125 (1/3), 이후 #3130, #3136 |
| 제외 | 원 PR 내부 devel merge `faa28a8`, `d277f974` |
| 저자 보존 | 기능/문서 커밋은 `git cherry-pick -x`로 원 작성자와 원 SHA 추적을 유지 |

적용 SHA는 `75f412f, 7ca01ee, 4ba2262, 59def61, 7338ad5, c4533cd, 46afd83,
7fc36e9, 9263cf9, 1e10d9e, 063cdd3, 04bdf0d`다.

## 메인터너 조정

- `mydocs/orders/20260722.md` add/add 충돌에서 #2424 완료 기록과 기존 #2431 기록을 모두 남겼다.
- 이후 #3130이 같은 rendering/pagination 접점을 변경했을 때 deferred job의 revision과 continuation
  수명은 #3125 의미를 유지하도록 최종 tree에서 함께 검증했다.

## 다음 단계

1. 이 review 문서·검증 asset·오늘할일·가시성 브랜치 절차 문서를 통합 PR에 포함한다.
2. 원격 통합 브랜치 push 및 PR 생성 뒤 최신 head CI만 모니터링한다. 이 운영 기록 추가 때문에
   로컬 전체 cargo suite를 다시 실행하지 않는다.
3. merge 승인 뒤 통합 PR을 merge하고 #2424의 open 상태를 확인한다.
4. 원 PR 감사/통합 안내 및 close 여부는 별도 승인 뒤 수행한다. merge 전에는 원 PR을 변경하지 않는다.
