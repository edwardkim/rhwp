# #8856 최종 렌더링 보정·검증

## 판정과 고정 조건

소스·회귀 후보 `e0ce30729e4d87d2a75376d17c6d351a29ef4345`, devel `3170fddc48cc4fcac0c18761d146b619d0f90c7b`의 제출 전 검증을 완료했다.
이 파일·이미지를 추가하는 문서 commit은 제품 Runtime을 변경하지 않는다. 원래 빌드 parent,
실제 producer·실행 source, 제출 전 Runtime 2,634개 파일 동일성은
[producer](committed-source-equivalence.json)·[현재 동일성](current-runtime-equivalence.json)·
[문서-only 리베이스](rebase-equivalence.json)에 구분했다.

최소 글꼴 sandbox에서 개인·공유 한컴 글꼴과 CoreText 등록 서비스를 차단했다. 현재 CoreText의
528개 face는 모두 OS 기본 경로였고 비OS는0이다([현재 목록](coretext-current.json),
[실제 sandbox](minimal-font.sb)). 공개 웹폰트·오픈 소스 글꼴은 명시한 공급 환경으로 사용했다.
원본 글꼴/독립 Print 공급 대조는 이 최소 환경의 통과 수치에 섞지 않았다. 비공개 글꼴은 커밋하지 않았다.

## 완료한 검사

- 최소 글꼴 전체 nextest **13,153 PASS /0 FAIL /45 skip**,334.650초.
- fmt·Native/WASM32/Native-feature/workspace all-target Clippy·workspace build PASS.
- Native Skia root3852 PASS/12ignore, 다른 libs182 PASS; 실제 Native CI 모듈22 PASS; PNG2/PDF35 PASS.
- manifest·unit tiers·Node 글꼴·실제 브라우저 그림 계약 PASS.
- Studio TypeScript PASS, unit2784 PASS/2skip; 최종 실제 screen/print 소비는 아래 별도 기록.
- Mac fresh WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  Docker 최적화 통과로 보고하지 않는다.

[게이트 결과](final-gates.json)는 첫 전체 시도의 두 SVG 조회 실패와 수정 뒤 전체 통과를 구분한다.
원 해결 커밋·독립 PDF·전쪽100%·기존 ±2px/본문 존재 보호와 검사 변경 이유는
[stage15](../stage15-svg-note-locators/README.md)에 있다. 실패 검사를 삭제하거나 판정 문턱을 완화하지 않았다.

## 전체 페이지·devel 대조

PDFium96dpi·호환2022/2024·raw2px·capture8 조건을 유지했다. 각 Native/fresh WASM은
9문서군219쪽, 합계438쪽이며 누락·중복·쪽수 불일치·90% 미만은0이다.
실제 입력·PDF의 commit 내용과 실행 SHA를 [대조](input-audit.json)했다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 전쪽 원본 |
| --- | ---: | ---: | ---: | --- |
| `interview` | 2 | 99.23307% | 99.25469% | [Native](whole/native/interview/silhouette.tsv) · [WASM](whole/wasm/interview/silhouette.tsv) |
| `request` | 1 | 99.22562% | 99.20457% | [Native](whole/native/request/silhouette.tsv) · [WASM](whole/wasm/request/silhouette.tsv) |
| `newyear` | 3 | 98.42814% | 98.42806% | [Native](whole/native/newyear/silhouette.tsv) · [WASM](whole/wasm/newyear/silhouette.tsv) |
| `ktx` | 1 | 99.66127% | 99.69700% | [Native](whole/native/ktx/silhouette.tsv) · [WASM](whole/wasm/ktx/silhouette.tsv) |
| `policy-flags0` | 1 | 99.99813% | 99.99259% | [Native](whole/native/policy-flags0/silhouette.tsv) · [WASM](whole/wasm/policy-flags0/silhouette.tsv) |
| `legislation-summary` | 10 | 99.23795% | 99.22090% | [Native](whole/native/legislation-summary/silhouette.tsv) · [WASM](whole/wasm/legislation-summary/silhouette.tsv) |
| `report-628` | 195 | 94.83952% | 94.84611% | [Native](whole/native/report-628/silhouette.tsv) · [WASM](whole/wasm/report-628/silhouette.tsv) |
| `idol-counterexample` | 2 | 99.50829% | 99.50802% | [Native](whole/native/idol-counterexample/silhouette.tsv) · [WASM](whole/wasm/idol-counterexample/silhouette.tsv) |
| `clip5587` | 4 | 92.32561% | 92.15037% | [Native](whole/native/clip5587/silhouette.tsv) · [WASM](whole/wasm/clip5587/silhouette.tsv) |

같은 입력·기준 PDF PNG·글꼴 공급·profile의 fixed devel과 원래 JSON 정밀도로 비교했다.
최대 하락은0.04934%p이고2%p 초과는0이다. 두 자리 반올림은 일반 devel 하락 계산에 쓰지 않았다.
새 devel3170fddc48은 문서·보안 종료 기록뿐이며 fixed baseb5064fae와 Runtime·검증 입력이 같다.
[원래 raw 대조 TSV](base-head-raw-comparison.tsv)·[조건 감사](base-head-comparison.json)·
[fixed base 빌드](fixed-base-build-state.json)와 base/후보 전체 TSV를 함께 보존했다.

별도 최소 공개 웹폰트 편람 HWPX Native383쪽 gate도 PASS다.
[편람 판정](handbook-gate.json)·[383쪽 TSV](handbook-native.tsv).

## 정본 보호 TSV와 외부 그림 조건

