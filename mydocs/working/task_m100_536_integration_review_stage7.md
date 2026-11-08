# seo-rii 통합 검토 7단계 — 최신 devel 리베이스와 제출 형식

## 사용자 지시와 실제 base 변경

사용자의 후속 지시에 따라 TSV 원문을 PR 본문에 직접 넣을 수 있고 본문 크기 제한 시 asset 링크로
제출하도록 지침과 템플릿에 병기한다. 이번 최종 TSV 원본 32개는 다운로드·영구 보존도 가능하게
`mydocs/pr/assets/issue_536_seo_rii_20261017/` 아래에 바이트 그대로 커밋하고 PR 본문에서
정확한 head SHA의 raw 다운로드 링크로 연결한다. 후보는 `native/`, `wasm/`, 기준은
`base/native/`, `base/wasm/` 아래에 `<문서이름>_silhouette.tsv`로 보존한다.
실제 source/producer SHA·입력/Print·페이지·TSV SHA-256·바이트 수를 연결하고 push 뒤 다운로드한
원본을 재대조한다. 실행 로그·중간 JSON은 ignored output에 둔다.
본문 직접 삽입과 크기 제한 시 링크 제출의 두 방식·예시를 AGENTS·CLAUDE·CONTRIBUTING·Visual Sweep 가이드·시각 검증 governance·fixture
근거 문서 및 `.github/pull_request_template.md` 총 7곳에 일관되게 반영한다.

