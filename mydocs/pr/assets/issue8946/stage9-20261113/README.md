# #8946 단계 9 — 문서 충돌 해소 후 기능 보존 검증

검증 기간: 2026-11-13~14 KST. 증적 디렉터리 날짜는 실행 시작일이다.

제품·회귀 producer는 `436b09c7418a6d90b34174c25a149651340ba47c`, 통합 기준은 `47164eb67a8b3526fbba8cf3f3e017656de929ae`다. 검증 중 전진한 devel `dd7f453898776c262c00e862e10f47d22a4d6ceb`도 별도 producer로 빌드해 비교했다. 이후 문서 변경의 source identity는 [범위 감사](source-scope-audit.json)로 구분한다. 원본 manifest의 `git_head`는 측정 도구를 실행한 checkout이다. 실제 renderer 소스는 `renderer_source_sha`와 Native/WASM 산출물 SHA로 별도 연결했으며 서로 바꾸어 표시하지 않는다. 중간 기준 `959300e66c`의 136조합·전쪽 비교도 통과했으며 별도 원장을 보존한다. 이후 #9052의 편집 CLI 기준 `d7b927d073`도 전부 통과했으며, #8922의 PDF 가져오기 CMap 수정 기준을 다시 빌드·측정했다. 11월 13일 사용자 요청으로 중단했던 중간 기준 `6217d3cbed`의 미완료 자료는 최종 판정에서 제외했다. 이 원장은 로컬 검증이며 원격 CI·병합 완료를 뜻하지 않는다.

## 원본 전쪽과 정본 보호

독립 한컴 Print와 같은 외부 그림 3개·실제 글꼴 프로그램을 공급했다. Native/fresh WASM의 각 763쪽은 모두 90% 이상이고 PDF 쪽수가 같다. 최신 devel 대비 결과는 다음과 같다.

| 문서 / backend | 전쪽 수 | 최신 기준 최저 | 후보 최저 | 최대 하락 |
| --- | ---: | ---: | ---: | ---: |
| hwpx / native | 763 | 85.45580% | 96.34886% | 0%p |
| hwpx / wasm | 763 | 85.44688% | 99.56218% | 0%p |
| hwp3 / native | 763 | 98.77212% | 98.77212% | 0%p |
| hwp3 / wasm | 763 | 99.54853% | 99.54853% | 0%p |

[최신 기준 비교](last-devel-full-comparison.json)·[페이지별 TSV](last-devel-full-comparison.tsv)와 [통합 기준 비교](devel-full-comparison.json)를 구분한다. `original/{base,last,head}/`에 전쪽 TSV 원본을 보존했다. 통합 기준과 후보의 HWP3 Native/WASM 전쪽 PNG·배치 트리는 동일하다. 단계 7과 HWP3 Print PNG가 달라진 사실을 숨기지 않았으며 이전 PNG 판독만으로 통과시키지 않았다.

[정본 보호](protected-comparison.json)는 18개 입력의 양쪽 3,542쪽에서 공식 두 자리 표시값 하락 0쪽이다. [편람 Native 383쪽 gate](handbook-gate.json)도 통과했다. 실제 Host는 두 입력 각각 763쪽이며 최저 HWPX 99.36%, HWP3 99.35%다. [직접 판독](direct-reading.json)은 표지·수식/문장부호·표 끝·후속 본문을 같은 독립 Print와 대조한 기록이고 `reviews/`에 실제 비교 PNG를 보존했다. 100% 실루엣을 RGB 동일로 해석하지 않는다.

## 반영한 PR의 실제 기능 보존

#9025·#9033의 원 승인 입력/저장본 56개, CellBreak 8개, footer 1개, #9037 공통 소비 3개를 기준/후보 Native·fresh WASM에서 실행했다. 총 136개 조합·922쪽/소스다. [통합 기준 비교](incoming-comparison.json)는 모든 전쪽 PNG와 배치 트리가 동일하다. [최신 기준 비교](last-incoming-comparison.json)도 누락·90% 미달·2%p 초과 하락 없이 통과했다. 최초 입력·PDF·실제 글꼴 프로그램 SHA를 맞췄고 [출처 원장](visual-provenance.json)에 보존했다.

- 56개 원 승인/저장본: 112조합·852쪽, 후보 최저 90.64672%.
- CellBreak: 16조합·28쪽, 후보 최저 99.50531%.
- footer: 2조합·26쪽, 후보 최저 95.86384%.
- Flutter 공통 입력: 6조합·16쪽, 후보 최저 99.75363%. #8946은 Native/Flutter/FFI binding 파일을 바꾸지 않는다. Windows 장치·플랫폼 수명 검증을 이 Mac에서 다시 실행했다고 주장하지 않는다.

[실제 CLI 비교](api-comparison.json)는 영수증·157쪽 문서 저장/재저장, #8959 편집/저장, 중첩 search/extract, 긴 한글 파일명 출력 9계약을 기준/후보에서 확인했다. [Studio 보존](studio-preservation.json)은 실제 textarea ClipboardEvent를 통한 6조건과 HWP/HWPX 각 2회 저장·재열기(소스마다 24회)의 의미를 보존했다. synthetic 이벤트이며 OS 클립보드 자체를 검증했다고 표시하지 않는다. ByPath 실제 WASM API는 [기준](base8956-wasm-api.json)·[후보](head8956-wasm-api.json) 모두 통과했다. 공통 실제 Canvas suite도 양쪽 19 PASS다.

## 기본 동작과 명시적 환경

