# PR #8953 검토 — feat(flutter): build and launch relocatable reference apps (P118)

## 최종 판정

**머지 보류 — 검증 진행 중.** 원 head의 현재 CI 성공을 확인했고 실제 변경 범위의 로컬 검토·플랫폼 검증을 진행한다. macOS 실기 환경 부재에는 이번 사용자의 기여자 주장 수용 결정을 적용한다. GitHub 승인·댓글·push·merge는 아직 수행하지 않았다.

## 접수와 고정 기준

- 작성자 seo-rii / base devel / Ready / reviewer edwardkim 배정 및 API 확인.
- 원 head `9cfe2754d92a720135445aed285ff91c0c75dc7e`, fork `seo-rii/rhwp`, branch `task/8701-p118-reference-app-bundle`.
- 기준 devel `a85d3265dd10b316d1a0ae029ca3f67245b38b77`, 개별 병합 simulation exit 0.
- 실제 [CI run 43053940397](https://github.com/edwardkim/rhwp/actions/runs/43053940397)의 head 일치·attempt 2 success를 확인했다. 본문에 남은 attempt 1 DNS 실패와 현재 상태를 구분한다.
- [누적 검토·검증 계획](seo_rii_20261112_review_impl.md), [Mac 수용 결정](assets/seo-rii-20261112/macos-maintainer-decision.json).

## 검토·검증 상태

현재는 실행 전 접수 기록이다. 실제 코드·선행 계약·검사 결과와 source/hash를 확인한 뒤 고유 범위·조판 영향·입력 커밋·직접 화면 판독·최종 판정을 보완한다. 원 기여 CI를 maintainer가 직접 수행한 플랫폼 검증으로 표현하지 않는다. #536·#8701 전체 로드맵은 OPEN을 유지한다.
