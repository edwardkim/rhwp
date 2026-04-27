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
cargo test                       # 1014+ 테스트 통과
cargo clippy -- -D warnings      # 린트 경고 0건
```

두 명령이 모두 통과하는지 확인한 후 PR을 생성해주세요.

### 한컴 PDF 와의 일치 검증에 대해

> ⚠️ **한컴 PDF 출력은 정답지가 아닙니다.**
>
> 동일 HWP 파일도 한컴 환경 (버전 / 폰트 설치 / OS / 출력 방법) 에 따라 PDF 결과가 다릅니다. 페이지 분할까지 환경별로 달라지는 사례가 발견되었습니다 (PR #360 정황). 따라서 **"한컴 PDF 와 일치"** 만을 PR 검증 기준으로 제출하셔도 머지가 보장되지 않습니다.

신뢰할 수 있는 검증 기준 (우선순위):

1. **결정적 자동 검증** (필수):
   - `cargo test --lib` (회귀 0)
   - `cargo test --test svg_snapshot` (rhwp 자체 일관성)
   - `cargo clippy --lib -- -D warnings`

2. **시각 검증** (참고):
   - 한컴 PDF / 한컴 화면 캡처 + rhwp SVG 비교 — **본인 환경 명시 필수** (한컴 버전, OS, 폰트 등)
   - 페이지 분할 영향 PR 의 경우 메인테이너 환경 재검증 후 머지 결정

3. **다른 렌더링 결과** (참고):
   - HTML / Canvas / VS Code 확장 등 다른 출력 경로와의 일관성

### 페이지 분할 / 페이지네이션 영향 PR 의 경우

페이지 분할은 한컴 환경 의존성이 가장 큰 영역입니다. 이 영역의 PR 은 다음 절차 권장:

1. PR 본문에 검증 환경 명시 (한컴 버전, OS, 폰트, 출력 방법)
2. 메인테이너 환경 재검증 후 머지 결정 (작업지시자가 직접 확인)
3. 회귀 테스트 (`tests/page_number_propagation.rs` 같은 패턴) 포함 권장

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

## 문서 작성 규칙

rhwp는 코드뿐 아니라 **작업 과정의 기록**도 프로젝트의 일부입니다(Hyper-Waterfall 방법론). PR에 문서를 포함하시는 경우 아래 규칙을 지켜주세요.

### 폴더 구조 (`mydocs/` 하위)

| 폴더 | 용도 |
|------|------|
| `orders/` | 일일 작업지시 (`yyyymmdd.md`만 허용) |
| `plans/` | 수행 계획서, 구현 계획서 |
| `working/` | 단계별 완료 보고서 (`_stage{N}.md`) |
| `report/` | 최종 결과보고서 (`_report.md`) **— 최종 보고서는 반드시 여기** |
| `feedback/` | 피드백, 코드 리뷰 의견 |
| `tech/` | 기술 조사·분석 (스펙 정오표, 라이브러리 발견 등) |
| `manual/` | 사용자/개발자 매뉴얼 |
| `troubleshootings/` | 트러블슈팅 (재발 방지용 해결 기록) |
| `pr/` | **외부 기여자 PR 검토 기록** (메인테이너가 관리, 기여자는 작성 불필요) |

### 파일명 규칙

타스크 관련 문서는 다음 형식을 따릅니다:

- 수행 계획서: `task_{milestone}_{이슈번호}.md` (예: `task_m100_235.md`)
- 구현 계획서: `task_{milestone}_{이슈번호}_impl.md`
- 단계별 보고서: `task_{milestone}_{이슈번호}_stage{N}.md` (`working/`)
- 최종 보고서: `task_{milestone}_{이슈번호}_report.md` (`report/`)

**주의 사항:**

- `task_` 접두어 고정 (`task_bug_`, `task_feat_` 등은 사용하지 않음)
- 마일스톤은 `m{숫자}` 형식 (예: `m100`). 생략·약식 금지
- 후속 수정: `_v2`, `_v3` 버전 접미어 사용 (`_fix`, `_hotfix` 금지)
- `orders/` 에는 `yyyymmdd.md` 외의 파일을 두지 않습니다. 이슈 상세 조사는 `troubleshootings/` 또는 `tech/` 로
- 최종 보고서(`_report.md`)는 반드시 `report/` 폴더에 위치 (`working/` 아님)

### 기여자가 작성해야 하는 문서 범위

기여자는 본인 작업 범위(내부 타스크 문서: `plans/`, `working/`, `report/`, `tech/`, `troubleshootings/` 등)만 작성합니다.

**`pr/` 폴더는 메인테이너가 PR을 검토한 기록을 남기는 전용 공간**이므로, 기여자는 직접 작성할 필요가 없습니다. 메인테이너가 PR을 리뷰하면서 `pr_{번호}_review.md`, `pr_{번호}_report.md` 등을 자동으로 생성합니다. 이 파일들은 나중에 **PR 처리 이력으로 공개**되므로, 본인 PR이 어떻게 검토되었는지 추적 가능합니다.

### 이 규칙이 애매하다면

애매한 상황이 있다면 PR 코멘트로 질문해주세요. 메인테이너가 안내드리고, 필요하면 이 문서를 보완합니다. (이 규칙 자체가 PR 리뷰 과정에서 지속적으로 다듬어지고 있습니다.)

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
