# Dependabot 9건의 결합 검증 증적

기준 `d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에 원 PR
#8457 → #8449 → #8451 → #8453 → #8450 → #8452 → #8454 → #8455와
#8456의 원 기여·최소 lockfile 보정을 적용한
`e58024b15e11438a0e1162a25d836d3c3c4ff7a9`를 검증했다.
원 head별 판정은 상위 `pr_8449_review.md`부터 `pr_8457_review.md`까지 분리한다.
모든 원 head·출처·실제 CI와 충돌 simulation은 다음 자료를 따른다.

- [원 PR·branch·고정 head](source-pr-provenance.json)
- [원 head 최신 CI](source-head-ci.json)
- [독립 merge simulation](merge-simulations.json), [누적 simulation](cumulative-simulation.json)
- [Skia source 정렬과 검증 graph의 동일성](skia-submission-alignment.json)

## 실제 실행과 범위

- [표준 Native·최적화 WASM 빌드](build-results.json)
- [격리된 최소 글꼴의 Rust 명령·종료 코드](rust-gates.json)
- [Studio TypeScript·전체 test·build와 VS Code compile](node-gates.json)
- [실제 Chromium의 AI 설정·패널 검사](browser-results.json)
- [브라우저가 초기화 중 실제 받은 WASM 응답](browser-wasm-consumption.json)
- [명령별 로그 해시·마지막 결과](verification-log-summaries.json)

브라우저는 Chrome 155.0.8059.79의 독립 headless profile과 별도 Vite를 사용했다.
Anthropic 연결·도구 호출·401/400·크레딧 부족 응답은 테스트가 가로챘고 실제 계정이나 유료
API를 호출하지 않았다. WASM 응답 해시는
`0b916be3db137fdce3071d9069dd9669157c4e7ea86bedb78657625728916fbd`와 일치한다.
[소비 기대값](browser-wasm-expectation.json)은 완료된 wrapper 빌드 영수증·producer와 실제
파일에서 작성했다. measurement helper의 배포 manifest나 개별 도구 실행 시간 측정값을
만들어 낸 것으로 표시하지 않는다.

원 head의 적용 범위 GitHub 검사는 PR별로 재사용했다. Skia 보정 뒤 9건을 결합한 그래프의
전체 검증을 한 번 수행했으며, 같은 검사를 각 원 PR에서 9번 수행한 것으로 표시하지 않는다.
새 head push 뒤 GitHub CI의 성공을 대신하는 자료도 아니다.

완료 수량은 [검증 집계](validation-results-summary.json)를 따른다. 전체 nextest는
12,369 PASS/기존 45 skip, Native 라이브러리는 4,040 PASS/기존 12 ignored와 집중 24 PASS,
Studio는 2,716 PASS/기존 2 skip이다. Node의 두 선택 검사는 별도 pkg-node가 없어 제외됐으며
실제 fresh web WASM의 브라우저 동작과 구분했다.

## 최소 글꼴 회귀와 실제 출력 글꼴

회귀는 bubblewrap 파일 시스템 격리 안에서 실행했다. 시스템 글꼴 경로·개인 글꼴 디렉터리와
시각 대조용 글꼴은 노출하지 않았으며 저장소의 공개 fallback 5개만 노출했다.
[실제 프로그램 목록·SHA·격리 명령](font-isolation.json)에 기록했다. 격리 환경의 DNS 경로
누락과 실행 세션 종료 때문에 완료되지 않은 첫 실행은 통과로 세지 않았으며, 환경을 보완한
뒤 성공한 종료 코드를 별도로 수집했다. 제품 코드·검사 기대값을 바꾸지 않았다.

시각 비교에서는 원본이 요구하는 글꼴을 명시적으로 공급했다.
[파일·출처·해시](fidelity-fonts.json)만 보존하며 글꼴 프로그램은 커밋하지 않는다.

## 원본과 독립 기준

모든 입력과 PDF는 기존 기준 devel과 검토 후보에 이미 포함되어 있으며 별도 생성·교체하지
않았다. [경로·바이트 수·Git blob·SHA-256](visual-inputs.json)으로 원본을 확인했다.

| 대상 | 원본 | 독립 Print PDF | 호환 모드 | 전체 쪽 |
| --- | --- | --- | --- | ---: |
| 저장 후 Bottom | [HWP](../../../../tests/fixtures/pr8309-latest/textbox-saved-bottom.hwp) | [2024 Print](../../../../pdf/pr8309-latest/textbox-saved-bottom-2024.pdf) | 2024 | 1 |
| 어울림 정상 대조군 | [HWPX](../../../../samples/issue8399/A.hwpx) | [2024 Print](../../../../pdf/issue8399/A-hwpx-2024.pdf) | 2024 | 1 |
| 저장 LineSeg 없는 글상자 | [HWP](../../../../samples/issue8122/textbox_valign_no_lineseg.hwp) | [WebHWP Print](../../../../pdf/issue8122/textbox_valign_no_lineseg-webhwp-print.pdf) | 2022 | 1 |
| WMF 고정 face·Dx | [HWPX](../../../../samples/issue8160/wmf-fixed-face-dx-independent.hwpx) | [2020 Print](../../../../pdf/issue8160/wmf-fixed-face-dx-independent-2020.pdf) | 2022 | 2 |

2020·2022 계열 Print의 RHWP 실행 호환 모드는 정본에 따라 `--compat 2022`다.
이전 #8309에서 완료한 저장 정렬은 첫 원본의 실제 저장본과 정식 focused로 함께 보존을 확인했다.

## 전쪽 Native와 fresh WASM

96 DPI에서 각 문서의 모든 페이지를 출력하고 색상 경계 보정 전 2px 내용 실루엣의
`silhouette_raw_match_percent`를 비교했다. 통상 pixel match와 raw 실루엣은 구분한다.
[기준·후보 전쪽 통합 TSV](base-candidate-pages.tsv), `tsv/base/`, `tsv/candidate/`에
원본 TSV를 보존했다. 두 backend에서 누락·쪽수 불일치·90% 미만·2%p 초과 하락은 없다.

| 대상·쪽 | 기준 Native | 후보 Native | 기준 WASM | 후보 WASM | 하락폭 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 저장 후 Bottom 1쪽 | 100.00% | 100.00% | 100.00% | 100.00% | 0.00%p |
| 어울림 A 1쪽 | 100.00% | 100.00% | 100.00% | 100.00% | 0.00%p |
| 이전 글상자 1쪽 | 99.55% | 99.55% | 99.42% | 99.42% | 0.00%p |
| WMF 1쪽 | 94.40% | 94.40% | 98.77% | 98.77% | 0.00%p |
| WMF 2쪽 | 94.59% | 94.59% | 98.77% | 98.77% | 0.00%p |

기준/후보 RGBA의 전체 픽셀도 [Native](native-pixel-preservation.json)와
[WASM](wasm-pixel-preservation.json) 각각 전쪽 동일하다. 이는 독립 Print와 100% 동일하다는
주장이 아니다. [직접 확대 판독](visual-direct-reading.json)과 측정 결과를 분리했다.

- [Native producer](candidate-native-producer.json), [fresh WASM producer](candidate-wasm-producer.json)
- [기준 Native producer와 source 동일성](baseline-producer.json)
- [기준 WASM의 실제 source·lock snapshot](base-wasm-producer.json)
- [Native 실행 명령](visual-candidate-native-normal-results.json), [WASM 실행 명령](visual-candidate-wasm-normal-results.json)

기준 Native는 이전에 검증한 실행 파일을 해시로 고정했으며 runtime source/manifest/lock가
현재 기준 devel과 같음을 확인했다. 기준 WASM은 별도 output package에 새로 최적화 빌드했다.
그때 Git HEAD는 검토 branch였지만 runtime 파일과 Cargo.lock snapshot이 기준 devel과 같은지
검사했다. Git HEAD를 기준 devel로 checkout한 빌드라고 바꾸어 적지 않았다. 후보 실행 전 lock을
복원했고 사용자 기본 Studio의 JS/WASM을 이 후보로 바꾸지 않았다.

각 실행의 `runs/` manifest도 보존한다. 그 안의 `provenance.git_head`는 Sweep을 실행한
기본 작업공간의 도구 source SHA이고, 별도 실행 파일·WASM package의 빌드 source SHA는
위 producer에 기록한 값이다. 실제 CLI/WASM 파일 해시를 함께 대조했으며 manifest의 기존
값을 후보 SHA로 덮어쓰지 않았다. PNG backend는 Native Skia와 fresh WASM Canvas2D다.
새로 빌드한 JS glue는 기준/후보 모두 같았다. wrapper가 소유한 검증 worktree의 public glue를
동기화한 변경은 제출 diff에서 제외했고, 검증 종료 후 원 상태로 복원했다.
[glue 동일성 기록](generated-glue-review.json)을 참조한다.

## 직접 판독과 남은 차이

확대 영역은 진단용 crop일 뿐 회귀 assertion의 절대 좌표나 새 fixture/golden이 아니다.
같은 영역을 2배로 읽고 글상자 내부 소유·문자 순서·그림과 앞뒤 본문·줄 간격을 확인했다.
대상에 수식은 없다. 모든 페이지의 원 PNG·비교·overlay와 crop은 아래 경로에 보존한다.

- `png/base/{native,wasm}/`, `png/candidate/{native,wasm}/`, `png/independent-print/`
- `png/compare/{native,wasm}/`, `png/overlay/{native,wasm}/`
- `review-crops/`, [crop 위치와 배율](review-crop-regions.json)

이전 글상자의 획 두께·일부 세로 위치, Native WMF의 face·굵기와 1쪽 얇은 삼각형은 Print와
차이가 남는다. 동일한 source·글꼴·좌표의 기준 devel 출력에서도 같은 차이를 직접 확인했으며
후보 픽셀은 그대로다. 이 의존성 업데이트를 해당 문서들의 피델리티 수정 완료로 표시하지
않는다. 새로운 배치 규칙·예외·좌표 보정·기대값 완화는 추가하지 않았다.

## 공급망 검토

변경한 npm 패키지 3개의 NPMScan 결과는 `pr-8449-npmscan.json`,
`pr-8451-npmscan.json`, `pr-8453-npmscan.json`과 scope 파일을 따른다.
변경 lock entry만 비교한 감사이고 전체 npm 그래프 감사가 아니다. 3개 모두 전이 delta와 새
설치 스크립트·출처 교체·identity mismatch·보고된 취약점은 없다.

[Cargo 정확 버전 OSV 응답](cargo-osv-response.json)은 변경·추가된 모든 버전을 포함한
15개 버전을 조회해 advisory ID 0건이었다. C/C++ 내장 라이브러리를 포함한 전체 보안 감사로
확대 해석하지 않는다. [실제 Cargo delta](cargo-delta.json)를 함께 보존한다.

[Skia release](upstream-skia-release.json), [install-action compare](upstream-install-action-compare.json)
및 [primary source 해시](upstream-sources.json)를 확인했다. npm 3건의 CodeQL NEUTRAL은
scoped 실행에 없는 Python/Rust 설정 집계 메시지이며 annotation은 0건이다. JS/actions와
CI Impact Policy 성공을 확인했고 alert 분류나 보안 설정은 바꾸지 않았다.
