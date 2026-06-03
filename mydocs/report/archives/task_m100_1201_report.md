# Task M100-1201 최종 보고서 — HWPX masterpage idRef 기반 연결

## 대상

- Issue: #1201
- 작업: HWPX masterpage XML의 EVEN/ODD 바탕쪽 파싱 및 section 연결
- 브랜치: `task-1201-hwpx-masterpage`

## 변경 요약

### `src/parser/hwpx/content.rs`

- `PackageInfo`에 `master_page_items`를 추가해 masterpage manifest 항목의 `id`와 `href`를 함께 보존했다.
- 기존 `section_master_page_files`는 fallback 및 기존 호환 경로로 유지했다.

### `src/parser/hwpx/section.rs`

- section XML에서 `<hp:masterPage idRef="...">`를 수집하는 helper를 추가했다.
- HWPX `masterPage@type` 해석을 helper로 분리하고 공식 문서식 표기와 실제 샘플식 표기를 함께 지원했다.
- 지원 표기:

```text
BOTH / Both / both
EVEN / Even / even
ODD / Odd / odd
LAST_PAGE / LastPage / lastPage
OPTIONAL_PAGE / OptionalPage / optionalPage
```

### `src/parser/hwpx/mod.rs`

- `parse_hwpx()`에서 section XML의 `idRef`를 먼저 수집한 뒤 `manifest id -> href -> masterpage XML` 순서로 연결하도록 변경했다.
- section 단위로 동일 href가 중복 연결되지 않도록 처리했다.
- `idRef`가 없거나 resolve되지 않을 때만 기존 manifest 순서 기반 fallback을 사용한다.

## 샘플 검증

대상 파일:

```text
/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx
/Users/melee/Downloads/[2027] 온새미로 1 본교재.pdf
```

확인 결과:

- section 0부터 section 4까지 모두 `바탕쪽: 2개`가 채워진다.
- section 0:
  - `[0] Even`: `2027 온새미로 II`
  - `[1] Odd`: `독서 · 문학`
- section 1부터 section 4:
  - `[0] Even`: `2027학년도 수능 대비`
  - `[1] Odd`: `독서 · 문학`
- PDF 4/5/6/7쪽 기준의 짝수/홀수 바탕쪽 방향과 rhwp SVG의 masterpage 적용 방향이 구조적으로 반전되지 않는다.

주의:

- rhwp 파싱 결과는 47쪽, 원본 PDF는 49쪽이다. 전체 pagination 차이는 #1201 범위 밖의 기존 layout 차이로 남긴다.
- SVG footer 텍스트가 DOM에는 존재하지만 일부 PNG 렌더에서 극소 크기로 보이는 현상은 #1201의 idRef 연결 문제와 별도 이슈로 분리하는 것이 타당하다.
- 대상 원본 HWPX/PDF 파일은 PR에 포함하지 않는다.

## 검증

통과:

```text
cargo fmt --all --check
cargo test --lib hwpx
cargo test --test issue_1100_exam_social_hwpx_header
cargo test --test issue_1113_header_autonum_placeholder
```

추가 확인:

```text
rhwp-studio dev server: http://127.0.0.1:5177/
```

`rhwp-studio` 페이지 제목과 주요 UI 텍스트가 브라우저에서 로드되는 것을 확인했다.

## 완료 판정

- section XML의 `masterPage@idRef`가 파싱된다.
- manifest id/href와 `idRef`가 연결된다.
- `SectionDef.master_pages`가 대상 샘플의 section별 바탕쪽을 보존한다.
- masterpage `type`은 HWPX XML의 명시값을 기준으로 매핑된다.
- 실제 샘플에서 홀짝 바탕쪽이 구조적으로 반전되지 않는다.
- 지정 회귀 테스트가 모두 통과했다.
