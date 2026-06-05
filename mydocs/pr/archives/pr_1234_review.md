# PR #1234 검토 — 폰트 충실도: 한컴 돋움 폴백을 Noto Sans KR ExtraLight로

- **작성일**: 2026-06-02
- **PR**: #1234 (OPEN)
- **제목**: `폰트 충실도: 한컴 돋움 폴백을 Noto Sans KR ExtraLight로 (closes #1224)`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1224
- **base/head**: `devel` ← `feature/issue-1224-font-fidelity`
- **Head SHA**: `dbf8aa87789d463e417cc7e43c4b5e071f512308`
- **PR 기준 base SHA**: `f83c43b57ee4e9bf3e5ecb8be73f53a81f290430`
- **현재 local/devel**: `914ee139` (#1232 반영 후)
- **규모**: 23 files, +2257 / -1389, 1 commit
- **mergeable**: true
- **PR 댓글**: 없음

## 1. PR 요약

PR #1234는 #1224에서 보고된 “한컴 돋움(Haansoft Dotum) 미설치 환경에서 Noto Sans CJK KR Regular로 폴백되어 본문이 과하게 굵게 보이는 문제”를 폰트 weight 문제로 재규명하고, `Noto Sans KR ExtraLight`를 대체 폰트로 추가한다.

핵심 방향은 다음과 같다.

```text
1. SVG/웹 텍스트의 advance/position/textLength는 그대로 유지
2. 글리프 획 두께만 한컴 돋움에 가까운 ExtraLight로 보정
3. 네이티브 SVG 폴백 체인과 rhwp-studio 웹폰트 매핑을 함께 보강
4. OFL 폰트 자산과 측정/전략 문서를 함께 추가
```

## 2. 주요 변경 범위

| 영역 | 변경 |
|---|---|
| `src/renderer/mod.rs` | `generic_fallback` sans 체인에 `'Noto Sans KR ExtraLight'`를 `'Noto Sans KR'` 앞에 삽입 |
| `src/renderer/svg.rs` | 고딕/돋움/굴림 계열 대체 후보 `NotoSansKR-ExtraLight.ttf`, `ttfs/opensource` 탐색 경로 추가 |
| `rhwp-studio/src/core/font-loader.ts` | `Haansoft Dotum`, 돋움/돋움체/굴림/새굴림, `Noto Sans KR ExtraLight` 웹폰트 매핑 추가 |
| `ttfs/opensource/*` | `NotoSansKR-ExtraLight.ttf`, OFL, README 추가 |
| `web/fonts/*` | `NotoSansKR-ExtraLight.woff2`, FONTS.md 갱신 |
| `tests/golden_svg/*` | font-family 체인에 ExtraLight가 들어간 golden SVG 갱신 |
| `mydocs/*` | #1224 계획/단계/최종 보고서, font fallback 전략/측정 문서 추가 |

## 3. 타당한 부분

### 3.1 원인 분석 방향

이슈 #1224는 처음에는 em-fill/크기 문제처럼 보였지만, PR은 동일 페이지 픽셀폭 실측으로 “잉크 높이는 같고 획 밀도가 다르다”는 결론을 제시한다.

이 결론은 현재 렌더러 구조와도 맞다. rhwp는 위치와 폭을 메트릭 DB와 `textLength`로 고정하므로, 실제 시각 차이는 폴백 폰트의 glyph weight가 좌우한다. 따라서 레이아웃 로직을 건드리지 않고 폰트 weight만 보정하는 방향은 안전한 편이다.

### 3.2 웹 경로

PR 파일은 `web/fonts/NotoSansKR-ExtraLight.woff2`를 추가한다. `rhwp-studio/public/fonts`는 `../../web/fonts` symlink이므로, `font-loader.ts`의 `fonts/NotoSansKR-ExtraLight.woff2` 참조는 실제 배포 구조와 맞다.

웹에서는 `@font-face`로 폰트를 등록하므로 사용자가 별도로 OS 폰트를 설치하지 않아도 결정적으로 적용될 수 있다.

### 3.3 라이선스/용량

추가 폰트는 SIL OFL 1.1 문서와 README를 포함한다.

파일 크기:

```text
ttfs/opensource/NotoSansKR-ExtraLight.ttf: 약 2.8 MB
web/fonts/NotoSansKR-ExtraLight.woff2: 약 0.6 MB
```

저장소/확장 배포 규모 관점에서 감당 가능한 크기다.

### 3.4 적용 한계 문서화

PR은 `--embed-fonts` 경로가 현재 cmap 제거 때문에 SVG `<text>`에는 실효성이 없고, 네이티브 CLI/rsvg는 fontconfig에 ExtraLight가 설치되어야 실제 적용된다고 명시한다. 이 한계는 숨기지 않고 문서화되어 있어 수용 가능하다.

## 4. 확인 필요 사항

### 4.1 PR base가 현재 devel보다 뒤처짐

PR의 base는 `f83c43b5`이고 현재 `local/devel`은 `914ee139`이다.

그 사이 #1232가 렌더링 공통 경로(`typeset.rs`, `layout.rs`, `height_cursor.rs`)를 크게 변경했다. 이 PR은 폰트 중심이지만 `src/renderer/mod.rs`, `src/renderer/svg.rs`, golden SVG를 변경하므로 현재 devel 기준 merge 시뮬레이션과 회귀 테스트가 필요하다.

### 4.2 golden SVG 변경 범위

golden SVG 6개가 갱신된다.

```text
tests/golden_svg/form-002/page-0.svg
tests/golden_svg/issue-147/aift-page3.svg
tests/golden_svg/issue-267/ktx-toc-page.svg
tests/golden_svg/issue-617/exam-kor-page5.svg
tests/golden_svg/issue-677/bokhakwonseo-page1.svg
tests/golden_svg/table-text/page-0.svg
```

대부분 font-family chain 문자열 변경이지만, snapshot이 넓게 바뀌므로 현재 devel 기준 `cargo test --test svg_snapshot`를 반드시 확인해야 한다.

### 4.3 단계 문서의 Light/ExtraLight 서술 혼재

`font_fidelity_measurement_1224.md`와 Stage 2 문서는 초기에 `Noto Sans KR Light(wght 300)`를 선정/적용한 흐름을 기록한다. Stage 3/4와 최종 보고서는 이후 검증에서 `ExtraLight(wght 200)`로 확정했다고 설명한다.

작업 흐름 기록으로 보면 blocker는 아니다. 다만 독자가 Stage 1 문서만 보면 최종 선택을 오해할 수 있으므로, 수용 과정에서 “Stage 3에서 ExtraLight로 재확정됨” 같은 짧은 보완 주석을 Stage 1/2 문서에 추가하는 것이 좋다.

### 4.4 네이티브 CLI 적용 경계

`generic_fallback` 체인에 `Noto Sans KR ExtraLight`가 추가되어도 네이티브 SVG/rsvg는 OS/fontconfig에 해당 폰트가 있어야 실제로 그린다. PR 문서가 이 한계를 명시하므로 문제는 아니지만, 사용자가 “CLI export-svg만으로 항상 해결”로 이해하지 않도록 릴리즈 노트/가이드에서는 웹 자동 적용과 네이티브 fontconfig 의존을 구분해야 한다.

## 5. 권장 검증

현재 devel 기준 검증 브랜치에서 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --lib renderer::tests::test_generic_fallback
cargo test --test svg_snapshot
cargo test --lib
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

메인테이너 시각 판정 권장 대상:

```text
1. samples/3-09월_교육_통합_2023.hwp 6쪽 문26 주변 본문
2. pdf/3-09월_교육_통합_2023.pdf 동일 구간
3. samples/3-11월_실전_통합_2022 계열 비교 자료가 있으면 함께 확인
4. rhwp-studio 웹 캔버스에서 Haansoft Dotum/돋움 계열 본문 굵기
```

## 6. 권장 처리

권장안: **수용 후보로 진행한다. 단, 현재 `local/devel` 기준 검증 브랜치에서 병합하고, Stage 1/2 문서의 Light→ExtraLight 재확정 흐름을 보완한 뒤 테스트/WASM/Studio 빌드와 메인테이너 시각 판정을 게이트로 둔다.**

이 PR은 레이아웃을 직접 바꾸지 않고 폰트 weight만 보정하는 접근이라 방향이 좋다. 다만 폰트 자산과 golden SVG가 들어가고, 현재 devel이 PR base보다 앞서 있으므로 maintainer 검증 브랜치를 통한 수용이 안전하다.

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1234-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1234를 병합 시뮬레이션
3. 필요 시 Stage 1/2 문서의 Light→ExtraLight 보완 주석 추가
4. 자동 테스트/WASM/Studio 빌드
5. 메인테이너 시각 판정 후 local/devel 반영 및 PR 종료
```
