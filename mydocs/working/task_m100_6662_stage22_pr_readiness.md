---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage22_pr_readiness.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 22단계: 제목 보정의 최종 PR 준비

## 계획과 범위

코드 후보 `6dcd46608`. 21단계에서 제목 7개/기존 snapshot 8개/SVG lib 52개를 통과했다.
20단계 전체 회귀 실패는 기존 golden을 바꾸지 않고 수정했다.

- 18단계에 기록한 workflow route와 사용자 범위를 유지한다. #6712까지만 구현한다.
- 최종 코드의 fmt/prepare, 전체 nextest, native/WASM/all-target Clippy, workspace build,
  manifest check를 순차 실행한다. `target/pr-review`, `--locked`, nextest 12 threads를 쓴다.
- 새 한국어·중국어 원본에 security 검사, IR/overflow baseline dump를 실행하고 증가 여부를 확인한다.
- Native Skia 공식 3종과 WASM wrapper `--no-opt`를 실행한다. Docker 부재는 명시하고 표준
  최적화 Docker WASM 성공과 혼동하지 않는다.
- 기존 두 기준 PDF로 visual sweep 1/2쪽을 재산출하고 최종 비교 4장을 직접 확인한다.
  20단계 중간 PNG를 이번 코드 SHA가 표시된 최종 패널로 교체한다.
- 코멘트 계획은 20단계를 승계하며 #6712에 최종 비교 4장, #6708에 기존 비교 2장을 실제 표시한다.
  로그/중간 PNG/SVG/JSON은 커밋하지 않는다. #6699는 잔여 dx=-1.41px 때문에 Ref로 유지한다.
- 오늘할일/번호가 붙은 PR review는 PR 생성 후 동일 PR trailing 작업이며 지금 만들지 않는다.
  이 단계에서는 원격 push, PR 생성, issue close를 하지 않는다.

## 결과

**로컬 PR 준비 완료.** #6712의 제목 겹침을 포함한 두 문서의 시각 판정을 갱신했다.
원격 push/PR 생성/issue close는 하지 않았다. Docker 최적화 빌드 대신 아래 명시한 host
WASM 진단을 수행했으며, 서로 같은 검증이라고 주장하지 않는다.

- 재실행 전체 nextest: **9,096 passed (6 slow), 46 skipped**, 414.500초,
  빌드 포함 512.770초, exit 0. 이전 두 SVG snapshot 실패가 해결됐음을 전체 실행에서도 확인했다.
- Node 웹폰트 raster 계약 2 passed; 실제 Chrome viewport + Python Visual Sweep 47 tests,
  11.342초, OK. source-side unit inventory 4,205 tests / 298 modules check도 통과했다.
- Native Clippy 43.880초, WASM32 Clippy 42.975초 모두 exit 0.
- Workspace build 95.399초, all-target Clippy 70.299초, manifest check 1.214초 exit 0.
- 새 두 HWP의 상대 경로를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 명시한 security/injection
  검사 20 passed (새 sample 전수 detector와 injection 검사 포함), 실행 2.039초, exit 0.
- IR dump 계약 4 passed / 162 skipped / 101.598초, overflow dump 16 passed / 50.470초.
  모두 exit 0. overflow 16개 part를 구조적으로 병합 비교한 결과 기존 12행과 수치가 같았다.
- IR은 baseline 568행 대비 현재 250행, 증가 0/감소 318행이다. 감소는 모두
  `hwp5rb`의 `list_header_width_ref`(중첩 경로 포함)다. base `1f861362a` 대비 파서·직렬화기·
  IR 진단/검사 코드 변경이 없으므로 이 감소를 이번 렌더 보정의 성과로 집계하지 않는다.
  무관한 baseline 일괄 갱신도 하지 않는다. 새 #6712 원본에 비영 IR/overflow 행은 추가되지 않았다.
