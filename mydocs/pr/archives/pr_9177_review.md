# PR #9177 리뷰 — 문서 원본 5종의 표·개요·글꼴 조판 보정

## 최종 판정

**승인.** #7733·#7735·#7736·#7737·#7738의 다섯 원본에서 전쪽 Native 및 fresh WASM과 코드 head의 원격 CI를 확인했다. 이 기록을 추가한 head의 필수 검사와 병합 가능 상태를 확인한 뒤 `devel`에 병합한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#9177](https://github.com/edwardkim/rhwp/pull/9177) · `jangster77` · `devel` |
| 정책 비교 base | `08b38c12a75fa5cabfe5fbbfde9388aceb3fcca3` |
| 검토 코드 head | `a21feb57083a8b8f05a77af12764d861d1a7b121` |
| 로컬 전체 회귀 소스 | `8771cdcbc48cc324bddc790b32bf8516b52b6379` |
| Native·WASM 제품 생산 소스 | `0fee3421a4ca0fd6e9484b67e4da0b2f4c5e44b0` |
| 리뷰 방식 | Collaborator self-review. 코드 head의 검사 시점 `MERGEABLE / CLEAN` |
| 이슈 범위 | #7733·#7735·#7736·#7737·#7738 해결. 부모 #7445는 남은 서브이슈 추적 |

## 변경과 검토 범위

저장 글줄, 표의 첫 프레임·이어받기·종료 간격, 그림 앵커 뒤 문단의 소속과 글꼴 윤곽·전진을 보정했다. Studio에서는 재감지 결과가 이전 글꼴 프로그램과 섞이지 않도록 하고, 실제 SFNT 이름과 PostScript 이름에 따라 별개 HY·한양 프로그램을 수집한다. 이식한 필수 검사의 범위와 전쪽 대조의 렌더 경로는 [원인·출력 근거](../assets/issues7733_7738_20261117/analysis.md) 및 [리베이스 재사용 감사](../assets/issues7733_7738_20261117/rebase-08b38-rendering-reuse.json)에 기록했다.

최종 후속 보정은 Studio 글꼴 확인 실패 로그의 고정 서식, #7902의 실제 한양 원본 영문명 기대값, 자체 생성 MIT 시험 글꼴의 저장소 전용 라이선스 등록이다. 글꼴 바이너리, 문서 저장 형식, Native/WASM 렌더 소스는 후속 보정에서 바뀌지 않아 아래 전쪽 증적을 재사용했다.

## 검증 입력과 결과

공식 한컴 사양 PDF는 저장소의 원문을 사용했다. 나머지 기준은 데스크톱 한컴의 Hancom PDF Print다. [입력·기준 PDF·원본 SHA·쪽별 비교](../assets/issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole-validation.json) 및 [대표 판독](../assets/issues7733_7738_20261117/final-efe954/README.md)을 연결한다. 다섯 원본 전 252쪽의 후보 최저는 Native 99.69%, fresh WASM 99.67%다. 새 2%p 초과 하락과 쪽수 불일치는 0건이다. 회귀는 글줄과 개체의 소속·순서·존재 관계를 검사한다.

| 원본 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 화학제품 표시 기준 | 103 | 99.75% | 99.71% |
| 한글문서파일형식 5.0 개정 1.3 | 71 | 99.69% | 99.67% |
| 개요 탐색 패널 | 3 | 100.00% | 100.00% |
| 가상융합산업 시행령 | 74 | 99.74% | 99.73% |
| 표 셀 개요 번호 | 1 | 100.00% | 100.00% |

