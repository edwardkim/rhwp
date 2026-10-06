---
kind: report
status: active
---

# PR #6933 검토: Markdown 내보내기의 줄 내부 이미지 누락

## 판정: 승인

원 PR의 Markdown 추출 변경 범위에서 수용 가능하다. 코드 검토에서 차단 결함을 발견하지 않았고,
정확한 원 PR head에서 집중 테스트 2개를 실행해 모두 통과했다. 메인터너 코드 보정은 하지 않았다.
이 판정은 코드 검토 결과이며 GitHub approve 게시, 원격 push 또는 merge 완료를 의미하지 않는다.

## 검토 대상과 출처

| 항목 | 확인 내용 |
| --- | --- |
| 원 PR | [#6933](https://github.com/edwardkim/rhwp/pull/6933) |
| 제목 | fix: Markdown 내보내기에서 텍스트 줄 내부 이미지 누락 수정 |
| 작성자 | `salgum1114`, GitHub `FIRST_TIME_CONTRIBUTOR` |
| 리뷰어 | `jangster77` 지정 완료 |
| 원 head | `227c8a75270c8fb0586af6644ac19e6828eaf832` |
| 원 branch | `codex/fix-markdown-inline-images` |
| 대상 branch | `devel` |
| 작성자가 명시한 기반 | `c72ad805cc60e4a5cf5689c18b44e7214cec68fe` |
| 조회한 upstream/devel | `c3bc96a6cc5aa852228ce157c2aa5104014a539e` |
| 로컬 검토 branch | `review/pr6933-markdown-inline-images-20260909` |
| 검토 당시 원격 상태 | OPEN, non-draft, MERGEABLE, CLEAN |
| 관련 이슈 | 본문에 별도 이슈 및 closing keyword 없음 |

검토 branch는 원 head를 그대로 사용했다. 최신 devel을 merge하거나 cherry-pick하지 않았으므로,
이번 집중 결과를 최신 devel과 합친 별도 merge tree의 실행 결과로 해석하지 않는다.
위 원격 상태는 조회 시점의 기록이며 병합 직전에 최신 head와 required checks를 다시 확인해야 한다.

## 변경 범위와 코드 검토

- `src/document_core/queries/rendering.rs`: Markdown용 `collect_line_text`가 `Image`를 만났을 때
  앞쪽 텍스트를 먼저 내보내고 이미지 식별 정보를 수집하도록 변경했다. 뒤쪽 텍스트는 이어서 수집한다.
- 이미지 처리 후 하위 순회를 반환하고, `TextLine` 처리도 상위 순회를 반환하므로 같은 방문 경로에서
  이미지를 두 번 수집하지 않는다. 기존 표 내부 이미지 전용 순회는 변경하지 않았다.
- `sec_idx`, `para_idx`, `control_idx`, `bin_data_id`를 기존 Markdown 이미지 처리와 같은 형태로
  전달한다. 이미지 바이트 추출·저장 경로를 별도로 변경하지 않는다.
- 일반 텍스트 추출 함수 `extract_page_text_native`는 변경하지 않았다. 문서 파서, 페이지 배치,
  PDF 렌더링 및 Studio UI 변경도 없다.
- `tests/cases/markdown_inline_images.rs`: 빈 문서에서 만든 합성 HWPX를 사용한다. CLI 실행 파일은
  런타임 `CARGO_BIN_EXE_rhwp`를 우선하고 컴파일타임 경로를 fallback으로 사용한다.
- 조회한 일반 댓글·인라인 리뷰 댓글·제출 review는 없었다. 봇 지적에 대한 별도 해소 대상도 없었다.

확인한 변경 범위에서 차단 결함은 없었다. 다중·연속 이미지, 이미지 전용 줄 등 모든 조합을 이번
합성 테스트 하나로 검증했다고 확대하지 않는다. 기존 Markdown 이미지 출력은 블록 구분을 사용하므로,
원본 줄의 인라인 기하나 문단 모양까지 그대로 보존한다는 판정도 아니다.

## 직접 실행한 집중 검증

실행 환경은 macOS이며 원 PR head `227c8a75270c8fb0586af6644ac19e6828eaf832`를 사용했다.
사용자 지시에 따라 집중 테스트 2개만 실행했고 제품 코드·테스트는 수정하지 않았다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs markdown_inline_images -- \
  --cargo-profile release-test --target-dir target/pr-review --test-threads 8
```

helper가 해석한 실제 실행 명령:

```bash
cargo nextest run --locked --test regression_suite_004 \
  -E 'test(/(^|::)markdown_inline_images::/)' \
  --cargo-profile release-test --target-dir target/pr-review --test-threads 8
```

| 테스트 | 결과 | 직접 확인한 계약 |
| --- | --- | --- |
| `inline_image_keeps_text_order_and_is_not_duplicated` | PASS, 0.019초 | 이미지 1개·토큰 1개, 앞 텍스트 → 이미지 → 뒤 텍스트 순서, 기존 평문 결과 유지 |
| `cli_exports_original_inline_image_bytes` | PASS, 0.802초 | CLI 성공·imageCount 1·Markdown 이미지 참조 1개, 저장 PNG와 HWPX 내부 원본 바이트 동일 |

- 최종 결과: **2 passed, 0 failed, 162 skipped**, exit code 0.
- 162 skipped는 같은 suite에서 필터로 제외된 테스트이며 전체 회귀 성공 수가 아니다.
- 테스트 실행 0.804초, 테스트에 필요한 `release-test` 컴파일 2분 32초.
- Nextest run ID: `0f9d85ac-0156-45c6-a281-18004c8a6ccb`.
- nextest가 `profile.ci-duration-observation.junit.report-skipped` 미지원 설정 경고를 출력했으나
  집중 테스트는 위 요약과 exit code로 성공을 확인했다.
- 파생 suite는 검증 준비물이며 제출 대상이 아니다. 임시 로그도 리뷰 문서에 첨부하지 않는다.

## GitHub CI와 남은 원격 조건

다음은 위 원 head에서 직접 조회한 GitHub 결과다. 작성자가 PR 본문에 적은 로컬 전체 테스트 결과와
검토자가 이번에 실행한 집중 결과를 혼동하지 않는다.

| 구분 | 조회 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/34310380611) | Build & Test, archive A/B/C/D 및 테스트 worker, Lint, Native Skia 성공. 적용하지 않는 일부 lane은 skipped |
| [CodeQL workflow](https://github.com/edwardkim/rhwp/actions/runs/34310380627) | preflight 및 JavaScript/TypeScript·Python·Rust job 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34310380477) | Canvas visual diff 성공 |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34310380650) | 성공 |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34310380595) | 성공 |
| CI Impact Policy | SUCCESS |
| [별도 CodeQL check](https://github.com/edwardkim/rhwp/runs/102340026857) | **NEUTRAL**, `2 configurations not found` |

별도 CodeQL check는 devel의 JavaScript/TypeScript·Python 분석 구성 두 개를 이번 PR에서 찾지 못해
추가된 alert를 판단할 수 없다고 설명한다. annotations는 0개였다. 이는 이번 변경의 취약점을 보고한
결과는 아니지만, 분석 비교가 완전하다는 뜻도 아니다. workflow job 성공을 이 별도 check의 SUCCESS로
바꾸어 적지 않는다. 최종 병합 전에는 해당 상태의 정책상 허용 여부와 최신 required checks를 확인한다.

## 생략한 검증과 시각 증적 판단

- 사용자 요청에 따라 로컬 전체 회귀, Clippy·fmt 재실행, 독립 workspace/WASM 빌드, Studio 전체 E2E를
  실행하지 않았다. 테스트 명령에 수반된 컴파일만 수행했다.
- MCP 실제 호출은 이번 검토자가 실행하지 않았다. 공통 추출 API와 CLI 동작을 집중 테스트로 확인했다.
- 원 PR의 스크린샷은 작성자가 제공한 Markdown 출력 비교이며 검토자가 재산출한 증적으로 표시하지 않는다.
- 페이지 배치·렌더링을 변경하지 않는 Markdown 추출 계약이므로 별도 한컴 PDF 변환·페이지 visual sweep은
  이번 범위에 해당하지 않는다. 직접 확인한 출력 순서·이미지 바이트 계약을 검증 근거로 사용한다.
- 신규 `samples/` 문서 추가·교체는 없다. 테스트 소스에 포함된 합성 입력을 공개 sample 보안 전수 검사나
  비공개 실물 문서 검증으로 표현하지 않는다.

## Merge 후 contributor PR comment 계획

실제 병합과 devel CI 성공 및 승인된 후속 처리 단계에서만 UTF-8 body file로 게시한다. 같은 merge SHA의
기존 댓글이 있으면 중복 게시하지 않는다. 게시 뒤 API로 본문을 재조회한다.

- 첫 기여 환영과 감사: "rhwp 첫 기여를 보내주셔서 감사합니다."
- 원 contributor 구현은 Markdown 줄 내부 이미지 누락 수정이며 메인터너 코드 보정은 없었다고 구분한다.
- 실제 merge SHA, 최종 PR/devel CI URL, 위 집중 테스트 **2개 통과**를 기록한다. 이번 검토에서 전체 회귀나
  MCP 직접 호출을 수행한 것처럼 쓰지 않는다.
- 최종 merge SHA에 고정된 `mydocs/pr/archives/pr_6933_review.md` 링크를 제공한다. 존재하지 않는
  비교 PNG·PDF 링크나 visual sweep 수치를 만들지 않는다.
- 원 본문에 closing issue가 없음을 확인하고, 관련 없는 이슈를 종료하지 않는다.
- contributor fork branch는 보존하고 이번 로컬 검토 branch만 종료 절차에 따라 정리한다.

## 현재 산출 상태

집중 검증과 최종 리뷰 문서 작성을 완료했다. 사용자 승인에 따라 이 문서와
`mydocs/orders/20260909.md`를 원 PR의 문서 전용 trailing commit으로 제출한다.
로컬 devel은 `74d0a68b74919761cc30343f7511dbc5d0fe32d3`까지 fast-forward했다. 원 PR 제품 코드는
검증한 `227c8a75270c8fb0586af6644ac19e6828eaf832` 그대로이며 devel을 merge/rebase하지 않았다.
최신 devel의 다른 작업 기록을 source branch에 복사하지 않고 이번 PR의 오늘할일만 보완했다.
GitHub approve·merge·후속 댓글은 수행하지 않았으며, trailing head의 CI는 별도로 확인해야 한다.