- Native Skia lib: rhwp 3,930 passed / 13 ignored, 내부 crate 15+165+2 passed, exit 0
  (전체 명령 226.901초). 공식 placeholder 제어군 2 passed / 0.733초,
  direct PDF 제어군 4 passed / 0.329초, 두 명령 모두 exit 0.
- `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web
  --out-dir /tmp/rhwp-stage22-wasm --no-opt`: exit 0, 312.577초.
  `command -v docker`는 exit 1이며 Docker 표준 경로/wasm-opt는 실행하지 않았다.
- 새 WASM을 Chromium의 임시 loopback 서버에서 직접 import하여 두 HWP를 `HwpDocument`로
  열었다. 한국어/중국어 모두 pageCount=2, 1/2쪽 `renderPageSvg()` 성공.
  서버와 browser는 종료했으며 Cargo/Rust 검증 프로세스도 남아 있지 않다.
- WASM SHA-256: `5d40af470bd36805ae820582c258ec9b7190b92e16ccfa3a7f86449adb20f33d`.
  최초 SVG 바이트 동일성 비교는 실패했다. CLI의 로컬 `@font-face` style 요소가 추가되는 차이를
  확인한 뒤 렌더 XML을 구조적으로 비교했다. 이 폰트 공급 CSS를 제외한 4쪽의 요소 수
  845/1052/705/889, 태그/속성/텍스트는 모두 동일하다. 바이트 동일이라고 쓰지 않는다.
- WASM SVG에도 동일한 webfont 공급 경로를 적용해 제목을 실제 Chromium에서 캡처했다.
  2배 ink 간격 `5, 2, 39, 5, 2, 36, 1, 8, 41, 7`px, 모두 양수이며 직접 확대 확인했다.
  이 캡처/JSON/WASM/로그는 임시 산출물이고 커밋에는 넣지 않는다.

첫 전체 검증 시도는 fmt/prepare 통과 후 컴파일 도중 프로세스가 exit 143으로 종료됐다.
test summary가 없으므로 실패/통과 테스트 수를 추정하지 않는다. 남은 Cargo/Rust 프로세스가
없음을 확인했고 `/tmp/rhwp-stage22-validation-interrupted`에 원본 로그를 분리한 뒤 재실행한다.
종료 신호의 발신 원인은 확인되지 않았다.

## 최종 시각 후보

CLI SHA-256: `65aba36c3f8ce312f046047ff1f051e492b693b1e486ae3fdc552ef74e5c55ec`.
18단계에 기록한 동일 원본/PDF를 재사용했다. 한국어는 `-2020.pdf`, 중국어는 `-2024.pdf`다.
`visual_sweep.py --hwp <원본> --pdf <기준> --pages 1,2
--rhwp-bin target/pr-review/release-test/rhwp --key <6712-ko|6712-zh>
--out /tmp/rhwp-stage22-sweep/<key>`를 실행했다.
`VISUAL_SWEEP_CHROME=/home/tsjang/.cache/ms-playwright/chromium-1187/chrome-linux/chrome`을 지정했다.

- 두 sweep 모두 exit 0, 요청 2쪽/완료 2쪽/누락 0쪽이다. 원본 PDF 2쪽, 수정 전 SVG 3쪽,
  수정 후 SVG 2쪽이다. 4개 패널 전체를 열어 제목/그림 어울림/본문/중첩 표/footer를 확인했다.
- 한국어 제목의 문자 겹침은 해결됐다. 실제 browser ink 간격은 21단계 결과와 같다.
  최종 CLI의 1배 확인도 `3, 2, 20, 3, 2, 19, 1, 5, 21, 4`px로 모두 양수이며 exit 0이다.
  원본 폰트의 outline과 같다는 의미는 아니며 대체 글꼴 디자인 차이는 남는다.
- 96dpi, threshold 32: 한국어 pixel match 84.51%/81.95%, ink proxy 14.79%/48.95%;
  중국어 pixel match 83.12%/77.89%, ink proxy 12.09%/41.24%다. 자동 수용 지표로 쓰지 않는다.
