# PR #8421 리뷰 — 시험지 ingest 안내 정정

## 최종 판정

**승인 — 메인터너 보정 head의 로컬 검증·GitHub CI·self-review 후 원 PR을 정상 merge했다.**
원 head의 P2 보완 요청을 해결한 후보 `fe915543091ea964e5c691b391e890cd028e5a91`를 검토한다.
원격 push·새 head CI·self-review·merge를 완료했다. merge SHA는 `f6681b19b78b34963b0b50208a33fe281af872df`다.

## 접수와 승인 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8421 · 작성자 `jeong-sik` (CONTRIBUTOR).
- 사용자 지시: “8421 PR 을 메인테이너쪽에서 처리해서 머지해야 겠습니다.”
- 원 head: `d8c2fc3e1a04a006e971219dd3fb0b175ee96987` · 원 기여 커밋 9개 유지.
- 최신 devel: `0b697f90c3637558196b9e3b81c4eb59049432af` · API base metadata의 과거 SHA와 구분했다.
- reviewer `edwardkim`을 지정하고 API로 확인한 뒤 파일 조사와 fetch를 진행했다.
- maintainer_general + intake_and_review + local_validation + post_merge 경로.
- review-only 허용 경로 밖인 `.claude/skills/**` 변경이다. CI의 실제 영향 분류와 새 head 결과를 확인하며 무조건 fast-pass로 단정하지 않는다.

## 원 기여와 메인터너 보정

원 기여는 YAML description을 유효한 한 줄로 고치고, 별도 모델 API 키가 필요 없다는 설명과
네트워크 불필요 주장을 구분했다. 과거 #182를 근거로 그림 누락을 성공 처리하던 안내도 개선했다.

