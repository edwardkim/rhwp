# PR #8711 메인테이너 검증 증거

검토 원 head는 `57f897a3bd844aadd883d7afb31def4904acb8ca`, 최신 base는 `13775409e57a9550cfb31866d40bdf319ab54bac`다. 제품 검증은 clean merge `a657ac0936e324f9648ab628862062b5b9fddbd8`에서 실행했다. 이후 `4e5a8d1565`에서 테스트 첫머리의 측정 근거 주석을 보완하고, `929f0f5ca5`에서 원자료대로 환경 추정과 래스터라이저 표기를 정정했다. 제품 source와 테스트 본문·assertion·기대값은 같다. [동등성](comment-only-product-equivalence.json), [입력·producer·글꼴 원장](manifest.json)을 따른다. **새 head CI·self-review 후 #8711 MERGED.** [원격 수용·후속 기록](remote/completion.md)을 따른다. 전체 12,782건·Native 공식 4,064건·집중 76건이 통과했다. 마지막 주석 정정 후 모든 Rust lint와 해당 집중 4건도 다시 통과했다. [명령·결과](rust-validation-summary.json), [정정 전후 동일성](metadata-correction-equivalence.json)을 따른다. 기존 source head의 GitHub 성공을 새 head의 성공으로 표시하지 않는다.

## 실제 출력 결과

기준은 원 PR에 포함된 Mac 한컴 Print PDF 1쪽이며 2022 호환·Print profile·96dpi다. 같은 로컬 Windows HANBatang 프로그램을 base/후보 양쪽에 명시했다. 수치는 색상 경계 보정 전 2px 이웃 raw 실루엣이다.

| 입력/출력 | 최신 devel | 후보 | 전쪽 수 |
| --- | --- | --- | --- |
| 원본 Native PNG | 100.00% | 100.00% | 1 / 1 |
| HWP 저장 재열기 Native PNG | 28.46% | 100.00% | 1 / 1 |
| 원본 fresh WASM Canvas | 100.00% | 100.00% | 1 / 1 |
| HWP 저장 재열기 fresh WASM Canvas | 28.94% | 100.00% | 1 / 1 |
| 후보 두 번 저장 후 WASM Canvas | 해당 없음 | 100.00% | 1 / 1 |
| 후보 저장본 실제 Native/WASM SVG | 해당 없음 | 각각 100.00% | 각각 1 / 1 |

[원본 TSV](tsv/)에는 base와 후보를 분리했다. [Native review](figures/candidate-native-saved-review.png), [WASM review](figures/candidate-wasm-saved-review.png), [같은 영역 확대](figures/content-zoom.png)에서 본문 앞 문장 → 표 → 뒤 문장, 머리말·표 셀의 내용과 순서, 본문 잘림 해소를 직접 확인했다. 이 변경 영역에는 수식·첨자가 없다. Mac 기준과 Windows 글꼴 프로그램·래스터 차이에 따른 농도·가장자리 차이는 남으며 raw 100%를 픽셀 완전 일치로 표시하지 않는다.

정식 전체 분석에서도 후보 Native/fresh WASM 모두 1쪽·flagged 0쪽·gate passed다. [실제 상세 지표](repro-visual-metrics.json)는 Native pixel match 99.66377% / 자동 잉크 보조값 29.04142%, WASM 99.62015% / 21.63350%다. 낮은 잉크 보조값을 생략하지 않고 위 확대 PNG에서 glyph 농도·가장자리와 내용·배치를 직접 대조했다. 이 보조값과 raw 실루엣 및 사람의 결론을 구분한다.

## 브라우저 저장과 실제 WASM

[후보 브라우저 실행](candidate-browser-result.json)은 실제 최초 WASM HTTP 응답 `f5bf3679c57cc2aac28e78a587c9b7ab31afe705cad363f644c462ce5d5b65bd`가 고정 package와 같음을 확인했다. `exportHwp()`, `exportHwpWithReport()`, 실제 글꼴 프로그램 2개를 준비한 뒤의 저장 모두 Native CLI와 같은 HWP `729ed729fec4da78c376bcfb5b18eaf12497554b5daabb846ad57e2b35f1bbf2`를 만들고 content-loss count는 0이다. 두 번 저장한 파일은 캐시 등 저장 표현이 달라 byte 동일성을 요구하지 않으며 실제 재열기 내용·쪽수·Canvas 출력이 보존되는지 확인했다.