- 자동 Square wrap 후보는 1쪽에 각 1건, 2쪽에 0건이다. 실제 TextRun이 아니라 가용 TextLine
  경계가 그림 옆에 닿은 후보라는 18단계 판정을 유지한다. 제목의 glyph overlap과 구분한다.
- 원본과 수 px 위치 차이, 중국어 끝 테두리 약 19px 차이는 남는다. #6712의 어울림/
  중복 높이/쪽 밀림/말미 소실과 사용자 지적 제목 겹침은 개선된 것으로 판정하되,
  전체 회귀 및 잔여 게이트 통과 전 PR 준비 완료로 쓰지 않는다.

| 최종 비교 파일 (`mydocs/pr/assets/`) | SHA-256 |
| --- | --- |
| issue_6712_ko_p1_compare.png | 253e402a52b4f242a807a83299afad5b47bb6b41fbc5b0d64d91fdaff7ebf7cc |
| issue_6712_ko_p2_compare.png | 517b7ec7f7b4c50354d79181a3c14689e536a03c89464cf5ec708b6eeb664e7c |
| issue_6712_zh_p1_compare.png | f65d868d4fe93c1a2a550a01d0cb88113e2bb091d61ebfbf63b039a2402d79e9 |
| issue_6712_zh_p2_compare.png | 2ff1d9067e141f1c33d22af765c1ede2e0872c69ce3733f630e79e08969259ab |

## 코멘트 전달 계획

최종 게이트 완료 뒤 같은 PR review의 코멘트 계획으로 옮긴다. #6712에는 위 4개 이미지를
모두 표시하며, 별도 내부 ID 3개를 검증했다고 하지 않는다. 아래 형식의 SHA는 실제 merge SHA로
바꾸고 GitHub 자산 존재를 확인한다. 원본과 기준 PDF의 provenance, 각 비교 페이지,
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment),
자동 후보와 잔여 차이도 명시한다.

```markdown
![한국어 1쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/issue_6712_ko_p1_compare.png)
![한국어 2쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/issue_6712_ko_p2_compare.png)
![중국어 1쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/issue_6712_zh_p1_compare.png)
![중국어 2쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/issue_6712_zh_p2_compare.png)
```

#6708은 기존 `issue_6708_cover_before.png`/`issue_6708_cover_after.png`를 같은 형식으로 표시한다.
#6714는 signed offset 계약의 검증 범위를 적고 내부 원본 8개 전체 시각 확인으로 과장하지 않는다.
#6699는 Ref 및 잔여 dx=-1.41px를 명시한다. 승인된 외부 작업 때 UTF-8 `--body-file`로
게시하고 API 재조회로 본문/이미지 링크/한글/줄바꿈을 확인한다. 이 단계에서 게시하지 않는다.

## PR 준비 범위

- 제목 후보: `fix(renderer): 표·그림 어울림과 한글 제목 겹침 보정`.
- 완료 근거를 기록한 #6712/#6708/#6714는 PR 본문에서 Closes 대상으로 준비한다.
  각각 두 확인 원본/기존 표지 원본/signed offset 코드 계약의 실제 검증 범위를 그대로 적는다.
- #6699는 부분 보정으로 Ref만 적는다. 이 이슈와 다른 미해결 이슈를 모두 해결했다고 쓰지 않는다.
- 최초 PR에는 오늘할일과 번호가 붙은 self-review 문서를 포함하지 않는다. 생성 후 같은 PR의
  trailing commit에서 검증 결과·위 코멘트 계획을 옮기는 사용자 지시를 유지한다.
- 최종 커밋에는 비교 4장과 이 보고서, 18단계 fixture README 상대 링크 수정만 포함한다.
  원본 golden, Cargo manifest/lock, 파생 suite, raw 검증 로그는 변경하지 않는다.