정본의 공식 두 자리 표시값을 기준으로 **36데이터셋3,542쪽 하락0**을 확인했다.
[전쪽 판정](protected-validation.json)에는 각 dataset의 TSV·원래 정밀도·producer·글꼴 환경이 있다.
글꼴 파일 목록과 환경 JSON은 해시를 고정했고 바이너리는 넣지 않았다.

sample10 Native는 입력 폴더의 `oracle.gif`, `rdb02.gif`, `s1.jpg`를 자동으로 공급한다.
처음 임시 WASM 명령에는 이 파일 공급이 빠져1쪽31.60283%가 나왔다. 이 실행은 입력 조건 오류로
보존하고 제출 근거에서 제외했다. 기존 `--linked-image` 옵션으로 **같은 tracked 원본3개·SHA**를
주입하여 WASM763쪽을 다시 산출했다. 1쪽은99.96385%이며 전체 공식 보호 값이 유지됐다.
빈 unresolved link의 decode 억제는 그림을 숨기는 조건이 아니다. 공급된 유효 그림은 실제 출력한다.
[그림 공급](sample10-linked-assets.json)·[같은1쪽 raw](sample10-linked-assets/raw-001.png)·
[review](sample10-linked-assets/review-001.png)·[overlay](sample10-linked-assets/overlay-001.png).
PDF·비교 영역·문턱은 바꾸지 않았다.

## 원인별 실제 보정과 선행 해결

- [stage1](../stage1-distributed-forms/): 마지막 inline 양식의 분산 공간·선택지 누락.
- [stage2](../stage2-table-circles/): 문단용과 다른 표 원형 점선 pen.
- [stage3](../stage3-legacy-shadow/): 이전 인쇄 세대의 연속 흰 그림자와 공급 프로그램.
- [stage4](../stage4-policy-fields/): flags0 재조판의 기호 전진과 하단 줄 소속.
- [stage5·7·8·9](../stage5-soft-edge-source/): 그림 효과 원시 속성/저장·공통 트리·마스크·2024 Print 평면.
- [stage6](../stage6-replacement-symbol-cell/): 명시 대체 기호의 잉크 폭과 소스 전진 분리.
- [stage10](../stage10-hft-bullet-advance/README.md): HFT 글머리표 전각 칸과 뒤 본문의 소속.
- [stage11](../stage11-newyear-source-contract/README.md): 한양해서 원본 공백과 캡션의 그림 마커.
- [stage13](../stage13-note-reference-paint-cell/README.md)·
  [stage14](../stage14-note-reference-source-style/README.md): 참조 문자군·크기/장평·공통 전진 배열과 실제 소비.

원 해결 PR/실제 통합 커밋·독립 한컴 Print·원시 속성·적용/비적용 경로·실제 FAIL→PASS는 각 단계에 있다.
문서명·쪽 번호의 좌표 예외를 추가하지 않았다. 줄 소속·내용 순서·누락·중복·칸의 비겹침과
정상 대조군을 유지했다. 98% 미만인 report95쪽의 새 배치 검사는 관계형이다.

## 직접 판독과 글꼴 조건의 구분

정책연구 하단의 `포상금 / 지급(` 경계, request의 홈쇼핑·기타, 제목 연속 그림자·사진 외곽,
설날 그림 마름모와 뒤 본문, 표 원형 점, report95·96의 참조와110쪽 bullet 뒤 본문을 같은 쪽
Native/fresh WASM review·overlay로 확인했다. 각 원본·페이지의 raw와 PDF도 focus 폴더에 있다.

최소 공개 글꼴에서는 실제 대체 glyph 윤곽이 원본 PDF와 다를 수 있다. 이를 AA만의 차이나
RGB 완전 일치로 표시하지 않는다. report95쪽의 최소97.64481/97.64831%와 같은 Print Gulim 공급
99.99487%는 서로 다른 조건이다. 원본 언어 슬롯·전진 칸의 결함은 보정했고 현재 Print 공급 대조에서
원점·글리프·앞뒤 내용까지 확인했다. clip5587의 기존0.14171%p 하락도 정확한 원 Print 조건을
복원해 별도4쪽 전쪽 대조했다. legislation의 명시 Haansoft 대조도 최소 조건과 분리했다.
[원본 글꼴 대조](original-control-state.json)·[뒤배경 표 전쪽](strict-clip-whole-state.json)·
[독립 Print 대체 전쪽](paired-legislation-whole-state.json).

## 실제 소비 경로

[실제 소비 판정](actual-consumers/validation.json)에 Studio screen/print·Canvas2D·CanvasKit software의
요청 backend와 실제 backend, fallback guard, 같은 쪽 이미지와 초기화 응답 JS/WASM hash를 구분했다.
참조95쪽의 CanvasKit 요청은 `canvaskitResourcePreparationFailed`로 Canvas2D에 내려갔다.
이를 각주의 CanvasKit 지원 통과로 바꾸지 않았다. 기존 미지원 guard도 변경하지 않았다. Native 실제 PNG는 whole의 Native 원본이며
SVG raster를 Native Skia나 Canvas2D로 표시하지 않았다.

초기 HTTP WASM과 root pkg/Studio public/frozen WASM을 해시로 대조했다. Vite의 JS 응답은
소스맵과 `/@fs/` WASM URL 변환을 구분해 검사했다. 설치 폰트·원본 공급 자료는 저장소에 추가하지 않았다.

## 알려진 한계

raw2px100%는 RGB100%가 아니다. 공개 대체 글꼴과 원본 글꼴은 구분하며, glyph가 다른 최소 조건의
점수를 원본 조건100%로 바꾸지 않았다. 이 PR은 명시한 렌더링 계약을 보정하고 실제 전쪽 gate를
통과한 것이며 모든 글꼴·모든 인쇄 장치가 픽셀 단위로 같다는 보장은 아니다.
