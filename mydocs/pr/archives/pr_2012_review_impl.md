# PR #2012 처리 계획

## 커밋 구성

- `7a17c9f53 task 2010: Studio 자동저장 간격 설정 추가`
  - 사용자 설정과 환경설정 UI에 복구용/idle 자동저장 간격을 추가했다.
  - `AutosaveManager`를 idle timer와 recovery interval timer 중심으로 재구성했다.
  - 자동저장 상태를 Studio 상태창에 노출했다.
- `ec7abf90a task 2010: 대형 표 입력 랙 완화`
  - 표 셀 단순 텍스트 입력용 page-local Rust/WASM command를 추가했다.
  - Studio command/input 경로에서 page-local fast path를 우선 사용하도록 했다.
  - table bbox hover cache를 page hint 기반으로 정리했다.
- `7a4e2cebb task 2010: 대형 문서 지연 페이지네이션 조정`
  - 30쪽 초과 대형 문서의 idle pagination flush를 자동 실행하지 않도록 조정했다.
  - 저장, 다른 이름 저장, 인쇄 전 pending pagination flush 경로를 추가했다.
- 후속 문서 커밋
  - 옵션 1 기록: `pr_2012_review.md`, `pr_2012_review_impl.md`, 오늘할일 PR 번호 반영

## 옵션 1 처리 방침

이 PR은 collaborator self-merge 후보 경로로 처리한다.

- review 문서: `mydocs/pr/archives/pr_2012_review.md`
- 구현/처리 계획: `mydocs/pr/archives/pr_2012_review_impl.md`
- 오늘할일: `mydocs/orders/20260707.md`
- 시각 검증 asset: 해당 없음. 성능/상호작용 계측이 수용 기준이다.
- 별도 docs-only 후속 PR은 만들지 않는다.

## 검증

로컬 검증은 각 stage 문서와 review 문서에 기록했다.

- Studio unit/build 검증 완료
- WASM build 완료
- 단위 Rust 테스트 완료
- 115쪽 샘플 실제 Chrome 입력 계측 완료

CI는 PR head 최신 커밋 기준으로 재확인한다.

## merge 후 후속 처리

1. PR #2012 merge SHA 확인
2. `devel`을 `upstream/devel`로 fast-forward sync
3. #2010 auto-close 여부를 시간을 두고 확인
4. auto-close 여부와 무관하게 #2010에 검증/merge 후속 코멘트 작성
5. PR #2012 브랜치 `task_m100_2010_autosave_interval` 로컬/원격 정리
