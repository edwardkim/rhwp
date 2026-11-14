# PR #9057 최신 base 병합 후보 검증

2026-11-14 KST. 원 source `f42f34dc45e58c5a4602af0c8df4976d918a9d58`와 최신 base
`88170fd75f9d0ce156fe49d79aeabea9bb3a4bcc`를 검증했다. 임시 producer는
`b6da239caeaf551947a9a6300d25a42f0cd86ab6`이며 회귀 파일 머리말만 보완한 작업 트리다.
보완 code head `3d3bd2b41d28bb0d9f86f45351f74cafd680e0d8`의 실제 base 병합 트리와
검증 트리 `1675188eb5478913195895a17d3b07c997c401f6`는 완전히 같다.
[트리 대조](source-head-bridge.json), [생산 해시·결과](validation.json).

## 전쪽 결과

- 보호 문서 18종 × Native/fresh WASM: 36개 데이터셋, 3,542쪽. 누락·쪽수 불일치·90% 미만·정본 보호값 하락은 없다. 최저 raw는 98.09%다.
- 같은 입력·PDF·글꼴 조건으로 최신 base도 양 경로 3,542쪽을 새로 출력했다. 후보와 공식 두 자리 raw 값은 전쪽 동일하다.
- Native 1,771쪽 PNG는 바이트까지 동일하다. WASM 1,771쪽 중 1,768쪽은 동일하고, 나머지 세 쪽은 각각 경계 픽셀 하나의 명암만 다르다. 최대 채널 차이는 1·9·3이다. 독립 PDF와 3배 확대해 배치·내용 변화가 없음을 확인했다. [픽셀 대조](wasm-pixel-differences.json).
- #9021은 양 경로 각각 독립 PDF와 같은 1쪽이며 base 61.61% → 후보 100.00%다. 두 일반 `pr_review_gate`도 passed다. 첫 줄 표 두 개의 공유 기준선, 둘째 줄 표 한 개, 글상자 내부 포함·비겹침을 직접 판독했다.
- 별도 `--handbook-gate`: HWPX Native 383쪽 passed, 최저 99.96%, 보호 하락 0. [실제 gate](handbook-gate.json).

[보호값·base·후보 전쪽 대조](protected-comparison.tsv), [입력/PDF SHA와 실행별 생산 원장](datasets.json).
`tsv/`에는 base/후보 각각 36개 전쪽 TSV와 대상·추가 문서·편람 TSV를 보존했다.
원문 값과 전쪽 범위를 유지하고 줄 끝만 저장소의 LF로 정규화했다. 생산 TSV와 보존 TSV SHA를 구분했다.
쪽별 최고값을 조합하거나 PDF·허용치·마스크를 바꾸지 않았다.
측정은 96 DPI Print, 독립 한컴 Print PDF의 PDFium raster,
색상 경계 보정 전 2px 내용 실루엣 `silhouette_raw_match_percent`다. raw는 사람의 판독 정확도나 RGB 동일률이 아니다.

## 새 base 기능 보존

검증 중 #9007이 devel에 반영되어 새 base에서 다시 빌드·회귀·전쪽 검증했다.
무늬 HWP 1쪽·HWPX 1쪽·hwpspec 178쪽·59043 37쪽을 양 경로에서 base/후보 모두 출력했다.
[추가 434쪽 대조](incoming-pr9007-comparison.tsv)의 공식 값은 모두 같다.
무늬 두 문서는 Native 99.97%, WASM 99.99%이며 hwpspec은 최저 95.21%다.
무늬 0~5의 방향·격자와 무늬 없음, hwpspec 23쪽 RECORD HEADER와 뒤 내용도 직접 대조했다.
59043의 90% 미만 8쪽(12·14·15·16·20·25·28·37)은 base에도 동일한 기존 [#7778](https://github.com/edwardkim/rhwp/issues/7778) 범위다. 이 문서를 이번 PR의 시각 개선 완료 대상으로 선언하거나 기대값을 바꾸지 않았다.

## 회귀와 빌드 출처

최소 글꼴 환경에서 전체 nextest 13,526개가 통과했다(정책상 45개 제외).
Native lib는 rhwp 3,852개(12 ignored)와 workspace 182개가 통과했고, placeholder 2개·직접 PDF 36개도 통과했다.
fmt, Native/WASM Clippy, workspace build·all-target Clippy, 고정 base suite 정책 검사도 모두 통과했다.
[회귀 집계](regression-results.txt), [명령 결과](full-regression-results.json), [lint](lint-results.json).

공유 target 재사용 중 base 산출물이 후보와 같아진 실행은 해시와 #9021 재현으로 발견해 제외했다.
사용 중인 Cargo 작업이 없음을 확인하고 rhwp 패키지 캐시를 비운 뒤, base의 명시적 manifest·작업 디렉터리로 Native와 최적화 WASM을 다시 빌드했다.
최종 base는 두 경로 모두 수정 전 61.61%를 재현하며 후보와 빌드 해시도 다르다. 이 자료에는 그 최종 base만 사용했다.
base sparse checkout의 CLI 내장 문서 누락은 해당 Git blob 그대로 복원했다. 제품·assertion·허용치를 고친 것이 아니다.

## 글꼴과 직접 판독

사용자가 제공한 글꼴 저장소와 공개 KoPub를 문서별로 공급했다. HFT 프로그램을 한꺼번에 넣을 때의
256개 snapshot 한도를 피하도록 기존 가족·스타일 목록을 사용했다. 독립 PUA 92개, 구형 산술 기호,
삼각형의 원 윤곽·advance 해시를 선행 수용 증거와 대조했고 대상 PDF에서 글꼴을 추출하지 않았다.
[PUA](pua-provenance.json), [삼각형](triangle-provenance.json), [최소 회귀 글꼴 환경](minimal-font-environment.json).
추가 문서의 과거 HDotum 한 프로그램은 확보되지 않았으므로 역사적 글꼴 공급 전체가 같다고 주장하지 않는다.
이번 base/후보는 같은 실제 공급·대체 조건으로 비교했다. [추가 공급 판독](pattern-font-receipt.json).
글꼴 바이너리나 개인 저장 경로는 이 묶음에 포함하지 않는다.

`png/`의 #9021 review·overlay·2배 확대, sample16 HWPX 22·43쪽의 기호·간격,
sample10 81·351쪽의 삼각형·제목·본문, nested 4쪽과 새 base 무늬를 직접 판독했다.
WASM 잔여 세 픽셀의 base/후보/PDF 확대도 함께 보존했다.
