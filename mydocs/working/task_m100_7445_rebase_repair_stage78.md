# Stage78: 공통 Print 종이 표본 질의와 실제 표시 소비 연결

Issue: #7445. 부모 HEAD `3a09d0df4b01297d101abc6729939f6fcde29220`, base `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 기존 대상만 사용했다. 이 단계는 중간 보정이며 PR 제출 준비 완료가 아니다.

## 공통 producer와 소비

Print는 정수 point 종이를 만들지만 편집용 PageInfo는 원본 크기다. CanvasView가 원본 크기만으로 장치 표본 소유를 검사해 DPR 1.5의 완성된 Print canvas를 축소 표면으로 오인하고 다시 보간했다. 기존 `PagePaintContext::output_size`를 `getPageInfo`의 선택적 `printSize`로 연결했다. 별도 레이어 트리·그림·글꼴을 만드는 질의가 아니다. 원본 width/height·안여백과 편집 정보는 유지한다.

`PagePaintContext → PageInfo.printSize → CanvasView의 native 표본 판정 → CSS raster filter`가 같은 종이를 소비한다. 낮은 해상도 예산·zoom preview·DOM 그림의 기존 보간은 유지한다. 원본 페이지 목록·CSS 크기·스크롤 배치를 바꾸지 않는다. Native PNG와 Print 레이어 트리 출력 크기도 바꾸지 않는다.

## 검증

- 기존 Native Print 상태 보존 검사에 PageInfo.printSize와 실제 Print layer tree 크기의 일치를 추가해 실제 1 PASS다. 함수·문서 fixture를 늘리지 않았다.
- 실제 refreshPages 생명주기를 사용하는 관계 검사는 auto/crisp-edges의 실제 1 FAIL → PASS이며, cache·scroll·zoom scheduler 묶음은 71 PASS다. source PageInfo 불변성과 Screen·실제 축소 표면도 확인했다.
- fresh WASM은 동일한 두 Rust source hash로 display-paper-contract 단계에서 만든 locked wrapper `--target web --out-dir pkg --no-opt` 산출물을 재사용했다. 원본·복사 pkg·Studio public 바이트 hash를 고정했고 실제 초기화 JS 응답은 Vite 변환을 검증했다. Docker 최적화 통과가 아니다.
- 현재 Q 후보 TypeScript exit 0. Studio 전체 2,411건 중 2,409 PASS / 0 FAIL / 2 skip, 10.039초다.
- 실제 Canvas2D 54건·지원 CanvasKit 8건의 요청 backend·직접 재생·그림 준비를 확인했다. 실제 화면 62건 실패 0이다. 개인정보 preflight fallback 등 지원 미달을 포함해 통과시킨 결과가 아니다.
- 독립 Print 31건은 모두 90% 이상이며 앞선 동일 소비 조건 대비 2%p 이상 하락 0건이다. 로드맵 153쪽의 DPR 1.5는 89.22797%에서 96.10150%로 개선됐다. review/overlay를 직접 판독했다.
- 기존 4개 입력의 확대·DPR 전환/복원 20건 중 19건은 90% 이상이다. 저장 줄 결측 CanvasKit 대조군의 zoom 1.25 / DPR 1.5는 86.78369%로 미완료다. 내용·표 소속과 복원은 보존됐으나 가로선의 확대 위상 차이가 남는다. 다른 쪽의 통과로 상쇄하지 않는다.

## 기각한 후보와 자체 오류

Print 크기로 원본 페이지 목록을 교체한 초기 후보와 표면 CSS만 Print 크기로 바꾼 후보는 모두 정식 96 DPI Print 5건을 악화시켜 기각했다. CSS 후보의 확대 4건도 논리 크기 불변식에서 실패했다. object-fit contain 후보 역시 확대 대조군 87.19421%로 미달이어서 기각했다. 최종 source는 위 Q 후보로 복원하고 5개 hash를 재확인했다. 기각한 source·PNG·수치·로그는 ignored output에 보존했다.

초기 준비 스크립트가 Studio cwd를 저장소 루트로 오인해 파일 복사 전에 종료했고 다음 name filter는 0건이었다. 이를 증거로 쓰지 않고 절대 경로로 고친 실제 1 FAIL / 71 PASS를 기록했다. 관측 스크립트의 exit 0과 내부 실패를 구분하도록 후속 드라이버도 강화했다.

## 제출 상태

현재 base로의 382개 commit rebase와 Cargo.lock 충돌 보존은 rebase-d9fd85ed-20261023 로그에 있다. 최신 HEAD의 전체 회귀·52조건 Native/fresh WASM 전쪽·패키지/범위 gate는 최종 수용 전 계속 필요하다. sample16 HWP 18·35쪽과 기존 bitmap 대조군 미달도 남아 있다. 미검증을 CI로 넘기지 않으며 새 HWP 보정 대상으로 확대하지 않는다.

[기계 결과와 대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/print-raster-ownership/README.md). 원시 로그·기각 후보와 중간 산출물은 ignored `output/pr-review/7445-completed-20261009/rebase-d9fd85ed-20261023/`에만 보존했다.
