# #7751 편람 그림 색상·묶음 글상자 보정 — 2단계

Issue: #7751 · 연계 #7494 · 2026-10-16

1단계 배치 코드 `b8db997a2` 뒤에 진행한다. 같은 원본·383쪽 한컴 Print·설치된
원본5폰트 경로를 유지한다. [1단계](task_m100_7751_stage1.md)의 시각 게이트와
최종 전체 검증·PR 승인 절차는 계속 적용한다.

## 독립 근거와 위반 계약

한컴2024 실제 HWP→HWPX job `23013efe-fbdd-463a-8c3b-e6d6e4653773`은
원본의 그림354를 밝기8·대비0으로 읽는다. rhwp는 밝기0·대비8로 읽었다.
명세 표32의 이름 순서와 실제 저장 순서가 다르다. HWP5 parser/serializer를
대칭 수정해 의미 IR로 정규화하며 HWPX의 `bright`/`contrast`는 그대로 둔다.
원본 TIFF 회색206→실제 Print 내부 이미지226이다. 원본 바이트는 수정하지 않는다.

명시적으로 합성한 RGB11패치×10조건의 한컴2024 Print job
`4ac093ee-057b-455f-afe0-d621b0cdacad`(13.0.0.3901, PrintMethod0,
`frame_print_to_pdf_ex_one_up`)과 부호 교차 job
`114173c0-fd06-4c97-a74d-be2d78670310`에서 확인했다:

- 밝기8: RGB206→226, RGB80/100/120→100/120/140. 인코딩된 RGB에 가산한다.
- 대비20: RGB80/100/120→68/93/118. 양수 대비의 기울기는1/(1-c)다.
- 밝기20·대비20: RGB80/100/120→125/150/175. 음수 대비는 기울기1+c다.
- 밝기·대비 부호 교차8조건×11색의 실제 출력은 아래 일반 선형식과±1 이내다.
  `a = c>=0 ? 1/(1-c) : 1+c`, `d = (1-a)/2 + b*(1+a)/2`,
  `output = floor(clamp(a*input + 255*d,0,255))`(b/c는백분율의비율).
- 회색조·대비50의 빨강255/0/0은24/24/24, RGB80/100/120은64/64/64다.
  회색조를 먼저 정수화한 뒤 대비를 적용한다. 기존 SVG 중첩/linearRGB와는 다르다.
- 끝값 교차 job `ec6b5b8b-56aa-47b8-bd69-df3de7ad6839`은 대비-100에서
  먼저1로, 대비100에서 원본 채널128 초과만255로 양자화한 뒤 밝기를 가산한다.
  중간 범위 선형식의 극한과 달라 별도 분기로 보존한다.

입력 생성 코드·원시 관측·job JSON은 ignored `output/pr-review/pr7494-20261016/`에,
독립 Print는 `pdf/issue7751/`에 보존한다. 이전 SVG 계약 검사는 이 독립 출력과
충돌하는 범위를 고치며, 기존 실행 결과 자체는 보존한다.

## 구현과 반례

1. 공통 raster 색조 helper가 제한된 이미지 decode·RGB 조정·alpha 보존·PNG 결과를
   만든다. Native·Canvas·SVG가 같은 픽셀 결과를 소비한다. 캐시 키에 입력 전체와
   effect/밝기/대비를 모두 포함하고 기존 바이트/항목 예산을 유지한다. 이미 bake된
   워터마크에는 이 helper를 재적용하지 않는다. 이산 흑백/패턴·벡터 fallback·미지원
   Direct PDF 경계는 별도 계약으로 보존한다.