[기준 브라우저](base-browser-result.json)는 세 저장 경로가 모두 잘린 같은 `cb1fec65…` 파일을 만드는 것을 확인했다. 기준 WASM은 현재 base와 제품 source fingerprint가 같은 이전 최적화 산출물이며 실제 수신 해시 `46ef070e…`로 확인했다. 새 후보 빌드나 메인테이너 실행과 혼동하지 않는다.

`--verify-pages`는 후보 1쪽으로 통과했다. 엄격한 `--verify`는 base·후보 모두 `section[0] paragraph[0] linesegs: count: expected=1 actual=0` 한 건으로 exit 3이다. 기존의 잘못된 저장 줄 캐시 제거 계약에서 생기는 같은 진단을 PASS로 세지 않았다. 본문·표의 보존과 실제 배치는 위 시각 출력과 회귀로 별도 확인한다.

## 새 devel 기능의 보존

최근 #8566·#8595·#8632 입력은 각각 4·5·4쪽이다. 원본·독립 Print는 기존 [입력 원장](../kang-t-20261106/input-provenance.json)의 같은 blob을 사용했다. Print의 유일한 글꼴은 HCRBatang이며, [기존 검증 전용 글꼴 매핑](../kang-t-20261106/print-reference-font-environment.json)을 양쪽에 그대로 적용했다. 로컬 HANBatang/HANDotum 프로그램의 실제 해시는 manifest에 고정하고 프로그램은 재배포하지 않는다. 원본 글꼴을 확보했다고 주장하거나 제품 fallback 정책을 바꾸지 않는다.

| 문서 | Native 전쪽 최저 base = 후보 | WASM 전쪽 최저 base = 후보 |
| --- | --- | --- |
| #8555 | 98.63% | 98.53% |
| #8556 | 98.89% | 98.77% |
| #8557 | 98.73% | 98.60% |

각 backend의 전 13쪽 PNG, 총 [26쌍이 바이트 단위로 같다](latest-feature-preservation.json). 기본 fallback 조건의 선행 진단도 후퇴가 없었지만 최종 표는 Print 매핑 조건으로 다시 산출한 값이다. 같은 쪽·영역·배율의 [#8555 제목/정렬](figures/8555-p1-zoom.png), [#8556 그림 옆 글줄](figures/8556-p1-zoom.png), [34→35번 행 이어받기](figures/8556-p2-zoom.png), [#8557 그림과 9/10번 글줄](figures/8557-p3-zoom.png)을 직접 판독했다. 그림 위치·줄 순서·셀 외곽을 보존하며 추가 누락·중복·겹침이 보이지 않는다.

[실제 브라우저 셀 편집](candidate-browser-control-result.json)은 0/4/0/5/0 경로에 글자 삽입→삭제 후 저장·재열기를 실행했다. 4,442자 본문을 보존하고 base/후보 저장 파일이 같다. [Native 재파싱 관측](cell-edit-preservation.json)에서는 원본과 같은 600/1200 HWPUNIT 기준선 및 정렬 2의 111개 줄 전체를 유지했다. helper 수치만 보고 통과로 판정하지 않았다.

편람 전 383쪽의 [정식 Native gate](handbook-gate.json)는 최저 99.96%·기준 대비 후퇴 0쪽으로 통과했다. [전쪽 TSV](tsv/handbook-native.tsv)를 보존한다.

## 재현과 공개 범위

[재현 코드](reproduction/)는 실제 build producer, 최소 글꼴 wrapper, 브라우저 저장, Print 대조군, 저장 기준선 관측 및 Rust 검증 명령을 보존한다. 로컬 절대 경로는 실행 환경에 맞게 바꾼다. 최소 글꼴 회귀와 독립 Print 글꼴을 명시한 Visual Sweep은 별개 실행이다. [회귀에 실제 노출한 5개 글꼴](minimal-font-inventory.txt)은 라이선스가 확인된 저장소 fallback이며 상용/사용자 글꼴 경로는 숨겼다.

글꼴 프로그램, 내장 글꼴을 포함한 진단 SVG, 실행 로그와 중간 파일은 커밋하지 않는다. 최종 TSV·PNG·producer/입력 해시와 재현 코드는 이 경로에 보존한다. 원 #8226의 체리픽 출처와 협업자 검증은 [기존 기록](../../archives/pr_8226_review.md)이며, 위 메인테이너 실행으로 대체하거나 소급 표시하지 않는다.