[기본 9조건](comparison-summary.json)과 [최신 기준 9조건](last-comparison-summary.json)은 각 문서의 전쪽 PNG·배치 정보를 비교했다. [첫 합성 화면](initial-screen-comparison.json)·[최신 기준 첫 화면](last-initial-screen-comparison.json)은 실제 그림 3개 decode와 DOM 포함 화면을 대조했다. HWP3의 글꼴 준비 후 동일한 2회 출력은 초기 1회 출력과 구분한다.

[편집 관계](edit-verification.json)는 두 원본의 삽입·삭제·저장·재열기 각 8조건을 통과했다. [HWPX 첫 paint](host-hwpx-first-paint.json)·[HWP3 첫 paint](host-hwp3-first-paint.json)에서 실제 program SHA·faceIndex·환경·메트릭 준비 순서를 확인했다. 조판 환경·수명 E2E는 97조건이 모두 통과했다. [실제 브라우저 수신 자원](actual-browser-runtime.json)은 producer WASM과 동일하며 기준·후보·최신 기준의 서로 다른 WASM을 [출처 검사](base-wasm-source-check.json)·[최신 검사](last-wasm-source-check.json)로 확인했다.

## 로컬 게이트와 정정한 환경

[실행 원장](checks-progress.json): 전체 nextest는 13,429개를 실행해 13,428 PASS·1 FAIL이었고 별도 45개는 skip이었다. 유일한 실패는 PDF CLI 기본/명시 SVG 호출의 stderr 비교로, 온라인 글꼴 timeout과 캐시 상태가 달랐다. 코드·기대값을 바꾸지 않고 동일 최소 글꼴 격리, 빈 캐시와 `RHWP_WEBFONT_OFFLINE=1`에서 해당 모듈 2개를 다시 실행해 모두 통과했다. 최초 전체 실행을 무조건 전체 PASS로 표시하지 않는다. [최초 실패·정정 근거](environment-corrections.json)를 함께 보존했다.

Native Skia library 4,034 PASS(12 ignore), placeholder 2·직접 PDF 36·HFT 집중 22 PASS. fmt·Native/WASM/workspace Clippy·workspace build·suite/unit-tier·frontend bindings·E2E manifest 통과. Studio 2,789 PASS(2 skip), TypeScript와 Canvas2D/CanvasKit E2E도 통과했다. 최소 글꼴 격리와 실제 시각 검증 글꼴 환경은 [별도 감사](font-isolation-audit.json)로 구분한다.

#9003의 초기 미달은 HCI Poppy 원본 space 프로그램 누락이었다. 대상 문서와 독립적인 기존 PDF 2개에서 보존한 11개 glyph의 윤곽·전진을 로컬에서만 공급했다. 완전한 원본 프로그램 확보나 제품 보정으로 주장하지 않는다. 원본 PDF xref·SHA·독립 전진 대조는 환경 정정 원장에 있다. 기준 sparse worktree의 PDF 2개 누락으로 실패한 Canvas 검사도 원 commit 바이트 복원 후 19개 전부 재실행했다. 0바이트 임시 글꼴 캐시는 실제 프로그램 집합과 구분하되 원장에서는 제거하지 않았다.

## 독립 PDF 래스터 재사용

최신 기준에서는 기존 독립 PDF 69개·1,986쪽의 파일 SHA, DPI와 PDFium 라이브러리·정책·스크립트 신원을 맞췄다. 각 기준 PNG의 SHA를 확인해 복사했으며 제품 PNG·배치 트리·실루엣 점수는 새 producer로 다시 산출했다. 캐시가 적용되지 않으면 원래 PDF 래스터 경로를 실행한다. [소표본 동일성 검사](reference-cache-qualification.json), [실제 재사용 영수증](reference-raster-reuse.json), [래퍼](reference-raster-cache-wrapper.py)에 근거를 보존했다. 최초 래퍼의 병렬 진입점·청크 기록 위치 오류는 본 검증 투입 전에 보정하고 재검증했으며 최초 로그 해시도 남겼다. 기준 이미지 재사용을 새 PDF 재출력이라고 표시하지 않는다.

## 재현과 자료 보존

표준 실행은 `scripts/visual_sweep.py --silhouette-only --rhwp-bin <source별 CLI> --hwp <원본> --pdf <독립 Print> --compat <기록값> --embed-fonts --font-path <로컬 프로그램> --out <ignored output>`이며 fresh WASM은 `--wasm-pkg <source별 pkg>`를 추가한다. 두 원본의 WASM에는 동일한 `oracle.gif`, `rdb02.gif`, `s1.jpg`도 공급했다. [전쪽 출처](visual-provenance.json)의 원본/PDF/프로그램 SHA·compat·96 DPI를 함께 맞춘다. 빌드는 locked release-test Native Skia와 `scripts/wasm-pack-locked.sh --target web --out-dir pkg`의 최적화 경로다.

[입력 감사](input-commit-audit.json)는 실제 입력/PDF 170개의 commit 바이트와 대용량 PDF 로컬 예외를 확인했다. 763쪽 HWPX 독립 Print(90,804,612바이트)는 기존 단계 4의 SHA와 같고 50 MiB 초과 정책에 따라 로컬에만 있다. [지속 보관 입력 원장](local-reference-preservation.json)에 worktree 정리 후에도 남는 T7 기준 입력의 크기·SHA를 기록했다. 사용자 및 PDF에서 준비한 글꼴 바이너리·WASM/Native 산출물·서비스 인증 자료는 커밋하지 않는다. 최초 실패와 모든 재실행 로그는 [producer](producer.json)의 SHA로 식별하며 정식 원장·TSV·대표 PNG만 보존한다. 원본 전쪽 TSV는 바이트 그대로 복사했고 문서용 비교 집계 TSV의 줄 끝만 LF로 정규화했다.
