# Contributing to rhwp

rhwp에 관심을 가져주셔서 감사합니다!

"모두의 한글"은 이름 그대로 모두의 참여로 완성됩니다. 코드 기여, 버그 리포트, 문서 개선, HWP 샘플 파일 제공 — 어떤 형태든 환영합니다.

## 처음 참여하시나요?

### 1. 프로젝트 체험하기

코드를 보기 전에 먼저 사용해보세요:

- **[온라인 데모](https://edwardkim.github.io/rhwp/)** — 브라우저에서 바로 HWP 파일 열기
- **[VS Code 확장](https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode)** — VS Code에서 HWP 미리보기
- **[npm 패키지](https://www.npmjs.com/package/@rhwp/editor)** — 3줄로 HWP 에디터 임베드

### 2. 개발 환경 설정 (5분)

```bash
# 클론
git clone https://github.com/edwardkim/rhwp.git
cd rhwp

# 빌드 + 테스트
cargo build
cargo test

# 웹 에디터 실행 (선택)
cd rhwp-studio
npm install
npx vite --port 7700
# http://localhost:7700 에서 확인
```

### 3. 첫 기여 찾기

- [`good first issue`](https://github.com/edwardkim/rhwp/labels/good%20first%20issue) 라벨이 붙은 이슈
- 렌더링 불일치 제보 (한컴과 비교하여 스크린샷 첨부)
- 문서 오타/개선
- [Discussions](https://github.com/edwardkim/rhwp/discussions)에서 질문/아이디어 제안

## 기여 방법

### 버그 리포트

HWP 파일이 한컴과 다르게 렌더링되면 알려주세요:

1. [이슈 생성](https://github.com/edwardkim/rhwp/issues/new?template=bug_report.md)
2. **한컴 스크린샷** + **rhwp 스크린샷** 비교 첨부
3. 가능하면 HWP 파일 첨부 (개인정보 제거 후)

디버깅 정보를 함께 제공하면 수정이 빨라집니다 (아래 "디버깅 가이드" 참고).

### 코드 기여 — Fork & PR 워크플로우

컨트리뷰터는 **Fork 기반**으로 작업합니다. 저장소에 직접 push할 수 없으며, PR을 통해 코드를 제출합니다.

```
[본인 Fork]                              [edwardkim/rhwp]

1. Fork (GitHub UI)
   edwardkim/rhwp → myid/rhwp

2. Clone
   git clone https://github.com/myid/rhwp.git
   cd rhwp

3. 브랜치 생성 + 작업
   git checkout -b fix/issue-123
   (코드 수정 + 테스트)

4. Push (본인 Fork에)
   git push origin fix/issue-123

5. PR 생성 (GitHub UI)                   ──→ devel 브랜치로 PR
                                              CI 자동 실행 (빌드+테스트+Clippy)
                                              메인테이너 코드 리뷰
                                              승인 후 merge
```

**중요:**
- PR 대상 브랜치는 **`devel`** 입니다 (`main` 아님)
- PR을 생성하면 CI가 자동으로 빌드 + 테스트 + Clippy를 실행합니다
- CI가 통과하지 않으면 merge할 수 없습니다
- 메인테이너의 코드 리뷰 승인 후 merge됩니다

### PR 전 체크리스트

```bash
cargo test                                       # 793+ 테스트 통과
cargo clippy --all-targets --all-features        # native-skia까지 확인하려면 fontconfig/freetype 개발 패키지가 필요할 수 있음
```

두 명령이 모두 통과하는지 확인한 후 PR을 생성해주세요.

### HWP 샘플 파일 제공

다양한 HWP 파일로 테스트할수록 렌더링 품질이 올라갑니다. 개인정보가 없는 공공 문서나 테스트용 파일을 제공해주시면 큰 도움이 됩니다.

## 브랜치 규칙

| 브랜치 | 용도 | 보호 규칙 |
|--------|------|----------|
| `main` | 릴리즈 (안정 버전) | PR 필수 + CI 통과 + 리뷰 1명 |
| `devel` | 개발 통합 (PR 대상) | CI 통과 필수 |

- 컨트리뷰터 PR → `devel`
- 릴리즈 시 `devel` → `main` + 태그

## 디버깅 가이드

렌더링 버그를 조사할 때 코드 수정 없이 사용할 수 있는 3종 도구:

```bash
# 1. 문단/표 식별 (디버그 오버레이)
cargo run --bin rhwp -- export-svg sample.hwp --debug-overlay

# 2. 페이지 배치 목록
cargo run --bin rhwp -- dump-pages sample.hwp -p 3

# 3. 특정 문단 상세 (ParaShape, LINE_SEG, 표 속성)
cargo run --bin rhwp -- dump sample.hwp -s 0 -p 45
```

`export-svg`의 기본 출력 폴더는 `output/`입니다. 예를 들어 `sample.hwp`를 한 페이지 문서로 내보내면 `output/sample.svg`, 여러 페이지면 `output/sample_001.svg`처럼 저장됩니다. `-o`를 사용하면 다른 폴더로 보낼 수 있습니다.

### 렌더러 비교 가이드

현재 렌더러 경로는 아래처럼 나뉩니다.

- **Legacy SVG**: 기본 `cargo run --bin rhwp -- export-svg sample.hwp`
- **Layer SVG**: `RHWP_RENDER_PATH=layer-svg cargo run --bin rhwp -- export-svg sample.hwp`
- **Native Skia PNG**: 테스트 경로에서 `render_page_png_native()`로 검증되며, 현재 별도 `export-png` CLI는 없습니다
- **Browser Canvas2D / CanvasKit**: `rhwp-studio`에서 기본은 Canvas2D, `http://localhost:7700/?renderer=canvaskit`로 CanvasKit 비교
  - CanvasKit 래스터 모드: `?canvaskitMode=compat`(기본, Canvas2D 유사도 우선) 또는 `?canvaskitMode=default`(CanvasKit 기본 동작)

SVG를 직접 비교하려면 보통 아래처럼 두 번 내보냅니다.

```bash
cargo run --bin rhwp -- export-svg sample.hwp -o output/legacy
RHWP_RENDER_PATH=layer-svg cargo run --bin rhwp -- export-svg sample.hwp -o output/layer
```

자동 회귀 테스트는 다음 명령을 사용합니다.

```bash
cargo test layer_svg --lib
RUSTFLAGS='-L native=target/native-libs' cargo test skia --lib --features native-skia

cd rhwp-studio
npm run e2e                           # 기본: host Chrome CDP 모드, CanvasKit compat/default 둘 다 실행
node e2e/text-flow.test.mjs --mode=headless && node e2e/canvaskit-render.test.mjs --mode=headless && RHWP_CANVASKIT_MODE=default node e2e/canvaskit-render.test.mjs --mode=headless
```

WSL/CI처럼 호스트 Chrome CDP가 없는 환경에서는 `npm run e2e` 대신 `--mode=headless` 명령을 사용하세요.

비교 아티팩트는 아래 위치에 남습니다.

- `output/layer-svg-diff/` — legacy SVG vs layer SVG
- `output/skia-diff/` — layer SVG vs native Skia PNG
- `output/e2e/` 및 `rhwp-studio/e2e/screenshots/` — 브라우저 Canvas2D vs CanvasKit

- `layer-svg` 비교는 현재 exact match 기준입니다. 한 픽셀이라도 diff가 생기면 테스트가 실패합니다.
- `native-skia` / `CanvasKit` 비교는 exact diff를 계속 저장하고, 별도로 채널 차이가 `8` 이하인 픽셀을 무시한 tolerant diff를 계산합니다.
- `native-skia`는 추가로 `1px neighborhood`를 고려한 `raster-tolerant diff`와, 거의 흰색인 안티앨리어싱 커버리지를 접고 실제 잉크 모양만 비교하는 `ink-mask diff`도 계산합니다.
- 테스트 통과 여부는 tolerant 계열 기준으로 판단합니다. 현재 기준은 `native-skia`는 `ink-mask diff ratio 0.30%` 이하, `CanvasKit`은 tolerant diff ratio `0.25%` 이하입니다.
- `native-skia`의 `ink-mask diff`는 `white delta 25`, `alpha threshold 8`, `neighborhood radius 1px` 기준입니다. exact/raw tolerant/raster-tolerant 아티팩트는 계속 저장하고, 최종 실패 여부만 실제 잉크 모양 차이 기준으로 판정합니다.
- `CanvasKit` e2e는 기본적으로 전체 페이지를 비교합니다. `eq-01`도 다시 전체 페이지 회귀에 포함됩니다.
- 추가로 `equation`처럼 특정 op 자체를 분리해서 추적하고 싶은 기능 회귀는 해당 `layer op` bbox만 잘라서 비교합니다.

디버그 오버레이는 문단/표에 라벨을 표시합니다:
- 문단: `s{섹션}:pi={인덱스} y={좌표}`
- 표: `s{섹션}:pi={인덱스} ci={컨트롤} {행}x{열} y={좌표}`

이 정보를 이슈에 첨부하면 버그 수정이 빨라집니다.

## 프로젝트 구조

```
src/
├── model/          ← 순수 데이터 구조 (의존성 없음)
├── parser/         ← HWP/HWPX 파일 → 모델 변환
├── document_core/  ← 편집 명령 + 조회 (CQRS)
├── renderer/       ← 레이아웃, 페이지네이션, SVG/Canvas
├── serializer/     ← 모델 → HWP 파일 저장
└── wasm_api.rs     ← WASM 바인딩

rhwp-studio/        ← 웹 에디터 (TypeScript + Vite)
```

의존성 방향: `model` ← `parser` ← `document_core` ← `renderer` ← `wasm_api`

## 코드 스타일

- `cargo clippy -- -D warnings` 경고 0건 (CI에서 강제)
- `unwrap()` 최소화
- 모든 문서는 한국어로 작성

## HWP 단위 참고

- 1 inch = 7,200 HWPUNIT
- 1 mm ≈ 283.465 HWPUNIT

## 소통

- **[Discussions](https://github.com/edwardkim/rhwp/discussions)** — 질문, 아이디어, 기술 토론
- **[Issues](https://github.com/edwardkim/rhwp/issues)** — 버그 리포트, 기능 요청

## Notice

본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.

## License

이 프로젝트는 [MIT License](LICENSE)로 배포됩니다. 기여하신 코드도 동일한 라이선스가 적용됩니다.
