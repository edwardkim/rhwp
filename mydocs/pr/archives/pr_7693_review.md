---
kind: report
status: active
last_verified: 2026-10-09
---

# PR #7693 리뷰 — Visual Sweep 지침과 도구

## 최종 판정

승인 — 전체 페이지 Native·fresh WASM TSV 제출, 한 페이지라도 90% 미만이면 재검토하는 지침과
Visual Sweep 도구 계약을 함께 검증했다. 정확히 90%는 통과한다. code head의 전체 CI와 별도
검증이 모두 성공했다. 이 기록을 포함한 trailing head의 CI·병합 가능 상태를 확인한 뒤 통합한다.

## 대상과 역할

- [PR #7693](https://github.com/edwardkim/rhwp/pull/7693), 작성자 `jangster77`, collaborator self-review.
- base `devel`; 최초 고정 base `6f66932a73ee6fd05a74a83bdc438c9499a89a12`.
- 검토한 code SHA `17032d307ff8abbdcd51be1fe7978f2fb2511b57`. 기록 전 최신 base `6f66932a73ee6fd05a74a83bdc438c9499a89a12`.
- branch `docs/visual-sweep-page-tsv-gate-20261009`, 원본 저장소 `edwardkim/rhwp`.
- 지침·문서 21개와 Visual Sweep Python/MJS·계약 테스트 6개만 분리했다.
  원 수식·페이지 보정 브랜치의 Rust·WASM 배포물·글꼴·fixture는 포함하지 않았다.
- 사용자 승인 범위는 PR 생성·CI 모니터링·병합·후속 처리다. self PR이므로 reviewer를 지정하지 않았다.

## 지침 검토

- 각 문서의 전체 페이지 TSV를 `--page`·`--pages` 없이 Native/fresh WASM에서 각각 산출하고
  **두 TSV 원본을 모두 제출**한다. 대표 review/overlay PNG만 본문에 표시한다.
- 어느 한 쪽이라도 `tolerant_content_match_percent < 90`이면 재검토·수정·새 head 재실행이 필수다.
  평균값·대표 쪽·CI 성공으로 미달을 상쇄하지 않는다. 누락·측정 불가·전체 쪽수 차이도 보류한다.
- 해결 불가능한 실제 글꼴 차이는 정식 증거·gate 예외 계약만 적용한다. 새 렌더링 회귀 추가의
  90% 선행 조건은 글꼴 예외로 면제하지 않는다.
- 90% 이상이어도 수식·장평·기준선·첨자·장식·기호 양쪽·본문·보기·탭·줄 사이 간격을 확대 판독한다.
- 실패한 회귀는 원본 출력 보정 → 독립 Print PDF와 시각 검증 → 독립 기대값에 따른 관계형 검사
  순서다. SHA를 고정한 upstream base에서도 이미 미달인 부적합한 검사만 별도 이슈로 이관한다.
  현재 수정으로 생긴 회귀는 구현을 고친다. 특정 과거 이슈 번호를 일반 이관 지침에 고정하지 않았다.
- collaborator self PR은 정확한 code head CI 성공 뒤 리뷰·오늘할일을 trailing commit으로 추가하고
  trailing head CI를 다시 확인한다. 이번 기록도 초기 CI 성공을 확인한 후 작성했다.

## 도구 구현과 실제 검증

| 구현 주장 | 검사와 결과 |
| --- | --- |
| 정확히 90% 통과, 한 쪽 미달·누락 보류 | 기존 gate 경계·다중 페이지 검사 PASS; NaN·무한대·비정상 값 추가 검사 PASS |
| 높은 선택 쪽 점수로 전체 쪽수 차이를 면제하지 않음 | 원본 export 메타데이터·PDF 전체 쪽수 비교, TSV의 count 근거 보존 검사 PASS |
| 필수 쪽수 증거 실패 시 과거 통과 요약 무효화 | 실패 이전 raster 미실행 및 summary가 `failed/re_review_required`로 바뀌는 검사 PASS |
| Native/WASM/글꼴 정책의 같은 조판 세대 | 명령 전달·다른 세대 manifest/resume 거부 검사 PASS; 실제 Chrome 2022/2024 export 확인 |
| RHWP만 파랑, PDF만 빨강과 범례 | 실제 overlay 생성의 소속 색상·잉크 차이 건수·JSON 범례 검사 PASS |

- `python -m unittest discover -s scripts/tests -p 'test_visual_sweep*.py'`: **104 PASS / 0 FAIL**.
- 잘못된 경계 비교(`<=90`), 쪽수 차이 차단 제거, 색상 반전의 메모리 내 음성 대조:
  **3건 모두 관련 검사가 의도한 결함으로 실패**했다. 저장소 파일을 변이하지 않았다.
- `node --check scripts/export-wasm-for-sweep.mjs`: PASS.
- 변경 Markdown 21개 상대 링크 검사, 변경 manual front matter, `git diff --check`: PASS.
- 실제 Native/WASM 도구 smoke: `samples/hwpx/eq-002.hwpx`와 기존 `pdf/hwpx/eq-002.pdf`,
  전체 1쪽, `--compat 2022 --dpi 96 --silhouette-only`. 양쪽 TSV의 p1은 93.13814%,
  원본 쪽수는 각각 RHWP 1 / PDF 1, gate는 측정 전용 `not_evaluated`다.
  실제 Chrome exporter의 `--compat 2024`도 `layoutGeneration=2024`와 1쪽을 기록했다.
- smoke 렌더러 source는 검증한 `3ba6bffa472c1a4d87b512f593d632dd952030ff`의 Native와
  fresh WASM 산출물이다. 도구 code와 렌더러 source를 구분했다. 엔진을 새로 수정하지 않았으므로
  사전 검증한 렌더러 산출물을 재사용했다. 현재 수식 보정 브랜치의 전수 시각 통과 근거로 쓰지 않는다.
- 로그·TSV·JSON·nextrun은 ignored `output/pr-review/visual-sweep-guidelines-20261009/`에만 보존했다.
  source PR에 생성 suite·baseline·TSV를 추가하지 않았다.

## 초기 code head CI

모두 위 code SHA의 실행이며 완료·성공을 확인했다.

| Workflow | 실행 | 결과 |
| --- | --- | --- |
| CI Impact Policy Controller | [37907804605](https://github.com/edwardkim/rhwp/actions/runs/37907804605) | success |
| Skill router gate | [37907804614](https://github.com/edwardkim/rhwp/actions/runs/37907804614) | success |
| Proptest roundtrip | [37907805230](https://github.com/edwardkim/rhwp/actions/runs/37907805230) | success |
| Adapter inter-diff | [37907805192](https://github.com/edwardkim/rhwp/actions/runs/37907805192) | success |
| CodeQL | [37907805196](https://github.com/edwardkim/rhwp/actions/runs/37907805196) | success |
| CI | [37907805168](https://github.com/edwardkim/rhwp/actions/runs/37907805168) | success |

CI에는 fmt·Native/WASM/workspace lint, Native Skia, Archive A/B/C/D 회귀, frontend package 검증이
포함됐다. 실행 대상에 따른 skipped 작업은 통과로 바꾸어 적지 않았다. Rust source·Rust test/helper는
변경하지 않았으며 로컬에서는 이 도구 범위의 계약·실제 호출·문서 검증을 수행했다.

## 조판 원칙 판정과 남은 범위

- **충족:** 도구의 gate·출처·전체 쪽수·조판 세대·색상 계약과 역할별 지침의 정합성.
- **비해당:** 제품 renderer/layout/typeset/paint·편집 저장의 구현, 실물 문서의 회귀 기대값 변경.
  이 PR은 해당 생산·소비 코드를 바꾸거나 특정 문서의 조판 개선을 주장하지 않는다.
- 원 수식·페이지 보정과 원본 문서 전수 검증은 별도 브랜치에서 진행 중이다. #7642는 `Refs`로
  유지하며 이 PR로 종료하지 않는다.
- 다음 조건: 문서 trailing의 실제 merge tree에서 최신 오늘할일 보존·링크·공백 검사를 통과한 뒤
  push하고, 최신 head CI·merge 상태를 다시 확인한다. 병합 후 duration 결과·devel 포함·후속 안내·
  이번 PR 전용 worktree/ref 정리를 확인한다.