2. KoPub 전용 폭 표가 Unicode 반각 문자를 전각으로 먼저 처리하는 가정을 확인한다.
   `｢｣`는 [Unicode EastAsianWidth](https://www.unicode.org/Public/17.0.0/ucd/EastAsianWidth.txt)의
   H이며 독립 Print에서도0.5em이다. 전용 CJK 표 앞에서 실제 메트릭·공통 반각
   경로로 넘기는 후보를 검증 중이다. 재조판 후 줄 높이가 달라지면 공통 vpos
   재계산기로 뒤 문단을 연결한다. 기존 재조판 적용 범위는 넓히지 않는다.
   한컴 첫 문단 시작0/42/87과3줄, 뒤 문단 순서/간격으로 실제 효과를 판정한다.
   공백 메트릭 override 실험은 첫 문단4줄을 개선하지 못해 제거했다. 해당 실험의
   patch와 출력은 ignored 증적에 보존하며 원인 해결 증거로 세지 않는다.
3. 색상 부호·alpha/object opacity·배경 저장 순서·캐시 매개변수 분리와 기존 거절
   경계를 확인한다. 영향 쪽 출력→전383쪽 Native/fresh WASM→직접 판독을 거친 뒤
   정식 실물 회귀와98% 이상 완료 페이지의±2px 검사를 추가한다.

## 2단계 실행 결과

생산 소스는1단계 `b8db997a29f317fca1785ae1813e80365bac4f7b`와
`output/pr-review/pr7494-20261016/7751-halfwidth-source.patch`의 조합이다.
새 helper는 별도 `7751-halfwidth-picture_adjustments.rs`로도 보존했다.
Native 실행 파일 SHA-256은
`7ee6260dd13eb72a398a9830b7121454f872a4f2f0d5824a271300f12d0a9740`,
fresh WASM은 `e558317eb8c58691695011a001a2c108da3340f5fbea73acb9c72d7bdbfbe1a4`다.

모든 실행은96dpi·compat2024·동일 원본·동일383쪽 Print·1단계의5개 글꼴 경로다.
Native는 `cargo build --profile release-test --features native-skia --bin rhwp`,
fresh WASM은 루트의 `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`로
빌드했다. jobs8과 공용 절대 `target/pr-review`를 사용했고 WASM에만
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`을 지정했다. Docker 미설치 호스트의 진단 경로다.
Native 빌드도 lockfile 변경 없이 끝났으며 최종 lint는 `--locked`로 실행했다.

| 범위 | Native | fresh WASM | 판정 |
| --- | ---: | ---: | --- |
| 전체 쪽수 | 383 | 383 | 독립 Print383쪽과 일치, 누락0 |
| 전쪽 최저값 | 92.24708%(375쪽) | 92.30005%(375쪽) | 90% 미만0 |
| 3쪽 | 100.00000% | 100.00000% | 서문3문단 각각3줄, 앞뒤 순서·배경색 직접 판독 |
| 130쪽 | 99.10104% | 97.33611% | 그림 내부 글자 raster 차이 존재, px 고정 대상에서 제외 |
| 241쪽 | 98.39116% | 98.43857% | 이어받기 끝·제목·다음 표 직접 판독 |
| 289쪽 | 99.74010% | 99.71466% | Q13 종료·Q14 시작 직접 판독 |
| 290쪽 | 99.18776% | 99.17536% | Q14 이어받기·Q15·Q16 시작 직접 판독 |

전쪽 실행의 `summary.json`은 두 경로 모두 `complete`, 프로세스 exit0이다.
silhouette-only의 `not_evaluated`를 승인으로 쓰지 않았다. 별도 대표5쪽 WASM 및
Native3/130쪽·1단계와 출력이 같은241/290쪽의 review·overlay를 직접 열어 확인했다.
대표 비교 실행의 gate는 `passed`다. 3쪽 Direct PDF 생성과 실제 PNG 판독도 성공했다.

검증 기준 `cdef0073bb24512daf0622dec5051ac62cb8d57e`의 생산 코드와 동일한
고정 baseline을 대조했다(실제 생산 SHA와 동등성 근거는1단계 기록).
0.01pp 이상 하락은 Native/fresh WASM 모두74·206·209·297쪽의4개다.
Native 변화는 각각-0.08611/-0.28531/-0.12329/-0.05528pp,
WASM은-0.08462/-0.27003/-0.10005/-0.05252pp다.
네 쪽의 baseline/현재/Print PNG를 열었고 변경된 run을 대조했다.
`｢｣`·`･`의 폭만 달라졌으며 표 외곽·문단/줄 수·내용 순서는 유지한다.
206쪽 `유사･중복`의 x는141.4→143.4px, PDF143.52px,
209쪽은120.8→123.1px, PDF123.04px로 정렬이 개선된다.
74쪽 숫자225의 시작은407.3→397.8px, PDF402.88px로 약5px 잔차가 남는다.
따옴표 자체의 실제 반각 진행 근거를 유지하며 이 숫자 기점의 완전 일치를 주장하지 않는다.
이4쪽을 새로운 px 회귀의 기준으로 사용하지 않는다.

### 증적과 다음 단계

ignored `output/pr-review/pr7494-20261016/` 아래:

- `7751-halfwidth-{native,wasm}-all/handbook/silhouette.tsv`: 전쪽 원본 TSV.
- `7751-halfwidth-{native,wasm}-comparison.json`: 기준 대비 변화 목록.
- `7751-halfwidth-native-review/`, `7751-halfwidth-wasm-review/`: 대표 PNG·overlay·review.
- `7751-halfwidth-{head,base}-drops-review/`: 소폭 하락4쪽의 직접 비교.
- `independent-pdf-border-coordinates.json`: PDF content stream의 실제 괘선 좌표.
- `logs/7751-halfwidth-*.log`: 빌드·전쪽/대표 비교·Direct PDF 실행 로그.
- `logs/7751-final-{native,wasm}-clippy.log`: 양쪽 Clippy `-D warnings`, 각각exit0.

독립 색조 PDF·입력 SHA·관측표는 [pdf/issue7751/README.md](../../pdf/issue7751/README.md)에
연결했다. 전쪽 원본 TSV·중간 JSON·로그는 Git에 추가하지 않는다.

다음 단계에서 기존 색조 검사의 SVG 기대값을 독립 Print의 상수로 대체하고,
`tests/cases/`에 서문·이어받기·통째 표 종료의 내용 관계와 완료 페이지±2px 검사를 추가한다.
수정 전 검출·수정 후 통과, 전체 release-test·Native Skia3종·나머지 lint가 아직 남았다.
현재 기록은 PR 준비 완료나 #7751 종료 선언이 아니다.
