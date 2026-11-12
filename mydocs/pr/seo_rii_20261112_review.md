# seo-rii 2026-11-12 PR 검토

## 현재 판정

| 원 PR | 범위 | 판정 |
| --- | --- | --- |
| #8952 | macOS Metal 두 타일 | 승인 — 사용자의 Mac 기여자 설명 수용 결정 적용, 실제 Apple SDK·실기 미실행 명시 |
| #8953 | 이동 참조앱 | 머지 보류 — Linux 이동 실행 통과, Windows 공통 헤더 경로 보정·재검증 필요 |
| #8954 | Windows 두 타일 qualification | 머지 보류 — Native/SDK 통과, 실제 runner C1083으로 화면 검증 미실행 |

[누적 수행 계획](seo_rii_20261112_review_impl.md) · [검증 원장](assets/seo-rii-20261112/README.md) · [Windows 보정 계획](seo_rii_20261112_windows_build_finding.md).

원 기여 head 3개와 최신 devel `a85d3265dd10b316d1a0ae029ca3f67245b38b77`를 보존한다. 원 CI는 모두 같은 head에서 33개 success·6개 정상 skip이며 과거 DNS 실패를 재실행하지 않았다. 로컬 실제 Flutter 280개·Python 186개·Metal policy 1,669개와 Linux/Windows Native 빌드·Windows SDK 4개 단위, Native/fresh WASM 전 7쪽 raw 100%를 확인했다. Linux 이동 참조앱의 실제 문서 열기·쪽 이동·닫기·정상 종료도 통과했다.

이번 사용자가 Mac 환경 부재를 이유로 확정한 수용 결정을 적용했다. 별개로 실제 Windows Flutter runner의 junction 헤더 검색 실패가 재현돼 한 파일 CMake 보정안을 준비하고 보정 담당 범위 승인을 요청했다. 응답 전까지 해당 제품 소스를 수정하지 않는다.

공개 댓글·승인 리뷰·source push·새 PR·merge는 수행하지 않았다. 원 PR 3건과 #536·#8701은 현재 OPEN을 유지하며 Draft #8962–#8967은 대상에서 제외했다.