기존 [CHANGES_REQUESTED의 P2](https://github.com/edwardkim/rhwp/pull/8421#pullrequestreview-5727582196)는 최신 devel에서도 유효했다.
`src/cli/commands/generation.rs`의 `build-from-ingest`는 `--media-dir`의 존재만 확인한다.
`src/document_core/builders/exam_paper.rs`의 image 분기는 Picture/BinData 대신
`[이미지: …]` 텍스트를 생성한다. question·공유 지문·boxed의 공통 제한이다.
하위 `src/serializer/hwpx/picture.rs`가 구현돼 있다는 사실만으로 이 ingest 경로가
이미지를 지원하는 것은 아니다. 관련 #182·#5319는 이미 closed이며 이번 PR은 재종결하지 않는다.

메인터너 보정은 그림 필수 입력의 F18 중단을 SKILL, 알려진 한계, 배치·빌드·검증 안내,
예제·트랜스크립트·fixture와 생성기 문구에 반영했다. 수식·표 이미지도 같은 제한이다.
경로·크기·배치 재시도나 CLI 종료 코드 0으로 완료 처리하지 않는다.
이미지 삽입 구현은 이번 문서 정정 범위에 포함하지 않았다.

## 로컬 검증

| 검사 | 결과 |
| --- | --- |
| 시험지 계약 + route/catalog/author unittest | 48개 통과 |
| 실제 `run_ci_skill_gate.py` | gate 3회 및 부속 unittest·27개 Skill catalog sync 통과 |
| YAML parser와 router line-reader의 description | 동일 값 확인 |
| JSON | 76개 파싱 통과 |
| 생성기 | 원 head 대비 문자열 외 AST 동일 |
| 격리 재생성 | 103개 산출물과 저장 파일이 끝 공백 정규화 후 일치 |
| 변경 Markdown 내부 링크 | 36개 문서 통과 |
| 공백·diff 범위 | `git diff --check` 통과, 스킬 문구·데이터만 변경 |
| source 후보의 최신 base merge tree | `b0293e561f681f124511c6e6693c85fe7c219f97` · 검증 tree와 일치 |

명령: `python3 -m unittest scripts.tests.test_agent_exam_ingest tools.skill_router.test_route tools.skill_router.test_catalog_routes tools.skill_router.test_author_skill`,
`python3 tools/skill_router/run_ci_skill_gate.py`,
`python3 scripts/check_markdown_links.py <변경 Markdown 목록>`, `git diff --check`.

전체 스킬 링크 검사에는 PR에서 바꾸지 않은 `fixtures/md/sample_exam.md`의 외부 입력
`figures/pm10.png` 참조가 있다. 존재하지 않는 샘플 이미지를 새로 생성하지 않고 기존 예제 데이터로 구분했다.
변경 예제·행렬의 `![alt](path)` 문법 예시는 inline code로 고쳐 실제 링크로 오인되지 않도록 했다.
변경 파일의 링크 오류는 0건이다.

소스 코드와 helper·ingest 스키마는 최신 devel과 동일하다. 생성기는 설명 문자열만 바꾸며
실행 로직은 유지했다. 원 기여자가 보고한 기존 Ruff·Pyright 진단의 전체 해결을 주장하지 않는다.
모델 Vision, 실제 시험지 변환·한컴 표시, Rust 전체 lint/회귀·Native/fresh WASM·Visual Sweep은
실행 로직이나 조판 소비값을 바꾸지 않는 이번 문서 정정에 비해당이다.
조판 원칙 준수 검토도 같은 이유로 비해당이다. HWP/HWPX/PDF 검증 입력은 새로 사용하지 않았다.

## 원격 처리와 후속 기록

- 원 contributor branch를 유지하고 `d8c2fc3e1a04a006e971219dd3fb0b175ee96987` 위에 메인터너 보정 2개를 잇는다.
- 로컬 검증 commit `4ed28af8b4da98004eb1daf64ed65ab77df05be2`의 운영 계획 문서만 제외한 tree와,
  최신 base + 원격 제출 후보의 병합 tree가 정확히 같다. 임의의 devel 동기화를 source에 넣지 않았다.
- 새 head CI·정책 status와 정확한 head의 self-review 후 normal merge를 완료했다.
- 후속 문서는 maintainer 직접 반영으로 archive·오늘할일·검증 요약만 한 운영 기록 commit에 보존한다.

자체 검토에서 입력 정규화·판단 트리·boxed 스키마와 실제 지원의 구분·함정 안내 및 I025를 추가 정합화했다. 같은 48개 계약·문자열 외 AST·103개 재생성 일치·36개 변경 Markdown 링크를 재검증했다.

## 최종 CI와 merge 확인

| 검사 | 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/41431435836) | success: lint·Frontend package·Native Skia·Archive A–E·Build & Test |
| [Skill router gate](https://github.com/edwardkim/rhwp/actions/runs/41431435757) | success |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41431435860) | success |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/41431435834) | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/41431435756) | success |
| [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/41435296198) | success |

최종 head는 `fe915543091ea964e5c691b391e890cd028e5a91`다. check 33개 success·6개 skipped와 commit status
CI Impact Policy success를 확인한 뒤 [승인 리뷰](https://github.com/edwardkim/rhwp/pull/8421#pullrequestreview-5735721570)를 게시했다.
현재 base/head를 다시 확인하고 `--match-head-commit`을 지정해 merge했다.
원 기여 head의 9개 commit을 ancestor로 보존했다.
실제 merge 부모는 `0b697f90c3637558196b9e3b81c4eb59049432af`와 `fe915543091ea964e5c691b391e890cd028e5a91`이며,
최종 tree `b0293e561f681f124511c6e6693c85fe7c219f97`는 로컬 및 GitHub 테스트 merge tree와 같다.

Render Diff는 변경 경로의 workflow trigger에 해당하지 않는다. 기존 실행 결과를 재사용했다고
표현하지 않는다. 처음 제출한 보정 head 뒤 자체 검토에서 하위 안내를 추가 정합화했으며,
최종 head에서 CI를 새로 완료했다. 이전 head의 실행은 GitHub concurrency 정책으로 자동 취소됐다.
수동 workflow 취소·재실행·cache 삭제나 CI 정책 변경은 하지 않았다.

[검증 요약](../assets/pr8421-exam-ingest/README.md),
[정확한 head의 CI 원장](../assets/pr8421-exam-ingest/ci-latest.json),
[merge gate](../assets/pr8421-exam-ingest/merge-gate.json),
[실제 merge](../assets/pr8421-exam-ingest/merge-result.json)를 보존했다.
자동 종료할 연결 이슈는 0개이며 #182·#5319는 기존 closed 상태를 유지한다.

후속 운영 기록은 이 archive·계획·검증 요약·오늘할일만 한 commit으로 직접 반영한다.
감사 댓글과 정리 실행 결과는 `/home/edward/.local/state/rhwp/pr8421-cleanup-20261102.json`에 남긴다.
contributor fork branch, 사용자 worktree, 실행 중인 Studio와 배포 WASM은 보존한다.

후속 문서 검증: 변경 문서 4개의 링크와 공백 검사를 통과했다. 전체 문서 메타데이터 검사에서는 기존 기술 문서 4개의 누락 16건을 확인했다. 해당 네 파일은 검토 전 devel과 바이트가 동일하고 이번 변경 대상에 없으므로 새 오류는 0건이다. [기준선 확인](../assets/pr8421-exam-ingest/metadata-baseline.json)에 기록했으며 범위 밖 문서는 수정하지 않았다.

## merge 후 소요 시간 메타데이터

[메타데이터 갱신](https://github.com/edwardkim/rhwp/actions/runs/41435573859)은 success로 완료했다. B–E의 동일 attempt 자료를 검증해 `ready:true`로 수집하고 metrics branch에 게시했다. 실제 수집·게시 결과는 [갱신 원장](../assets/pr8421-exam-ingest/postmerge-duration.json)에 보존했다. merge 뒤 제품 검증 CI를 다시 시작하지 않았다.