보호 TSV는 다섯 원본 504행을 포함한 총 4,046행·46개 데이터셋에서 하락·누락 0건이다. 편람 HWPX Native 필수 검사는 383쪽 모두 통과하고 최저 99.96503%를 기록했다. sample10 WASM은 원래의 자동 글꼴 환경으로 기준 763쪽 TSV를 SHA-256까지 재현했고 후보도 전쪽 하락 0건이었다. sample16의 세 형식은 고정 검증 목록에 설치 GulimChe 프로그램을 보충한 뒤 base/후보 총 384쪽에서 하락 0건이었다. [보호 검증](../assets/issues7733_7738_20261117/hy-hanyang-font-identity/protected-validation.json) · [편람 Native](../assets/issues7733_7738_20261117/hy-hanyang-font-identity/required-handbook/validation.json) · [정본 등록](../assets/visual-protected-baseline/updates/issues7733-7738-20261118.json)

최소 글꼴 macOS의 전체 회귀는 13,781건 통과·45건 제외였다. Rust 범위별 검증 21단계와 Studio 단위 2,814건을 확인했다. CI 보정 뒤 Native 집중 113건과 프런트엔드 빌드 및 패키지 계약 17건도 통과했다. CoreText가 자체 생성 글꼴을 세션 안에 등록하는 검사는 해당 OS 서비스가 있는 조건으로 실행했다. [Rust 단계별 결과](../assets/issues7733_7738_20261117/hy-hanyang-font-identity/rust-validation.json) · [제품 소스 대조](../assets/issues7733_7738_20261117/hy-hanyang-font-identity/test-only-render-reuse.json)

정확한 코드 head의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44357030546), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44357028618), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44357028789), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44357029015), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44357029575), [CI 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/44362998152)가 모두 성공했다.

## 시각 증적과 남은 차이

2px raw 실루엣 판정과 별도로 대표 그림과 본문의 글리프·줄 소속·누락·중복을 직접 확인했다. 일부 얇은 글리프와 선의 안티앨리어싱 차이는 남아 있다. Studio 실제 출력 45건 가운데 CanvasKit은 4건이었다. CanvasKit 요청 후 Canvas2D로 전환한 26건은 실제 Canvas2D 출력으로 기록했다. [Native 직접 판독](../assets/issues7733_7738_20261117/hy-hanyang-font-identity/native-direct-reading.json) · [WASM 직접 판독](../assets/issues7733_7738_20261117/hy-hanyang-font-identity/wasm-direct-reading.json) · [Studio 실제 소비](../assets/issues7733_7738_20261117/hy-hanyang-font-identity/actual-browser-validation.json)

| 대상 | Native | fresh WASM |
| --- | --- | --- |
| 화학제품 기준 56쪽 | [review](../assets/issues7733_7738_20261117/final-efe954/scope/native/chemical/review-056.png) · [overlay](../assets/issues7733_7738_20261117/final-efe954/scope/native/chemical/overlay-056.png) | [review](../assets/issues7733_7738_20261117/final-efe954/scope/wasm/chemical/review-056.png) · [overlay](../assets/issues7733_7738_20261117/final-efe954/scope/wasm/chemical/overlay-056.png) |
| 사양 70쪽 | [review](../assets/issues7733_7738_20261117/final-efe954/scope/native/spec13/review-070.png) · [overlay](../assets/issues7733_7738_20261117/final-efe954/scope/native/spec13/overlay-070.png) | [review](../assets/issues7733_7738_20261117/final-efe954/scope/wasm/spec13/review-070.png) · [overlay](../assets/issues7733_7738_20261117/final-efe954/scope/wasm/spec13/overlay-070.png) |
| 시행령 70쪽 | [review](../assets/issues7733_7738_20261117/final-efe954/scope/native/decree/review-070.png) · [overlay](../assets/issues7733_7738_20261117/final-efe954/scope/native/decree/overlay-070.png) | [review](../assets/issues7733_7738_20261117/final-efe954/scope/wasm/decree/review-070.png) · [overlay](../assets/issues7733_7738_20261117/final-efe954/scope/wasm/decree/overlay-070.png) |

## 병합 후 처리

정확한 merge SHA의 `devel` 반영과 duration 작업 결과를 확인한다. 해결한 다섯 서브이슈에 병합·비교 증거를 남기고 종료하며, 부모 #7445의 해당 행을 갱신한다.
