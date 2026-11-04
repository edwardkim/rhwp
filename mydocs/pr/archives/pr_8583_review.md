# PR #8583 리뷰 — WASM Print 메트릭·글꼴 프로그램과 JPEG 인쇄

## 최종 판정

**승인 — 현재 code head의 로컬 검증·전쪽 시각 검증과 GitHub CI를 완료했다.**
검증 code head는 `4446720c7227b292ba76a607af22696679cb9560`다.
이 문서와 오늘할일은 문서-only trailing commit으로 추가한다. 아직 실행 전인 trailing head의
required check·실제 fast-pass와 merge 직전 `MERGEABLE / CLEAN` 확인은 남은 병합 조건이다.
사용자는 CI 완료 후 이 기록 반영·머지·후속 처리를 승인했다. reviewer는 자동 지정하지 않았다.

## 접수 정보와 범위

- [PR #8583](https://github.com/edwardkim/rhwp/pull/8583), 작성자 `jangster77`, 대상 `devel`.
- code CI base `1924d371d6757392d063b8dc2e9bbe9a298be817`, 최초 제출 head `dc8e9b533bf5bb1dad0492016ae86d9c543a90b5`.
- 작성 시점 최신 devel `515412bef8`: #8203 텍스트 추출의 개체 자리표 중복 보정으로, Print/font 소비 경로와 변경 파일이 겹치지 않는다.
  검증 code head를 유지하며 실제 merge tree의 충돌·링크·기존 오늘할일 보존을 push 전에 확인한다.
- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `review_template.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `renderer_backend_verification.md`, `review_only_fast_pass.md`, `post_merge.md`.
- loaded documents: 위 문서와 `AGENTS.md`, `CLAUDE.md`, `pr_review_workflow.md`,
  문서·Git·GitHub 운영 및 Visual Sweep 정본. 기록 시각 2026-11-04 KST.
- [#8533](https://github.com/edwardkim/rhwp/issues/8533)의 WASM 후속 보정이다. 편람 3·4쪽 클릭·Esc는
  사용자 지시로 별도 [#8578](https://github.com/edwardkim/rhwp/issues/8578)에 남기고 미완성 mouse 변경은 base와 같게 철회했다.

## 구현과 조판 원칙

Screen Canvas 폭이 Print에 남지 않도록 Rust portable guard, Studio 상태 전환·오류 후 복원,
Sweep의 쪽수 조회 전 Print 전환을 연결했다. 실제 공급 프로그램의 family 체인을 측정·paint에
같이 전달하며, 정적 정상 TTF fallback의 Print 2024 윤곽은 제한된 cache로 소비한다.
원 face, Screen, Print 2022, HighQuality, variable/color/bold/italic, 미공급 face와 문맥이 필요한
단독 아랍어는 기존 경로를 유지한다. JPEG 프레임과 일반 그림은 공통 IJG RGB 복호화 결과를 사용한다.
PostScript 이름만 있는 face의 목록 생성과 실제 읽을 수 없는 Fontconfig cache face의 자동 공급도 보완했다.

문서명·쪽 번호별 좌표 보정, 본문 전진·원본 줄 정보·분할 계산·품질 문턱 변경은 없다.
프로그램 소유·문맥 경계·원점 및 전환 복원을 검사하고 실제 paint 결과를 독립 Print PDF와 대조했다.
140쪽 fixture는 독립 PDF의 무보간 절단이며, 보정 전 Native 100%·fresh WASM 99.98238%로
페이지별 98% 조건을 충족한다. 합성 `RHWPPrintFallbackContext.ttf`는 제품 배포가 아닌 문맥 검사 전용이다.

## 입력·전쪽 시각 결과와 직접 판독

[입력·독립 PDF·SHA·producer·대표 PNG](../assets/issue8533/wasm-print-20261104/README.md),
[전쪽 정밀 raw·PNG 해시](../assets/issue8533/wasm-print-20261104/page-results.tsv),
[직전 기록 대비 페이지별 차이](../assets/issue8533/wasm-print-20261104/recorded-source-comparison.tsv)를 보존했다.
입력은 `samples/2025 행정업무운영 편람(최종).hwpx`, 독립 기준은 기존
`pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`이며 원본·PDF와 기준 TSV는 변경하지 않았다.
전체 383쪽, compat 2024 Print·96 DPI·canonical PDFium RGB exact-DPI v2·문턱232·2px,
색상 경계 보정 전 raw를 사용했다. 전체 쪽수·누락·중복·내용 소속도 확인했다.

| 출력 | 쪽수 | 최저 raw | 최저 쪽 | #8389 기록 하한 하락 |
| --- | ---: | ---: | ---: | ---: |
| Native 최소 글꼴 | 383 | 99.96465% | 130 | 0 |
| fresh WASM 최소 글꼴 | 383 | 99.96403% | 130 | 0 |
| fresh WASM 일반 설치 | 383 | 99.96401% | 130 | 0 |
| Native 최소 글꼴·옵션 단독 | 383 | 99.96465% | 130 | 0 |

위 마지막 행은 `--handbook-gate`만 선택하고 외부 font path/environment를 지정하지 않은 자동 공급 실행이다.
207.3초·exit 0이고 명시 공급 Native와 383쪽 PNG 해시가 전부 같다. 일반 실행에 편람을 강제로 추가하지 않는다.
382쪽은 소수 둘째 자리 표시 100.00%, 130쪽은 99.96%이며 RGB 완전 동일을 뜻하지 않는다.
직전 일반 설치 WASM 대비 39쪽 개선·하락 0쪽이다. Native의 직전 일반 설치와 현재 최소 환경 비교는
환경 차이도 포함하며 raw 최저 차이 -0.00394%p를 코드 회귀로 오인하지 않는다. #8389 기록 정밀도
0.01%p의 하한은 전쪽 유지했다. 기록을 새 base 실행이라고 바꾸어 부르지 않는다.

30쪽 닫는 괄호, 140쪽 로그인 JPEG 버튼·테두리, 198·220쪽 글머리표와 뒤 간격,
272쪽 원·번호의 모양·상하 위치를 Native/fresh WASM raw·review·overlay로 직접 판독했다.
내용·줄바꿈·표·앞뒤 본문 누락은 보이지 않았다. 130쪽 삽입 화면의 기존 미세 차이는 남는다.
Native/WASM 실행 파일·프로그램·root pkg·Studio public·실제 초기화 JS/WASM 해시는
[생산 신원](../assets/issue8533/wasm-print-20261104/visual-validation.json)에 연결했다.
Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화가 아니다.
Canvas2D 결과를 CanvasKit 통과로 주장하지 않는다.

## 로컬 검증과 실패 검사 검토

[실제 명령·결과](../assets/issue8533/wasm-print-20261104/validation.md): workspace nextest
12,537 PASS·0 FAIL·45 skip(615.378초), fmt·Native/WASM/all-target Clippy·workspace build·manifest·unit tier PASS.
Native Skia root 3,857 PASS·12 ignore, 다른 libs 182 PASS, PNG 2 PASS·PDF 22 PASS.
Studio 2,718 PASS·2 skip·TypeScript PASS, 실제 WASM browser 13 PASS, Print bridge 1 PASS,
Sweep Python 245 PASS, 패키징 글꼴 계약 8 PASS다. Cargo 순차·jobs8·threads8·시각 capture 총8을 지켰다.

#7084 실제 10 PASS/1 FAIL → 11 PASS, 옵션 공급 2 FAIL → 관련 29 PASS → 전체 Python 245 PASS다.
기존 일·생활균형 검사는 Screen 측정 활성 상태에서 Print를 호출했다. 원점·소속 assertion을 지우거나
허용폭을 늘리지 않고 Screen과 portable Print 양쪽에 유지했다. 봉인 v1 원장도 실제 변경 집합의
독립 재생으로 검증했다. 검사 삭제·baseline 갱신으로 실패를 숨기지 않았다.

## 정확한 code head의 GitHub CI

아래 모든 검증의 head는 `4446720c7227b292ba76a607af22696679cb9560`다.

| workflow | run | 결과 |
| --- | --- | --- |
| CI | [41802090733](https://github.com/edwardkim/rhwp/actions/runs/41802090733) attempt 2 | success |
| CodeQL | [41802090578](https://github.com/edwardkim/rhwp/actions/runs/41802090578) | success |
| Render Diff | [41802090359](https://github.com/edwardkim/rhwp/actions/runs/41802090359) | success |
| Adapter inter-diff | [41802090565](https://github.com/edwardkim/rhwp/actions/runs/41802090565) | success |
| Proptest | [41802090494](https://github.com/edwardkim/rhwp/actions/runs/41802090494) | success |
| CI Impact Policy | [41807720156](https://github.com/edwardkim/rhwp/actions/runs/41807720156) | success |

CI root archive A/B/C/D/E는 실제 attempt 1 worker에서 각각 3,801/2,365/2,129/1,814/2,130 PASS,
합계 **12,239 PASS·0 FAIL·45 skip**이다. 로컬 workspace 12,537개와 집합이 다르다.
재시도 목록의 복제 job 기록을 새 worker 재실행이라고 쓰지 않는다.

최초 제출 CI는 합성 시험 글꼴의 repository-only MIT 고지 누락으로 실패했다.
정확한 SHA·출처·고지 등록 뒤 관련 검사 1 FAIL → PASS·전체 8 PASS와 package build를 확인하고
새 head에서 Frontend package gate도 성공했다. 제품 글꼴 38개 집합·바이너리는 바꾸지 않았다.
현재 head의 첫 Lint는 workspace build 중 디스크 1MB로 실패했다. 사용자가 재실행한
[job 126490641968](https://github.com/edwardkim/rhwp/actions/runs/41802090733/job/126490641968)는
Ubuntu 26.04 이미지 `20261025.179.1`에서 18:27:41 KST 성공했다. 실패 이미지는 `20261018.50.1`이며
이미지 차이를 디스크 실패의 확정 원인으로 단정하지 않는다. CI workflow·실행 정책은 수정하지 않았다.

## Merge 후 contributor PR comment 계획

trailing head의 CI·실제 fast-pass 확인 후 일반 squash merge하고 merge SHA와 devel 포함을 확인한다.
병합 뒤에는 duration 갱신의 성공 또는 근거 부족 보류만 확인하며 검증 CI를 시작·재실행하지 않는다.
이 review·오늘할일이 이미 포함되므로 기록용 PR을 다시 만들지 않는다.

운영 기록·devel sync 뒤 원 PR에 한국어 존댓말로 merge SHA, code/trailing CI, 위 전쪽 수치와 남은
130쪽 차이·별도 #8578을 적는다. 실제 주장 페이지인 물리 140·272쪽의 Native/fresh WASM raw·독립
PDF·review·overlay를 merge SHA 고정 raw URL의 Markdown 이미지로 첫 comment부터 표시한다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
게시 전 원본 해시와 이미지 응답·게시 후 API 본문 및 렌더링을 확인한다.
이미 닫힌 #8533에는 같은 merge·증적 comment가 없을 때만 추가하고 #8578은 OPEN을 유지한다.
후속 처리 뒤 소유 임시 branch, 루트 output 전체와 미사용 `target/pr-review/debug`를 정리하며
원본·PDF·글꼴·다른 작업트리·다른 target profile·사용자 7700 서버는 보존한다.
