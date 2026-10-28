# PR #7603 검토 — v0.8.7 릴리스 준비

## 현재 판정

로컬 준비 검증을 통과했고 사용자 승인에 따라 [Open PR #7603](https://github.com/edwardkim/rhwp/pull/7603)을 등록했다. 최종 제출 head의 원격 CI 완료와 별도 병합 판단은 남아 있다. 릴리스 태그·공개 배포·스토어 제출 완료로 표현하지 않는다.

## 접수와 검토 경로

- 작성자 edwardkim, base devel `17a69fa4e789226d529ef90caaae5a58655a1eec`, 비교 main `680111ec7bea2fe11110de18c3676ba5a1cf7847`.
- 패키지 source `942f856c19f65093736f4ee9a7a8ee8d24e2227e`, 최초 게시 head `e556244b85a4ac33d7f096262027235ad7691364`. 이후 검토·오늘할일·계획 변경은 문서뿐이다.
- collaborator_self_merge 기본 경로와 intake_and_review, local_validation, rework_and_exceptions를 적용했다. 본인 PR에 reviewer를 assign하지 않는다.
- 최초 API 조회: Open·draft false·41파일·4,895줄 추가/115줄 삭제. 대형 PR 경로를 적용한다. 3,971줄의 기여자 ledger가 대부분이며 코드·운영 계약 검토와 로컬 simulation을 마쳤다. 원격 CI·사용자 병합 판단을 별도 단계로 둔다.
- 게시한 한국어 본문이 UTF-8 임시 파일과 정확히 같고 선두 BOM·`??` 치환이 없음을 API로 확인했다.

## 구현 주장과 검증

[릴리스 준비 기록](../../plans/release_0_8_7_20261006.md)에 source SHA, 명령·결과, 패키지 해시, 원시 로그, 적용되지 않은 범위를 연결했다. 버전 14파일과 배포 문서, contributor provenance, workflow promotion 계약이 변경 범위다. Rust production·Studio rendering source·fixture·golden·허용치는 devel과 같다.

collector는 같은 caller run에 여러 workflow source가 연결될 때 마지막 hash만 남기던 dict 덮어쓰기를 제거했다. 각 source hash·mode·verdict 보존 검사가 수정 전 `1 != 2`로 실패하고 수정 후 통과했다. exact SHA·actor·필수 job·artifact·pagination 검사는 유지한다. metadata adapter는 실제 shipped script의 모의 API 계약을 검사하며 저장소 metadata를 쓰지 않는다. 실제 원격 label/metrics 쓰기는 미검증이다. Nextest reusable 정책은 실제 CI의 네 archive build/worker job을 요구한다.

Python release/promotion 116개·CI wiring 44개, Node metadata 384개, Studio 1,815개(2 skip)·editor 32개·tsc, 폰트/라이선스 6개, 변경 workflow actionlint를 통과했다. locked fresh WASM 0.8.7과 Studio public 동기화·native version 및 WMF 회귀 2개, CDP Chrome 편집 49개, 최종 Chrome smoke/download/lifecycle, Firefox 다운로드·편집·저장, npm dry-pack와 VSIX 생성도 완료했다. AMO source ZIP만으로 재빌드한 Firefox 78파일이 최종 후보와 모두 동일하다.

이전 보안 통합의 전체 nextest 10,508 PASS와 Native Skia 4,109 PASS는 해당 source의 검증이다. 이 준비 head에서 새로 전수 실행했다고 주장하지 않는다. 기존 main WMF 실패의 debug/ASAN·fuzz 대조는 별도 보안 기록의 진단이며 원시 비공개 입력을 이 PR에 공개하지 않는다. 이번 준비 PR에서 WMF source나 테스트를 변경하지 않았다.

## 조판·입력 증적 적용 여부

조판 원칙 검토는 **비해당**이다. 줄 소속·높이·좌표·측정/배치·pagination/continuation·렌더링 baseline 변경이 없다. 새로운 한컴 PDF 일치나 Visual Sweep 통과를 주장하지 않는다. 위 브라우저 검사는 편집·다운로드·패키지 계약 검증이다. 배포 산출물 및 source ZIP의 정확한 hash는 준비 기록에 남겼다. 신규 파일 기반 조판 회귀나 기준 PDF 수용은 없으며, 코드 생성 단위/편집 검사와 기존 정식 WMF 검사를 실행했다. 비공개 WMF crash 재현 입력은 별도 진단으로 구분한다.

## 병합 전 조건과 남은 절차

main 이력을 포함한 2-parent merge `7db4df1b4fd6f591a494da57502fcfe307db922f`가 필요하다. **이 PR은 merge commit으로 통합한다.** squash/rebase는 main ancestry 조건을 잃는다. 최초 게시 직전 최신 devel과의 merge-tree `23baffea0b84ae115a6d669a2b0a91a410e2bab3`가 head tree와 같았고 main ancestor를 확인했다. 병합 직전 최신 base/head·충돌·exact-head CI를 다시 확인한다.

PR 생성 head의 CI가 자동으로 시작됐으며 첫 조회에서 preflight 일부 성공, 다른 job은 실행/대기 중이었다. 이 관측은 최종 trailing head의 CI 성공을 대신하지 않는다. 문서 trailing push 뒤 정확한 head로 재조회한다. workflow 설정을 변경하거나 수동 dispatch하지 않았다.

준비 PR 통합 후 exact devel SHA에서 승인된 수동 preflight 11개를 수행하고 macOS 포함 CLI 5-platform 산출을 검증해야 한다. 이 증거가 완료돼야 devel→main PR과 이후 태그 CD를 진행한다. 실제 Edge·스토어 설치/업그레이드·macOS 실기기, 공개 advisory/CVE 종료는 로컬 기록으로 대체하지 않는다.
