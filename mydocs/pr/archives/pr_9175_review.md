# PR #9175 리뷰 — v0.9.1 승격을 위한 main 계보 동기화

## 최종 판정

**승인.** 병합 후보 `7d984cd776d69995da996fb0590a79bf2d4bb3a6`은 v0.9.1 준비본의 제품·테스트·workflow 파일을 그대로 보존하고 main의 보안 백포트 계보와 검증 기록을 연결한다. 리뷰 기록을 포함한 최신 head의 필수 검사와 병합 tree를 다시 확인한 뒤 merge commit으로 수용한다.

## 검토와 실행 결과

- [#9175](https://github.com/edwardkim/rhwp/pull/9175), `edwardkim`의 collaborator self-review. 현재 devel은 `d50dacc348d9533661378f40339b62381f0dd354`, main은 `7213ed1f96266b95e22cd5645a07851c9c18298e`다.
- [#8884](https://github.com/edwardkim/rhwp/pull/8884)의 두 브랜치 보안 보정은 같은 stable patch ID다. 네 충돌에서 최신 devel의 qcms NEON 조건, ToUnicode 설명과 최종 보안 기록을 보존했다. `mydocs/` 밖 변경이 0개임과 양쪽 base의 병합 tree 일치를 확인했다. 조판 규칙·테스트 입력에는 영향이 없고 로컬 승격·릴리즈 계약 61건과 문서 링크가 통과했다. [준비 보고서](../../report/release_0_9_1_preparation_report.md)
- [CI](https://github.com/edwardkim/rhwp/actions/runs/44268890513)에서 `all-review-only-no-code-impact` 판정과 `Build & Test` 성공을 확인했다. [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44268890359), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44268890229), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44268890352)의 preflight와 [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/44268955505)가 성공했다. 정책에서 생략한 코드 검사를 새 실행으로 집계하지 않는다.

병합 후 그 devel SHA로 승인된 승격 workflow 12개를 검증한다. #9154는 main 적용과 정기 실행을 확인할 때까지 열린 상태로 둔다.
