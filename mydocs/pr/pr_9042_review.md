# PR #9042 리뷰 — 같은 문단의 제목·본문 표 배치

## 최종 판정

**머지 보류 — 독립 검증 진행 중.** 원 PR의 개선·반례·필수 증적과 CodeQL neutral 사유를 조사한다.
기여자의 통과 주장과 메인테이너의 실제 검증은 구분한다. 원격 댓글·push·승인 리뷰·merge는 아직 승인받지 않았다.

## 접수와 범위

- 원 PR [#9042](https://github.com/edwardkim/rhwp/pull/9042), 작성자 puckqook의 첫 외부 기여다. base devel, 원 head `3ae4eeb332c3b0edbb3a8fbe334daf2b0a48115b`, fork `puckqook/rhwp:coanchored-table-pagination`.
- reviewer edwardkim을 지정하고 API users와 고정 head를 재확인했다. 최초 사전 조회/fetch가 지정 전에 실행된 순서 누락을 기록하며 지정 뒤 같은 SHA를 다시 고정했다. 원격 source는 변경하지 않았다.
- 검토 기준은 최신 upstream/devel `00d09f773a2f6c21604a4e6394df7152bbb26e8c`, 주 작업공간 branch `review/pr-9042-coanchored-table-pagination`다. clean한 주 작업공간을 사용하고 별도 worktree는 만들지 않는다.
- 기본 maintainer_general, 보조 intake_and_review·local_validation·visual_fixture_evidence·post_merge·first_time_contributor를 적용한다. 첫 기여자의 원 head 보정이 필요한 경우 collaborator_external_pr의 원 기여·보정 분리 조건도 따른다.
- 원 head 제품 CI는 success, CodeQL 종합은 neutral이다. 실제 경고와 분석 configuration 누락을 분리 확인한다. 직접 연결한 해결 이슈는 없으며 선행 공동 앵커·마지막 행 공통 로직의 이력을 먼저 추적한다.

## 검증 계획

1. 입력·첨부 Mac Viewer Print PDF의 commit/bytes/SHA·생성 방식·저장 줄 표지를 확인한다. 첨부본을 실제 열고 현재 devel에서 원 증상을 재현한다. 재사용 가능한 독립 PDF를 버전만으로 재변환하지 않는다.
2. 원 기여 `3ae4eeb332c3b0edbb3a8fbe334daf2b0a48115b`를 같은 주 branch에서 적용한다. 원 ref는 보존하고 실제 적용 SHA·충돌을 기록한다. 충돌 보정이 필요하면 구체적인 방침을 정리한다.
3. 새 공통 frame의 생산 → fit → 배치/paint 소비와 후속 덮어쓰기, terminal 행 빈 높이 회수의 실제 호출 경로를 추적한다. 실제 한컴 저장 정보와 RHWP 합성 입력의 계약을 구분한다.
4. 원 사례와 적용 경계·반례를 작게 실행해 유지·개선과 일반성을 확인한다. 기존 페이지 항목이 있는 경우, 텍스트/표 순서, 저장 줄·합성 줄, 중첩 표 tail의 주장 범위를 독립 근거와 대조한다. 새 회귀 파일 첫머리의 측정 근거 누락도 확인한다.
5. 수용 후보이면 관련 전쪽 Native/fresh 최적화 WASM TSV와 같은 영역 직접 판독, 편람 383쪽 gate·정본 보호 TSV·focused 및 local_validation 4.3의 필수 검사를 완료한다. 실제 결함이나 부족한 근거는 먼저 구체적으로 기록하고 보정 담당/공개 조치는 별도 승인 범위로 구분한다.

검증 입력은 원 PR의 추적 경로 `mydocs/pr/assets/coanchored-table-pagination/float-pagination.hwp`와
`pdf/coanchored-table-pagination/float-pagination-hancom-mac-viewer-12.31.9-print.pdf`를 그대로 사용한다.
원 HWP SHA-256 `a6c335583559f7e3057cf883c75f5c66942bb7a213da0e46796116a77cfc3ec8`,
PDF SHA-256 `f1ec7d39a06824b1407263ef872d4f05cfba57a0f335e5785fe4305b06309206`은 실제 bytes와 재대조한다.

## Merge 후 contributor PR comment 계획

수용 검증을 완료하고 사용자 원격 승인을 받으면 첫 기여 환영·감사, 원 기여와 추가 보정, exact head CI와
독립 Print·전쪽 TSV·같은 쪽 inline 이미지를 존댓말로 준비한다. 현재는 문안 게시·merge를 수행하지 않는다.
