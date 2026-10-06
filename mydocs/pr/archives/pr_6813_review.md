# PR #6813 self-review

## 접수와 범위

| 항목 | 작성 시점 참고값 (2026-09-06) |
| --- | --- |
| PR | [#6813](https://github.com/edwardkim/rhwp/pull/6813) |
| 작성자 / reviewer | jangster77 / collaborator self-review이므로 reviewer 지정 및 approve event 없음 |
| base / source | `devel` / `fix/jeong-sik-open-issues-20260905` |
| 검증 코드 | `6dcd46608` |
| 최초 PR head | `964ef10a646ca703cce630a68b504ddafeda4d0e` |
| 증적 commit | `2b75a052c7eb65b32e9a2fc5f83cdc765af7ed36` |
| 규모 | 최초 51 files, +3957/-145, 23 commits |
| 원 검증 base / fetch한 base | `1f861362a` / `3960844b2` |
| merge 상태 | 최초 MERGEABLE / BLOCKED, CI 진행 중. merge 직전 재조회 필요 |

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md` (대형 diff)
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본/보조 문서,
  `verification/visual_sweep_guide.md`, `codex/docs_and_git_workflow.md`, `github_operations.md`.
- 이번 요청은 PR 생성과 같은 PR의 기록 추가다. merge/issue close는 별도 승인 및 최신 CI 확인 뒤 수행한다.
- 분석, 코드 수정, 검증 보고, 일반 commit을 단계별로 반복했다. 종료 범위는 #6712까지이며,
  관련 작성자의 나머지 열린 이슈를 모두 해결했다고 기록하지 않는다.

## 이슈별 판정

| 이슈 | 포함 변경과 확인 범위 | 본문 처리 |
| --- | --- | --- |
| #6712 | 셀 안 Square 그림과 후속 문단 흐름, 중복 높이, 분할 anchor, 말미 clipping, 비례폭 한글 제목 겹침. 확인한 한국어/중국어 각 2쪽 | Closes |
| #6708 | `tac-img-02.hwp` 1쪽 inline 그림의 저장 기준선. 전체 66쪽 유지, 대상 dx=-0.02/dy=-0.03px | Closes |
| #6714 | pagination raw u32 표 세로 오프셋의 signed 해석. 음수/0/양수 코드 계약 | Closes |
| #6699 | 글상자 안 표 그림의 좌표/inline cursor 부분 보정. 잔여 dx=-1.41px로 dx<1px 기준 미충족 | Ref, close하지 않음 |
| #6662 | 전체 조사 원장. 나머지 이슈의 완료 증거 아님 | Ref, close하지 않음 |

#6712는 작업지시자가 지정한 한국어·중국어 원본 두 개로 범위를 확정했다. 사내 문서 ID 3종을
각각 확보·검증했다고 주장하지 않는다. #6714도 미확보 내부 원본 8개 전체의 시각 확인으로 과장하지 않는다.
변경하지 않은 parser/serializer의 IR 진단 감소나 폰트 외형 차이를 이번 수정의 성과로 집계하지 않는다.

## 완료한 검증

[22단계 최종 보고서](../../working/task_m100_6662_stage22_pr_readiness.md)의 실제 명령과 결과를 재확인했다.
최종 코드 `6dcd46608` 뒤 두 commit은 증적/보고서/상대 링크만 바꿨다.

- fmt, native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy,
  suite manifest check 모두 exit 0. `--locked`, 고정 `target/pr-review`, Cargo 순차 실행을 사용했다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review
  --tests --test-threads 12 --no-fail-fast`: **9,096 passed, 46 skipped**, 414.500초, exit 0.
- 제목 계약 7개, 기존 SVG snapshot 8개, SVG lib 제어군 52개 통과. 기존 golden은 수정하지 않았다.
- 새 두 HWP를 명시한 security/injection 20개, IR dump 4개, overflow dump 16개 통과.
  IR 증가 0, 기존 overflow 12행 동일. 새 원본의 비영 IR/overflow 행은 없었다.
- Native Skia lib: rhwp 3,930 passed / 13 ignored, 내부 crate 15+165+2 passed.
  공식 missing-picture 2개와 direct PDF 4개도 통과했다.
- WASM host wrapper `--no-opt` exit 0. 새 WASM을 실제 Chromium에서 열어 두 HWP의
  pageCount=2, 각 1/2쪽 `renderPageSvg()` 성공을 확인했다. Docker/wasm-opt 최적화 경로는 미실행이다.
- CLI/WASM SVG는 폰트 공급 CSS를 제외한 구조가 4쪽 모두 같았다. 바이트 동일성이라고 쓰지 않는다.
- Node raster 계약 2개, 실제 Chrome viewport/Python Visual Sweep 47개 테스트 통과.
  source-side unit inventory 4,205 tests / 298 modules check도 통과했다.
- 첫 전체 시도는 컴파일 중 exit 143으로 끝났으며 성공에 포함하지 않았다. 위 수치는 전체 재실행 결과다.
- 제출 전 Markdown 24개 링크 검사에서 18단계 fixture 상대 링크 1건을 발견했다.
  `964ef10a6`으로 수정 후 24개 모두 이상 없음, `git diff --check` 통과를 확인했다.
- 최신 fetch base `3960844b2`는 원 base보다 76 commits 앞서 있었지만 merge simulation은 충돌 없었다.
  최초 merge tree `c8c104b0f4bd0845a4fb20b2a309da1bc58ae684`의 diff check도 통과했다.
  코드 head를 재베이스하지 않았으며 새 base와의 실행 호환성은 이번 PR CI에서 확인한다.
- 오늘할일을 추가한 문서 후보는 `mydocs/orders/20260906.md`만 add/add 충돌했다.
  기존 base에는 파일이 없고 upstream에서 별도로 생성됐기 때문이다. 초기 CI 성공 뒤 문서 commit
  `d7f165819`과 current base `3960844b2`를 잇는 bridge `359f4d3ea`에서 기존 기록과 이번 항목을
  모두 보존했다. source/test/fixture/workflow의 수동 충돌 해소는 없었다. bridge는 불필요한 최신화가 아니라
  실제로 확인한 오늘할일 충돌을 해소하기 위한 예외이며 fast-pass를 성공으로 미리 단정하지 않는다.
- bridge의 `git show --remerge-diff`는 오늘할일 한 파일의 충돌 marker 제거만 보고했다.
  `verify_review_only_merge_resolution.py --repository . --base-sha 3960844b2f4a546a120cbbb50ae72ef2e5e7239f
  359f4d3eaf38af5c38c39ea475bcac372a7169ac`는 `current-base-merge-resolution-mydocs-only`, exit 0이었다.
  초기 GitHub merge commit 대비 diff는 review 두 문서와 오늘할일뿐이며 mydocs 밖 diff는 0이다.
  최종 tree의 위 세 문서 내부 링크 및 diff check가 통과했다.

## 시각 검증

렌더 경로와 실제 HWP fixture가 함께 바뀌므로 직접 시각 검증이 필수다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따라 재검증했다.
[원본 README](../../../samples/issue6712/README.md)에 출처와 두 파일/PDF 링크를 보존했다.

| 입력 | `lastSavedWith` | 기준 PDF | 입력 SHA-256 / PDF SHA-256 |
| --- | --- | --- | --- |
| `samples/issue6712/한국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp` | 2020 / 11.0.0.9136 | `pdf/issue6712/한국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방-2020.pdf` | `70a6663e75fefedc001b2c249bd20f5b994596954120740252f484b1892e2097` / `ff0ab5e0cc70c4104d9dae960be01f736c0bf7a6d9a394212efb860dda1bdfd8` |
| `samples/issue6712/중국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp` | 2024 / 13.0.0.3379 | `pdf/issue6712/중국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방-2024.pdf` | `34a5964fa791ae662052cba8482efae682f13b45a0c0cd6ce633425566a2d5a9` / `aa85b871b5d8049af5bd8240fa210678a48ebe4b258fe6497a528de234cb6f97` |

기존 PDF를 재사용했으며 이번 최종 검증에서 MCP를 재호출하지 않았다. 실행은
`visual_sweep.py --hwp <원본> --pdf <기준> --pages 1,2 --rhwp-bin target/pr-review/release-test/rhwp
--key <6712-ko|6712-zh> --out /tmp/rhwp-stage22-sweep/<key>`다. `VISUAL_SWEEP_CHROME`은 설치된
Playwright Chromium을 가리켰다. CLI SHA는
`65aba36c3f8ce312f046047ff1f051e492b693b1e486ae3fdc552ef74e5c55ec`, WASM SHA는
`5d40af470bd36805ae820582c258ec9b7190b92e16ccfa3a7f86449adb20f33d`다.

| 문서 | PDF / Before / After 쪽수 | 완료 / 누락 | 1/2쪽 pixel match | 1/2쪽 ink proxy | 1/2쪽 자동 Square 후보 |
| --- | --- | --- | --- | --- | --- |
| 한국어 | 2 / 3 / 2 | 2 / 0 | 84.51% / 81.95% | 14.79% / 48.95% | 1 / 0 |
| 중국어 | 2 / 3 / 2 | 2 / 0 | 83.12% / 77.89% | 12.09% / 41.24% | 1 / 0 |

96dpi / threshold 32이며 자동 지표는 수용 기준이 아니다. 1쪽 wrap 후보는 실제 비어 있지 않은
TextRun 겹침이 아니라 가용 TextLine 경계가 그림 옆에 닿은 후보로 확인했다. 제목·그림·본문·중첩 표·
footer를 4개 패널 전체에서 직접 확인했다. 한국어 제목의 2배 실제 ink 간격은
`5, 2, 39, 5, 2, 36, 1, 8, 41, 7`px로 모두 양수다. bbox 겹침과 실제 ink 겹침을 구분했다.

대체 글꼴 디자인, 일부 그림의 수 px 좌우 위치, 중국어 끝 테두리 약 19px 차이는 남는다.
pixel-perfect 또는 전체 fidelity 완료를 주장하지 않는다. #6712의 그림 옆 흐름·중복 높이·쪽 밀림·
말미 소실 및 제목 겹침은 이번 판정 범위에서 해결됐다.

| 최종 대표 PNG (`mydocs/pr/assets/`) | SHA-256 |
| --- | --- |
| [한국어 1쪽](../assets/issue_6712_ko_p1_compare.png) | `253e402a52b4f242a807a83299afad5b47bb6b41fbc5b0d64d91fdaff7ebf7cc` |
| [한국어 2쪽](../assets/issue_6712_ko_p2_compare.png) | `517b7ec7f7b4c50354d79181a3c14689e536a03c89464cf5ec708b6eeb664e7c` |
| [중국어 1쪽](../assets/issue_6712_zh_p1_compare.png) | `f65d868d4fe93c1a2a550a01d0cb88113e2bb091d61ebfbf63b039a2402d79e9` |
| [중국어 2쪽](../assets/issue_6712_zh_p2_compare.png) | `2ff1d9067e141f1c33d22af765c1ede2e0872c69ce3733f630e79e08969259ab` |

#6708의 별도 표지 비교는 [6단계](../../working/task_m100_6662_stage6_inline_picture_baseline.md)의
원본/PDF/CLI SHA와 실제 전후 결과를 사용한다. `samples/tac-img-02.hwp`, `pdf/tac-img-02-2022.pdf`
1쪽을 확인했고 66쪽을 유지했다. 그림 y 오차 -166.63px가 -0.03px로 줄었고 x=-0.02px다.
5개 신규 및 91개 기존 집중 계약을 통과했으며 최종 전체 회귀에도 포함했다.

## Merge 후 contributor PR comment 계획

이번 PR은 외부 PR cherry-pick이 아니라 본인 수정 PR이다. 아래 계획을 해당 이슈와 #6813의 후속
코멘트에 사용한다. 현 단계에서는 merge/완료 코멘트를 게시하지 않는다.

- 비교 방법은 [Visual Sweep GitHub 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 링크한다.
- #6712에는 위 두 원본과 각 1/2쪽, 2/3/2쪽 매핑, 자동 후보 및 지표 표, 제목 ink 간격,
  사람의 판정과 잔여 차이를 함께 적고 아래 4장을 모두 표시한다.
- #6708에는 표지 1쪽의 전후 두 장과 dx/dy, 66쪽 유지, 집중/최종 전체 검증 결과를 적는다.
  이 기록에 없는 pixel 지표를 추정해 채우지 않는다.
- #6714에는 signed offset 계약과 실제 검증 범위를 적는다. 내부 8개 원본 전수 확인이라고 쓰지 않는다.
- #6699는 부분 수정과 잔여 dx=-1.41px를 명시하고 close하지 않는다. #6662도 열어 둔다.
- 아래 placeholder는 실제 merge SHA로 교체한다. asset이 그 commit을 통해 devel에 존재하는지 확인한
  뒤 승인된 코멘트만 UTF-8 `--body-file`로 게시하고 API로 Markdown/한글/줄바꿈/이미지 URL을 재조회한다.

```markdown
![한국어 1쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_ko_p1_compare.png)
![한국어 2쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_ko_p2_compare.png)
![중국어 1쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_zh_p1_compare.png)
![중국어 2쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_zh_p2_compare.png)
![표지 그림 수정 전](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6708_cover_before.png)
![표지 그림 수정 후](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6708_cover_after.png)
```

원시 로그·중간 PNG/SVG/JSON·실험 PDF는 commit하지 않는다. 최종 코멘트에 사용하는 대표 PNG 6장,
확정 기준 PDF 2개, 원본 HWP 2개와 재현/검증 Markdown만 보존한다.

## GitHub CI와 trailing 기록

- 최초 PR head `964ef10a646ca703cce630a68b504ddafeda4d0e`의 아래 실행은 모두 completed/success다.
  2026-09-06 20:20 KST에 pending/failure 0, 20:21 재조회에서 MERGEABLE/CLEAN을 확인했다.

| 초기 Full candidate 실행 | 결과 |
| --- | --- |
| [CI 34029158620](https://github.com/edwardkim/rhwp/actions/runs/34029158620) | success; lint, Native Skia, Frontend package, A/B/C/D 회귀 모두 통과 |
| [CodeQL 34029158604](https://github.com/edwardkim/rhwp/actions/runs/34029158604) | success; Rust/JS-TS/Python 분석 통과 |
| [Render Diff 34029158300](https://github.com/edwardkim/rhwp/actions/runs/34029158300) | success |
| [Adapter 34029158586](https://github.com/edwardkim/rhwp/actions/runs/34029158586) | success |
| [Proptest 34029158575](https://github.com/edwardkim/rhwp/actions/runs/34029158575) | success |

- 검사된 GitHub merge commit은 `c0687162210c4277d44a506bdff325ec3e548861`이고 부모는
  `3960844b2`와 `964ef10a6`이다. 문서 bridge 뒤 이 commit과 mydocs 밖 diff가 없음을 확인했다.
- GitHub raw URL의 대표 PNG 6개를 실제 받아 PNG signature/content-type 및 로컬 SHA-256 일치를 확인했다.
- 오늘할일과 이 review 및 실행 계획을 같은 PR branch에 trailing commit으로 포함한다.
  single-parent 기록 commit은 문서만 바꾼다. current-base bridge는 remerge diff의 수동 해소가
  허용된 mydocs 경로뿐인지 별도로 검사하고 최종 tree의 내부 링크를 확인한다.
- 최신 head의 required check와 mergeability, 작업지시자 merge 승인이 남아 있다.

## 최종 판정

**승인**. 위에서 구분한 세 Closes 범위와 #6699 부분 수정에 대해 로컬 검증 및 직접 시각 검토를 완료했다.
이는 GitHub approve 또는 merge 승인이 아니다. 최신 PR head CI 성공, 충돌 없는 최신 base 정합,
작업지시자의 merge 승인을 선행 조건으로 유지한다. [실행 계획](pr_6813_review_impl.md)에 후속 순서를 기록했다.
