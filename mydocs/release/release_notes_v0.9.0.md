# rhwp v0.9.0 — 릴리즈 노트

## [0.9.0]

PDF를 편집 가능한 문서로 가져오는 기본 경로와 Rust·CLI 소비 계약을 갖추고, 표·글상자·수식·저장 및 편집의 누적 보정을 반영합니다. 채널별 게시 시각과 배포 파일은 [GitHub Release](https://github.com/edwardkim/rhwp/releases/tag/v0.9.0)에서 확인할 수 있습니다.

### PDF와 개발자 온보딩

- PDF → Document IR → HWP/HWPX를 CLI·Rust·WASM·Studio에서 제공합니다. 선택 쪽과 `fidelity`/`editable` 배치 모드, 표 추론 진단을 지원합니다 ([#7913](https://github.com/edwardkim/rhwp/pull/7913), [#8120](https://github.com/edwardkim/rhwp/pull/8120)).
- JBIG2·JPEG 2000·ICC 색 변환·예산 있는 xref 복구를 기본 포함하고, 요청 한도와 Linux CLI 격리·브라우저 Worker 종료/복구를 연결했습니다 ([#8181](https://github.com/edwardkim/rhwp/pull/8181), [#8190](https://github.com/edwardkim/rhwp/pull/8190), [#8360](https://github.com/edwardkim/rhwp/pull/8360)).
- 패턴 이미지·문자 외곽선, 지원되는 곡선 clipping·회전 문자·mask 처리를 보강하고 Standard Security R2–R6 열기 암호 인증과 원본 쪽/전체 쪽·완료율을 제공합니다 ([#8410](https://github.com/edwardkim/rhwp/pull/8410), [#8420](https://github.com/edwardkim/rhwp/pull/8420)).
- 가져오기 선택 대기의 모래시계와 복잡한 JSON 기본 화면을 정리했습니다 ([#8401](https://github.com/edwardkim/rhwp/pull/8401), [#8412](https://github.com/edwardkim/rhwp/pull/8412)).
- **[PDF 개발자 시작 안내](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_developer_onboarding.md)**에 빌드·첫 변환·암호·Rust/WASM·Worker 배치·쪽별 진행률·오류 대응을 모았습니다.

### 편집·조판·저장

- 중첩 표·RowBreak 이어받기·셀 여백·저장 줄·TAC 개체의 흐름과 머리말/꼬리말 상속, 수식·글꼴 대체를 보정했습니다. 다수 기여자의 원 커밋과 별도 검증 근거를 보존해 통합했습니다.
- 표 칸 그림 삽입·다른 쪽으로 이동·Undo/Redo, 연속 형광펜, 중첩 셀 문단 서식, 논리 편집·다단 재조판 API를 보강했습니다 ([#8725](https://github.com/edwardkim/rhwp/pull/8725), [#8379](https://github.com/edwardkim/rhwp/pull/8379), [#8340](https://github.com/edwardkim/rhwp/pull/8340), [#8187](https://github.com/edwardkim/rhwp/pull/8187)).
- 글상자 세로 정렬·캡션·정의 슬롯·BinData 참조·그림/단색 배경과 저장 재열기 보존을 개선했습니다 ([#8309](https://github.com/edwardkim/rhwp/pull/8309), [#8711](https://github.com/edwardkim/rhwp/pull/8711), [#8738](https://github.com/edwardkim/rhwp/pull/8738)).
- HWP/HWPX 저장 후 재열기에서 컨테이너 안 글줄·표 앞뒤 본문·머리말/꼬리말 정의 슬롯을 보존하도록 보강했습니다 ([#8741](https://github.com/edwardkim/rhwp/pull/8741)).
- 큰 표 조회·수식/JPEG 그리기 비용을 줄이고, 웹 폰트의 지연/무응답 처리를 보강했습니다. 입력과 환경에 따라 성능 효과가 다릅니다.

- 준비 중 병합된 셀 도형 이동·수식 글꼴/원본 프레임·연속 표 보존을 포함합니다 ([#8754](https://github.com/edwardkim/rhwp/pull/8754)).
- 메모 읽기·삽입·수정·삭제 API와 HWP/HWPX 왕복, 작성자·생성일 보존, Studio의 기준 글자 삭제 Undo/Redo를 추가했습니다. 새 메모 UI는 별도 작업입니다 ([#8269](https://github.com/edwardkim/rhwp/pull/8269)).

### CLI·배포·Native 호스트

- Sixel·Kitty graphics 출력과 글꼴 지정 안내를 추가했습니다 ([#7829](https://github.com/edwardkim/rhwp/pull/7829), [#8154](https://github.com/edwardkim/rhwp/pull/8154)). 공식 CLI는 `native-skia`와 실제 PNG 실행 검사로 패키징합니다. v0.8.7 공식 archive의 Native Skia 누락을 보정하는 배포 계약입니다 ([#7818](https://github.com/edwardkim/rhwp/pull/7818)).
- **0.9.0부터 crates.io 배포를 시작합니다.** Rust 개발자는 `rhwp = "0.9.0"`으로 의존성을 추가하고, CLI 사용자는 `cargo install rhwp --version 0.9.0 --locked --bin rhwp`로 설치하는 경로를 제공합니다. Rust 1.93.1과 본체·내부 의존성 14개 패키지를 대상으로 하며, 초회 게시·공개 registry 소비 검증을 이번 릴리즈의 완료 조건에 포함합니다. 채널별 설치 가능 여부는 해당 registry의 0.9.0 게시 상태를 따릅니다.
- Native snapshot·타일·캐시·Qt/Flutter host 통합 기반을 추가했습니다. macOS Metal은 기본 비활성 실험 기반으로만 수용했고 Apple SDK 실검증이 남아 있습니다. 모든 플랫폼의 완성된 GPU 편집기를 배포한다는 뜻은 아닙니다 ([#8739](https://github.com/edwardkim/rhwp/pull/8739)).
- 시험지 ingest의 이미지 삽입과 에이전트·MCP 사용 안내를 보강했습니다 ([#8499](https://github.com/edwardkim/rhwp/pull/8499)).
- 암호 문서 인증과 진단 흐름을 분리하고 Debug 출력을 비식별화했습니다. CodeQL 탐지를 끄지 않았으며 보호 브랜치 전체 분석은 별도 후속 조건입니다 ([#8750](https://github.com/edwardkim/rhwp/pull/8750)).

### 호환성과 알려진 경계

PDF는 원저작 문서 구조를 완벽히 복원하지 않으며 스캔 글자를 OCR하지 않습니다. 지원 밖의 clip·mask·문자 paint·암호화 조합은 명시적으로 거부합니다. JSON 한도는 선택한 프로필보다 올릴 수 없고, 전체 parser 할당/RSS 상한은 입증되지 않았습니다. 동기 API의 취소·격리는 호스트가 구성해야 합니다. PDF 가져오기 결과는 입력 암호를 자동 승계하지 않습니다.

정밀 문자 획의 RHWP 확장과 반투명 PNG를 다른 편집기가 동일하게 인쇄한다고 보장하지 않습니다. 실제 글꼴을 공급하고 대상 앱 출력을 확인하세요. JS/WASM·Worker·호스팅 Studio는 같은 릴리즈로 고정해야 합니다. VS Code 기본 파일 연결은 HWP/HWPX입니다.

### 기여자

v0.8.7 이후 릴리즈 마감 기준 `41e08ae19e360a32179d851ca2534c1195a8dced`의 4,896개 커밋에서 저자·공동 저자·원 PR·cherry-pick 출처를 대조했습니다. 사람 33명, bot 1개(Dependabot)를 분리했으며 사람 identity와 번호 판정의 미해결은 0건입니다. 조회 불가한 cherry-pick 원본 9개는 통합된 커밋 저자로 보완하고 원장에 한계를 남겼습니다. 이 기준 이후의 릴리즈 기록 변경에는 새 기여자가 없는지도 확인합니다.

[기여자 감사 원장](https://github.com/edwardkim/rhwp/blob/devel/mydocs/tech/investigations/release-v0.9.0/contributor-ledger.json) · [별칭·판정 근거](https://github.com/edwardkim/rhwp/blob/devel/mydocs/tech/investigations/release-v0.9.0/contributor-overrides.json)

<!-- release-contributors:v0.9.0:start -->
- @9ood2s
- @ashcastle
- @bellkjtt
- @chrisryugj
- @davindev
- @donggyun112
- @edwardkim
- @emptinessform
- @greenoceanplanet
- @h1-andro2d
- @hccgahy1-hub
- @hexisteme
- @hyunmyungJaneLee
- @jangster77
- @jeong-sik
- @johndoekim
- @Kang-T
- @ldj-diquest
- @lidge-jun
- @mytrashcan
- @oksure
- @pblsketch
- @Phantomn
- @physwkim
- @planet6897
- @postmelee
- @sacru2red
- @scari
- @semanticist21
- @seo-rii
- @Steveyang8282
- @studyreadbook4ever
- @wook8170
<!-- release-contributors:v0.9.0:end -->

[Full change range](https://github.com/edwardkim/rhwp/compare/v0.8.7...41e08ae19e360a32179d851ca2534c1195a8dced)
