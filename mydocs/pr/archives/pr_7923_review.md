# PR #7923 리뷰 — planet6897 PR 18건 통합·폰트 배포 계약과 시각 회귀 보강

## 최종 판정

**승인 — 명시한 통합 범위의 로컬 검증과 code candidate CI 완료.**
code candidate `cbc5815631d6c221dc6ec6e0f03565859cb1c346`의 전체 GitHub Actions·정책 체크 성공과 MERGEABLE/CLEAN을 확인했다. 사용자 승인으로 현재 PR에 개별 archive review·오늘할일을 함께 포함했다. 문서-only trailing head의 최신 체크 성공과 실제 merge·후속 처리가 남아 있다.

## #7926 추가 정책 보정

사용자가 [fast-pass 판정 버그 #7926](https://github.com/edwardkim/rhwp/issues/7926) 등록·개선을 요청했다. 고정된 workflow 실행 SHA와 가변 연결 PR head를 구별하도록 helper를 보완했고, 새5회귀 수정 전 FAIL/후 PASS 및 Node354·Python186PASS와 실제 API 재생 통과를 확인했다. [단계 기록](../../working/task_m100_7926_stage1.md)을 따른다. 앞서 기록한 `cbc581563` CI는 이 추가 수정 전 결과다. 새 정책 수정 head의 전체 CI 성공을 다시 확인하기 전에는 병합하지 않는다. renderer/Rust/폰트 바이트는 그대로여서 시각·로컬 Rust 검증을 반복하지 않았다.

## 접수·적용 범위

- [PR #7923](https://github.com/edwardkim/rhwp/pull/7923), 작성자 jangster77, base devel `463c48cc228170b576c832591b76826116e86a49`.
- 기본 경로: collaborator self-merge. 보조: intake/local validation, multi-PR, visual fixture, review-only fast-pass, post-merge.
- 18개 source head·실제 소비 경로·조판 원칙·수용/이관 범위는 [통합 기록](planet6897_followup_20261015_review.md)과 그 표의 개별18개 review를 따른다.
- 최종 로컬 Rust/lint/Skia code 검증 `39a634e547621c32ae6f9ba8c1694d35c827078b` 이후에는 검토 자료, 사용자 요청 PDF 제외, Node mock alias2곳과 PDF 보관 지침을 변경했다. 렌더러·Rust·배포 폰트 바이트와 검증 입력의 동일성은 유지한다.

## 검증 결과와 독립 근거

- [CI](https://github.com/edwardkim/rhwp/actions/runs/38734213321)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38734213350)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38734213214)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/38734213427)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/38734213384) 모두 성공. native/WASM/workspace lint, Skia, frontend package, 네 default-feature archive worker와 Build & Test 및 exact-head 정책 체크를 확인했다.
- CI 집계 로그 `archive_total=10162 shard_total=10162`. CI archive 수, 로컬 최초 전체 실행, 사용자 승인한 집중 재검증 합산을 구별하며 새 로컬 전체 실행을 했다고 쓰지 않는다.
- 로컬 최초10,354PASS/8FAIL/45SKIP에서 실물6건은 사용자 지정 이관, 폰트 계약2건 보정 후 경로 관련55PASS로 전체 회귀 통과 처리했다. Studio1,875PASS, 네 배포 빌드·폰트 계약, Native Skia4,041PASS/12ignored와 추가6건·doctest8PASS/3ignored를 확인했다.
- Native/fresh WASM 각각21문서181쪽 전쪽90% 이상, 독립 Print 쪽수 일치와 직접 확대 판독 완료. 핵심 ±2px 고정 쪽은 양쪽98% 이상이며 소유·순서·보존·포함 관계도 검사한다. 사용자 승인 소폭 하락2쪽은 통합 기록에 수치와 함께 명시했다.
- [입력·Print·PNG SHA 및 범위](../assets/planet6897_followup_20261011/qualified-print-d80593/README.md). 전쪽42개 TSV 원본은 PR 본문에 보존했다.
- Render Diff의 옛한글 mock alias 이름 불일치를 수정했다. 기존 긍정/부정 assertion을 유지했고 로컬12단계·원격 전체 Render Diff가 통과했다. Visual Sweep 제품 출력 변경은 없다.
- sample10 대용량 PDF는 현재 head·PR 최종 diff에서 제외했다. 50MiB 이상 PDF는 로컬에 보관하며 source/PDF SHA·쪽수·생성/검증 명령·전쪽 TSV·대표 PNG를 남기는 보존 정책을 정리했다.

## 이슈·병합 뒤 처리 계획

[#7445](https://github.com/edwardkim/rhwp/issues/7445), [#7774](https://github.com/edwardkim/rhwp/issues/7774), [#7922](https://github.com/edwardkim/rhwp/issues/7922)의 미해결 문서 피델리티는 OPEN 유지한다. 정상 parser/serializer/API 보호는 이관한 동일 바이트 경로에서 유지했다. 이관을 새 base90% 미달 입증이나 결함 해결로 기록하지 않는다.

## Merge 후 contributor PR comment 계획

실제 merge SHA와 devel의 archive·PNG·이슈 assets를 확인한 뒤 개별18개 review에 명시한 범위/페이지/수치로 원 PR에 한국어 존댓말 comment를 게시한다. Visual Sweep 정본을 direct link로 연결하고 해당 merge SHA 고정 raw PNG를 표시한다. source head를 재조회해 통합 범위가 같은 열린17개 원 PR만 supersede close하고 이미 닫힌 #7669는 상태를 유지한다. 기여자 fork branch는 보존한다.

후속 기록 PR 없이 현재 PR의 기록을 사용한다. merge SHA·duration 메타데이터 갱신 결과 → 문서/asset 존재 → devel fast-forward → 이슈/원 PR 안내·supersede → 이번 검토 전용 branch/worktree 정리를 실행한다. 공유 target/pr-review와 기본 작업공간·다른 작업의 자료를 보존한다. 원시 로그/임시 산출물은 필요한 영구 증적을 확인한 뒤 canonical 정리 절차를 따른다.
