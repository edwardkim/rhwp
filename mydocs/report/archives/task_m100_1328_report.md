# Task M100-1328 최종 보고서 — rhwp-studio 로컬 글꼴 감지 동의 UX

- 이슈: https://github.com/edwardkim/rhwp/issues/1328
- PR: https://github.com/edwardkim/rhwp/pull/1454
- 작성일: 2026-06-21
- 브랜치: `codex/1328-local-font-consent`
- 원격 PR 브랜치: `postmelee:1328-local-font-consent`

## 1. 목표

문서에 rhwp 기본 지원 글꼴이나 웹 대체만으로 원본에 가깝게 표시하기 어려운 글꼴이 있을 때,
사용자 동의 없이 로컬 글꼴 목록을 자동 조회하지 않고 먼저 안내 모달을 표시한다.

사용자가 승인한 뒤에만 로컬 글꼴 감지를 수행하고, 감지 결과는 브라우저/확장 로컬 저장소에만 보관한다.

## 2. 구현 결과

### Stage 1 — 로컬 글꼴 저장소와 상태 모델

- `LocalFontSnapshot`과 `LocalFontState`를 추가했다.
- 저장소 우선순위를 `chrome.storage.local` → `browser.storage.local` → `localStorage`로 정리했다.
- 저장된 snapshot 로드는 `queryLocalFonts()`를 호출하지 않도록 분리했다.

### Stage 2 — 문서 글꼴 상태 분석과 안내 모달

- `docInfo.fontsUsed` 기반으로 `available`, `needs-local-check`, `web-substitute`, `missing` 상태를 계산한다.
- 자동보정 모달과 같은 흐름의 로컬 글꼴 감지 모달을 추가했다.
- Firefox에서는 전체 목록 API가 없으므로 사용자 승인 후 현재 문서 후보 글꼴만 확인하는 fallback을 사용한다.

### Stage 3 — 원본 글꼴명 보존과 표시용 font-family chain

- 승인 전에는 미확인 로컬 글꼴명을 실제 CSS/Canvas font chain 앞에 두지 않는다.
- 승인 후 snapshot에서 확인된 글꼴만 원본 글꼴명 우선으로 렌더링한다.
- WASM Canvas2D 렌더링 경로에도 동일한 font substitution 규칙을 적용했다.

### Stage 4 — 감지 완료 이벤트와 UI 갱신

- `local-fonts-changed` 이벤트를 추가했다.
- 감지 후 canvas reload와 toolbar 글꼴 드롭다운 갱신을 같은 이벤트 경로로 연결했다.
- 환경설정에서 수동 감지를 실행할 때도 동일 이벤트를 발행한다.

### Stage 5 — 환경설정 재감지/초기화와 회귀 테스트

- 환경설정에서 저장된 로컬 글꼴 감지 상태를 표시한다.
- 로컬 글꼴 재감지와 감지 결과 초기화 버튼을 추가했다.
- 권한 거절, API 미지원, Firefox 후보 확인 경로의 안내 문구를 구분했다.

## 3. 샘플과 문서화

- `samples/hwpx/local-font-nanumsquare-bold.hwpx`를 추가했다.
  - 로컬에 `나눔스퀘어 Bold`가 있는 환경에서 승인 전/후 렌더링 차이를 확인하는 샘플이다.
- 샘플 제작과 rhwp-studio before/after PNG 검증 과정은
  `mydocs/working/task_m100_1328_sample_fixture.md`에 기록했다.
- 해당 과정을 일반화해 AI 샘플 문서 작성/검증 가이드
  `mydocs/manual/ai_sample_document_authoring_guide.md`를 추가했다.

## 4. 리뷰 지적 반영

PR #1454에서 GitHub Advanced Security가 테스트용 정규식의 비효율적 백트래킹 가능성을 지적했다.

- 대상: `rhwp-studio/tests/local-fonts.test.ts`
- 조치: 정규식 기반 quoted font family 추출을 선형 문자열 파서로 교체했다.
- 목적: CodeQL `Inefficient regular expression` 지적 제거.

## 5. 검증

통과:

```bash
git diff --check upstream/devel...HEAD
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
cargo fmt --all -- --check
cargo test
cargo clippy -- -D warnings
```

수동/시각 확인:

- Chrome에서 Local Font Access API 동의 모달과 승인 후 렌더링 변화를 확인했다.
- Firefox에서 전체 목록 조회 대신 문서 후보 글꼴 확인 fallback 문구와 적용 흐름을 확인했다.
- `samples/hwpx/local-font-nanumsquare-bold.hwpx`로 승인 전/후 PNG를 비교했고, 26,279px 차이를 확인했다.

## 6. PR 상태

- PR #1454는 `devel` 대상 draft PR로 생성했다.
- 이슈 자동 종료는 작업지시자 승인 전까지 하지 않기 위해 `Closes` 대신 `Refs #1328`로 연결했다.
- 원격 브랜치는 요청에 따라 `codex/` 없는 `1328-local-font-consent` 이름으로 push했다.
