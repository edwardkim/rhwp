# PR #8983 리뷰 — HTML 첫 서식 앞 평문

## 최종 판정

**머지 보류 — 필수 시각 증적·정식 회귀 보완이 필요하다.** 원 주장은 최신 devel에서도 재현됐고 코드 검토상 수정 방향은 해당 원인과 연결된다. 이를 구현 거부나 새 회귀 검출과 혼동하지 않는다. 원 head의 공개 시각 증거가 없으며, 검사 파일의 첫머리에도 정식 측정 근거가 없다.

## 접수·검토 경로

- 원 [PR #8983](https://github.com/edwardkim/rhwp/pull/8983), 관련 [이슈 #8943](https://github.com/edwardkim/rhwp/issues/8943), 첫 기여자 `Bongseop-Kim`, draft 아님.
- 고정 원 head `9d1f6248c4ffd9365c9a0aab79d949fabde5c567`, fork `Bongseop-Kim/rhwp`, branch `fix/paste-html-leading-plain-run`. reviewer `edwardkim` 지정 후 API로 확인했다.
- 최신 base `80b1c354c29b7a02fb2b3c14a274da7f5b653ada`, 공통 로컬 누적 코드 `cc6103f1a82149f2d3ae1b49d86a81698df1841a`. 원 head 두 개를 그대로 ancestor로 보존한 clean merge이며 메인터너 코드 보정은 아직 없다.
- 기본 `maintainer_general.md`; 보조 `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`, `visual_fixture_evidence.md`, `first_time_contributor.md`, `post_merge.md`를 적용한다. 각 원 PR의 판정·원격 조치는 분리한다.
- 기본 작업공간과 다른 worktree는 보존한다. Cargo target은 `/home/edward/mygithub/rhwp/target/pr-review`를 공용으로 순차 사용한다. 원격 reviewer 지정 외 댓글·approve·push·merge·close는 수행하지 않았다.

## 현재 CI와 근거의 경계

원 code head의 Build & Test, Rust CodeQL 분석, Adapter·Proptest 및 정책 status는 success다. CodeQL 종합 check는 non-Rust 구성 세 개 미발견으로 neutral이며 성공으로 바꾸어 기록하지 않는다. 원 head CI를 로컬 누적 코드의 새 CI로 표시하지 않는다.

원 PR은 실제 출력에 영향을 주는 변경을 시각 비해당으로 적었다. BorderFill·CharShapeRef를 실제 paint가 소비하므로 Native/fresh WASM·독립 한컴 Print·전쪽 TSV·직접 판독이 필요하다. 원본·PDF가 없다는 사실을 비해당 근거로 쓰지 않는다. 독립 자료 확보 전에는 새 렌더링 회귀의 측정 header나 기대값을 임의로 만들지 않는다.

## 실제 재현·검증

- 원 PR의 4개/3개 검사와 기존 #7516의 8개를 고정한 최신 base의 공개 DocumentCore API에 같은 소스 그대로 연결했다. base는 11 PASS·4 FAIL이며 두 원 버그를 재현했다.
- 로컬 누적 코드 `cc6103f1a82149f2d3ae1b49d86a81698df1841a`에서 같은 15개는 모두 PASS다. 원 두 버그를 해결하며 기존 #7516의 8개도 보존했다.
- 표·서식·두 기능 혼합의 세 입력에서 HWP/HWPX 각각 저장 → 재열기 → 재저장 → 재열기, 합계 12개 검사 지점을 모두 통과했다. 본문·칸·테두리 종류·글자별 서식 보존을 비교했다. 이 임시 진단은 기존 정식 회귀의 1회 HWPX 저장 보호 범위를 자동 확대하지 않으며, 새 시각 일치 증거도 아니다.
- 원 PR 테스트 7개는 Git blob을 그대로 가져와 진단 crate에 연결했고 기존 #7516의 8개도 현재 소스를 그대로 사용했다. 실제 package·source·명령·exit를 구분했으며 이 재현을 전체 suite 실행으로 표현하지 않는다.
- 실행 원시 자료는 Git에서 제외된 `output/pr-review/bongseop-kim-20261112/`에만 둔다. 실행 로그·중간 JSON은 커밋하지 않는다.

## 해제 조건과 다음 범위

[공통 보완 계획](bongseop_kim_20261112_review_plan.md)의 원칙에 따라 원 기여를 보존하고, 필요한 보정·정식 회귀·독립 Print·Native/fresh WASM·반복 저장 검증을 끝낸 뒤 다시 판정한다. 보정 담당 또는 공개 댓글은 작업지시자 결정 전 실행하지 않는다.

## 적용 경계

새 코드의 첫 0 위치 구간은 기존 CharShapeRef 조회의 앞 글자 fallback을 해결한다. 공개 빈 문서의 첫/둘째 문단·목록·서식 뒤 평문과 이전 #7516의 긴 문자열·그림 경계를 포함한 15개에서 새 실패는 없다. 기존 문서의 비기본 caret 서식과 실제 `paste_html_in_cell_native` 진입점은 아직 별도 검증하지 않았다. 첫 문단 앞 평문의 caret 상속이 기본 서식으로 달라지는 사실을 숨기지 않고 다음 검증에 포함한다.