PR 직전 확인에서 devel이 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`에서
`b7a5388034ee66f58a7e1d5363974e83b6374f57`로 전진했다. 새 commit은 #7953이며 PDF 가져오기를 기본 feature로
활성화하고 parser·WASM·PDF backend와 일부 CI 경로를 변경했다. 원 PR 20개의 head는 그대로다.

- 리베이스 전 통합 head: `70ca10c208389dfa8f03c3a4f048c9551f8eaf51`.
- 리베이스 직후 통합 head: `a7aac1a9ce5b2b902471ecd8f234f7d72834812d`.
- 원 161개 commit은 순서·제목을 보존해 충돌 없이 재적용했다.
- 독립 `git merge-tree --write-tree <이전 head> <새 base>`의 tree와 리베이스 결과 tree가
  `f05c9543837ab60314293f0620bee1ee28d17548`로 같다. upstream PDF 변경과 기존 통합 결과를 모두 보존했다.
- `prepr-rebase-audit.json`에 161개 이전/현재 SHA mapping, `logs/prepr-rebase-range-diff.log`에
  범위 비교를 보존한다. 1–6단계의 옛 SHA는 당시 실제 빌드·실행의 역사적 증거로 유지한다.

## 새 head 검증 계획과 범위

기본 feature가 바뀌었으므로 이전 head의 10,643 PASS를 새 head의 결과로 재사용하지 않는다.
다음은 기본 작업공간의 공용 `target/pr-review`, Cargo 잡 8개를 사용해 순차 실행한다.

1. clean한 이 검토 worktree를 새 base에 임시 checkout하여 Native Skia CLI와 fresh host `--no-opt`
   WASM을 빌드하고 source·명령·해시를 별도 `rebased-base-*` 증거로 복사한다. 검토 branch로 돌아온다.
   사용자 dirty 상태인 기본 작업공간과 다른 worktree는 변경하지 않는다.
2. 통합 후보 Native CLI·fresh WASM을 같은 조건으로 빌드한다. HWP/HWPX 8개 전체 페이지를
   새 base와 대조하여 양쪽 90% 이상·0.01%p 이상 하락 없음·쪽수 일치와 직접 판독을 확인한다.
   이전 base의 TSV 32개는 역사적 원본으로 보존하며 새 base/후보 TSV와 구분한다.
3. suite/unit-tier 정책, 원 변경 Native 집중과 기존 HFT·새 PDF paint 경계, 전체 release-test
   nextest, 공식 Native Skia lib·missing-picture·Direct PDF를 확인한다. 전역 feature 변화로 영향을
   받는 현재 전체 목록을 실행하며 이전 실행 수를 새 기대값으로 고정하지 않는다.
4. format·Native/WASM/workspace all-target Clippy·workspace build를 새 head에서 통과시킨다.
   새 fresh WASM으로 Studio TypeScript·unit·build·renderer/실제 browser 계약을 확인한다.
   기존 소스의 release LTO 실행은 6단계 역사적 결과이며 새 head 실행으로 표시하지 않는다.
5. 필요한 실제 ABI 소비·대표 PNG를 갱신하고 현재 source/binary/PDF/TSV/PNG SHA를 기록한다.
   완료 후 개별 판정을 갱신하고 원격 branch는 이전 remote SHA를 lease로 확인해 갱신한다.
   TSV 원문을 포함한 PR 생성 → code head CI → 같은 PR review/오늘할일 trailing → trailing CI →
   정상 merge·post_merge 순서를 유지한다. 부모 #536 전체를 종료하지 않는다.

이 단계의 전체 회귀는 통과했다. 아래 추가 devel 전진에 따라 최신 base의 검증은 다음 단계로 이어간다. 6단계 통과와 구분한다.
검증 로그는 ignored `output/pr-review/seo-rii-20261017/logs/rebased-*.log`에 기록한다.

## 리베이스 후 확인한 실제 결과

실제 producer는 `751e34e344d371d276917dccb06b93fa8e79eff7`, 기준은 위 `b7a538803`이다.
전체 release-test nextest는 **10,770 PASS / 0 FAIL / 45 skipped**, 실행 729.865초이며,
컴파일을 포함한 명령 전체는 1,491.2초였다. 원 변경 Native 집중은 **280 PASS**, 공식 Native Skia
lib는 4개 binary 합계 **4,043 PASS / 12 ignored**다. missing-picture·Direct PDF와 새 PDF paint,
feature-on FFI Clippy·빌드 및 실제 ABI 9종, format·Native Clippy도 모두 exit 0으로 확인했다.
WASM Clippy도 exit0이다. 새 base 전진 후 아직 진행 중이던 이전 base workspace build와 그 뒤의 lint·doc·feature-off·fresh Studio 검증은 중복 실행을 피하기 위해 최신 base 단계로 넘겼다. 미완료 검사를 통과로 표시하지 않는다.

8개 입력의 전체 1쪽씩 Native/fresh WASM16개 비교는 모두90% 이상·쪽수1:1·누락0이며,
동일 backend 기준 devel 대비0.01%p 이상 하락은 없다. 이번32개 TSV에서는 raw와 색상 경계
보정값이 모두 같다. WASM lseg-02-mixed의 새 base와 후보는 둘 다99.84053%이며 과거
6단계의99.84066%를 새 측정값으로 재사용하지 않는다. fresh review16개와 수식 동일 좌표4배
확대를 직접 판독했다. 수식 이름 크기는 개선됐고 기존 분자·괄호 간격은 남아 있다. 폼 Edit의
기존 bevel·그림의 raster/테두리 차이도 해결 성과로 표시하지 않는다.

실제 Native API IME 조합·undo/redo·저장·재열기는 같은7337바이트 입력을 재생성했고, live/reopen
794×1123 RGBA의 SHA-256은 모두 `51c6088a280c146a80b7ca91664f7426a662dd422b4dce6b069594ad7b341722`로 같다.
독립 Print 대비 screen100%, 실제 C ABI Direct PDF의 shape-group-02도100%이며 vector
`ShadingType 2`·Image 객체 부재를 확인했다. 두 새 review도 직접 판독했다. representative PNG20개는
이 새 출력으로 교체하고 source·파일 해시를 `rebased-visual-asset-plan.json`에 기록했다.

원 로그·명령·producer/binary hash는 ignored `rebased-rust-gates.json`, `rebased-builds.json`,
`rebased-visual-audit.json`, `rebased-direct-reading.json`, `rebased-native-consumers.json`에 있다.
최종 TSV 원본32개와 README를 검토 asset에 보존하고 이 단계 commit에 포함한다. source751/b7의 원본이며 새 base의 TSV로 표시하지 않는다.

## 추가 devel 전진 확인

위 검증 중 `upstream/devel`이 #7964 merge인 `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`로
전진했다. 그림 색조·WMF crop·인쇄 용지·표/도형 배치·Direct PDF와 Visual Sweep raw 판정이
바뀌었다. 사전 merge-tree는 `src/renderer/skia/image_conv.rs` 충돌을 검출했다. 현재 실행 결과는
위751/b7 조합의 증거로 보존하며 새 base의 통합 검증으로 표시하지 않는다. 완료된 회귀·lint 결과를 commit하고 미완료 검증은 최신 base에서 수행한다. obsolete base의 workspace build만 해당 driver·자식 PID를 확인해 종료했으며, 전체 회귀를 중단한 것은 아니다. 최신 base로 rebase한다. 캐시·색조·sampling 소비를 함께 보존하는 해소와
raw 기준 재비교·새 head 검증은 다음 단계에서 기록한다.
