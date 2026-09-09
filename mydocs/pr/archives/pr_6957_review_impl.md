# PR #6957 통합 및 후속 처리 계획

## 대상과 판정

- [통합 PR #6957](https://github.com/edwardkim/rhwp/pull/6957), 작성자 `jangster77`의 self PR.
- 판정: **메인터너 보정 후 수용 가능**. 근거와 잔여는 [self-review](pr_6957_review.md)를 따른다.
- 상세 보정·실패 이력·검증 명령·source/binary/asset 해시는
  [원 PR 통합 기록](../pr_6949_6952_review_impl.md)에 보존한다.
- 이번 지시는 동기화·무충돌 시 추가 테스트 생략·문서 갱신·push·PR 생성까지다. merge/close/comment 및
  원격 브랜치 삭제는 수행하지 않는다.

## 확정 커밋

| 단계 | SHA | 제목/역할 |
| --- | --- | --- |
| 최신 base | `d8e4ab727b70b6abfcf11766134e09a9a9bfc982` | devel, #6773 통합 |
| #6949 | `f39e79dd8098f105cdb1e22fcb67a7e18b2e74b0` | 수정(wmf): ROP 관용구의 1비트 마스크를 보이는 칠로 내보내지 않는다 (#6865) |
| #6952 | `a238c11141b546e861c18e3e12354bf7476406e8` | 수정(hwpx): 각주 번호 서식 네 필드를 왕복에서 지킨다 (#6872) |
| 최초 review | `0451d49f6a2a289d1bff25cd5124f747c8a88c02` | 문서: PR #6949/#6952 체리픽 검토와 보류 사유 기록 |
| 보정·증적 | `fe92170a0cac3e21aca3281b7e4a054052dadf5b` | 수정: WMF 하프톤 경계와 각주 왕복 회귀를 메인터너 보정 |
| 판정 갱신 | `bffaab24f16fd7341d15661c4905931f5ca0ec95` | 문서: 검증 완료된 메인터너 보정의 PR 최종 판정 갱신 |
| 동기화 기록 | `0bedf37bd3d8cde7c46f71ba4c68322391f9e0f8` | 문서: devel 리베이스 후 보정 SHA와 증적 해시 갱신 |

이 문서와 self-review·오늘할일을 추가하는 후속 commit의 SHA는 자기 참조로 예측하지 않는다.
최신 PR head와 Git 이력에서 식별하며 위 소스 후보를 변경하지 않는 문서 전용 commit이다.

## 단계별 상태

1. 원 PR 체리픽·메인터너 보정·전체 로컬 검증·대표 증적 보존: 완료.
2. upstream/devel fetch 및 리베이스: 완료, 오늘할일 포함 충돌 0건. 다른 작업의 오늘 기록 보존.
3. 변경 소스·테스트 7개와 PDF 2개/PNG 5개 해시 대조: 완료, 리베이스 전후 동일.
4. 작업지시자 지시로 리베이스 후 추가 테스트 생략. 최신 base 전체 재검증 완료로 표현하지 않는다.
5. upstream 작업 브랜치 push 및 devel 대상 Open PR #6957 생성: 완료. reviewer 지정 없음.
6. 채번 기반 archive self-review·이 문서·오늘할일을 후속 문서 commit으로 포함: 이번 단계.
7. 최신 head GitHub CI·작업지시자 시각 및 merge 승인: 대기. 이전 head CI를 최신 결과로 쓰지 않는다.
8. 승인 후 실제 merge SHA와 asset devel 포함을 확인하고 source PR/이슈별 comment·close 범위를 결정한다.
9. 승인된 후속 처리 범위에서 문서·오늘할일 갱신과 devel sync 및 브랜치 정리를 진행한다.

## 위험·중단 및 변경 되돌림 경계

WMF의 잔여 시각 차이는 #6865 전체 해결과 분리한다. CI가 실패하면 해당 최신 head의 원인을 먼저 보고하고
필요한 보정·재검증 범위를 결정한다. 1,000줄 초과 PR이라는 이유로 검토를 생략하거나 admin merge하지 않는다.
실제 merge 전에 통합 취소가 필요하면 PR을 닫는 별도 승인을 받는다. merge 후 되돌림은 확정 merge SHA를
대상으로 별도 revert PR로 수행하며 contributor 원격 브랜치·사용자 변경·공유 target을 임의 삭제하지 않는다.
