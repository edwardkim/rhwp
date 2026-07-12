# Task M100 #2185 최종 결과보고서 — 한글 줄 나눔 단위 의미 반전 정정

## 결과 요약

`issue1949_giant_cell_nested_tables_perf.hwp`의 대상 문단 끝에 `1` 하나만 입력해도 앞선
줄 경계와 문단 배치가 바뀌던 직접 원인을 확정하고 최소 수정으로 해결했다.

HWP/HWPX `attr1 bit 7`의 renderer 계약은 `0=어절`, `1=글자`인데 composer가 이를
반대로 소비하고 있었다. 토큰화와 줄 채움의 두 조건만 바로잡았으며 파서·직렬화기와
저장 bit 매핑은 변경하지 않았다. HWP/HWPX 실제 편집, 전체 pagination, 원본 형식 저장과
재로드, 새 WASM 및 Studio 브라우저에서 같은 줄 경계와 115쪽이 유지된다.

## 확정 원인

### 저장값과 소비 의미의 불일치

| 영역 | 올바른 계약 | 수정 전 동작 |
|------|-------------|--------------|
| HWP5 `ParaShape.attr1 bit7` | `0=어절`, `1=글자` | raw bit는 정상 보존 |
| HWPX 저장 매핑 | `KEEP_WORD→bit7=1`, `BREAK_WORD→bit7=0` | 정상 보존 |
| 모델·Studio UI | `0=어절`, `1=글자` | 정상 |
| composer 한글 어절 토큰화 | bit0에서 선택 | bit1에서 선택 |
| composer 단일 한글 줄바꿈 허용 | bit1에서 허용 | bit0에서 허용 |

파일을 처음 열 때는 한컴이 저장한 `LINE_SEG`를 사용하지만 편집이 들어오면 rhwp가
폰트 측정과 문단 reflow를 수행한다. 이때 반대로 해석된 한글 줄 나눔 단위로 줄을 다시
채워 앞선 `LINE_SEG.text_start`까지 달라졌다.

같은 폰트, 셀 폭, 측정 엔진을 둔 통제 대조에서 줄 나눔 소비 조건만 올바르게 선택하자
경계가 유지됐다. 따라서 이 재현의 직접 원인은 폰트 매트릭스 자체나 한컴 전체 조판
semantic의 부재가 아니라 `korean_break_unit` 소비 의미의 반전이다.

## 구현 내용

- `src/renderer/composer/line_breaking.rs`
  - 어절 토큰화 분기를 `korean_break_unit == 0`으로 정정했다.
  - 단일 한글 문자의 줄바꿈 허용 조건을 `korean_break_unit == 1`로 정정했다.
- `src/renderer/style_resolver.rs`
  - raw bit 추출은 유지하고 잘못된 의미 주석만 바로잡았다.
- `src/renderer/composer/tests.rs`
  - 토크나이저 테스트의 bit를 실제 계약에 맞췄다.
  - `"가나 다라"`, 60px 폭에서 bit0 `[0, 3]`, bit1 `[0, 4]`를 고정하는 결정적
    reflow 회귀 테스트를 추가했다.
- `tests/issue_2185_korean_break_unit.rs`
  - 공개 HWP/HWPX 샘플의 실제 편집부터 저장·재로드까지 고정하는 통합 테스트를 추가했다.

HWP5/HWPX 파서·직렬화기, 모델과 Studio 문단 모양 UI의 값 매핑은 수정하지 않았다.

## 실제 재현 문서 결과

대상은 `section=0 / parent paragraph=0 / table control=2 / cell=2 / cell paragraph=5`이며,
문자 인덱스와 UTF-16 위치가 모두 130인 문단 끝에 `1`을 입력했다.

| 상태 | `LINE_SEG.text_start` | 줄 수 | 다음 문단 `vpos` | 쪽수 |
|------|----------------------|-------|--------------------|------|
| 입력 전 한컴 저장값 | `[0, 44, 84, 122]` | 4 | 17160 | 115 |
| 수정 전 rhwp 입력 후 | `[0, 43, 78, 113]` | 4 | — (화면 배치 변경 관찰) | — |
| 수정 후 로컬 reflow | `[0, 44, 84, 122]` | 4 | 17160 | 115 |
| 전체 pagination 후 | `[0, 44, 84, 122]` | 4 | 17160 | 115 |
| 원본 형식 저장·재로드 후 | `[0, 44, 84, 122]` | 4 | 17160 | 115 |

HWP와 HWPX 양쪽에서 입력한 `1`만 보존되고 반환 커서는 131이 됐다. Studio 실제
브라우저에서도 마지막 줄만 `하여 적용한다.1`로 바뀌고 앞 세 줄, 다음 1.1.2 문단 위치,
`1 / 115 쪽` 상태가 유지됐다.

## 입력 지연과의 관계

정확성 버그와 입력 지연은 같은 사용자 입력에서 연속해 나타나지만 직접 원인은 다르다.

| 형식 | 문단 로컬 입력·reflow | 전체 pagination flush |
|------|-----------------------|---------------------------|
| HWP | 약 0.162ms | 약 1.168s |
| HWPX | 약 0.096ms | 약 1.185s |

이는 단일 로컬 실행의 관측값으로 성능 기준선은 아니지만, 한글 줄 나눔 계산 자체보다
115쪽 거대 셀 문서의 전체 pagination이 지연의 지배 항임을 분명히 보여 준다. Studio는
30쪽을 넘는 문서의 자동 전체 flush를 피하고 현재 페이지 로컬 갱신 경로를 사용했으며,
브라우저 재현에서는 `pagination` fallback이나 `Violation` 로그가 없었다.

따라서 #2185는 문단 구조 변경을 해결했으며 전체 pagination, page-tree와 Canvas 재도장
최적화는 후속 성능 이슈 #2193에서 추적한다.

## 단계별 결과

| 단계 | 결과 |
|------|------|
| Stage 1 | 두 소비 조건 정정, bit0/bit1 단위 회귀 테스트 통과 |
| Stage 2 | HWP/HWPX 편집·flush·저장·재로드 통합 회귀 핀 통과 |
| Stage 3 | 포맷·PUA·roundtrip·WASM·Studio 및 실제 브라우저 검증 통과 |
| Stage 4 | 최신 upstream rebase 후 전체 Rust·clippy·WASM·Studio·Canvas 게이트 통과 |

최종 주요 수치는 다음과 같다.

- 전체 Rust: **3,042 passed / 0 failed / 22 ignored**
- clippy: 경고 0건, fmt와 `git diff --check` 통과
- HWPX bit 7 파서·직렬화, #937 PUA, HWP5/HWPX roundtrip 통과
- #1949·#2164 인접 회귀 통과
- #1891 공식 PDF 핀: 82/157/17/65쪽 HWP/HWPX 유지
- Docker WASM 빌드 통과
- Studio: 185/185 테스트, production build, renderer contract 통과
- 최신 WASM의 legacy PageRenderTree/PageLayerTree 정합: 저장소에서 서빙 가능한
  6개 픽스처 모두 통과

상세 실행 결과는 `mydocs/working/task_m100_2185_stage1.md`부터
`task_m100_2185_stage4.md`까지 기록했다.

## upstream 동기화와 커밋

- 최초 기준: `upstream/devel@53a5093c`
- Stage 4 최초 동기화 기준: `upstream/devel@6f1bd284`
- PR 게시 전 최종 기준·merge-base: `upstream/devel@3077f96d`
- 최신 upstream 대비: 최종 재검증 시점 `0 behind / 4 ahead`
- 최종 upstream 추가 소스·테스트와 #2185 소스·테스트 경로 교집합: 0개
- 오늘할일 문서 add/add 충돌은 upstream PR #2191 기록과 #2185 기록을 모두 보존해 해결
- 두 차례 rebase 뒤에도 #2185 소스·테스트 diff는 동일

| 커밋 | 내용 |
|------|------|
| `a58d828f` | 한글 줄 나눔 단위 의미 복구와 단위 회귀 테스트 |
| `7ae9744d` | 실제 HWP/HWPX 편집·저장 회귀 테스트 추가 |
| `a07f663a` | WASM 및 Studio 회귀 검증과 Stage 3 보고 |

## 범위와 남은 항목

- 재생성 `LineSeg`의 `column_start`와 줄별 tag 보존 방식은 변경하지 않았다.
- 입력 지연의 전체 pagination·page-tree·Canvas 비용은 후속 성능 이슈 #2193의 대상이다.
- #2169 `kbu/kbu2`, 359 목록·기준선과 Windows 대량 코퍼스는 이슈 작성자의 저장소
  비추적 로컬 자산으로 확인돼 미실행·비차단 참고 항목으로 분류했다. 해당 자산에 대한
  `REGRESSED=0`은 주장하지 않는다.
- Canvas 11개 일괄 테스트는 `public/samples`에만 있는 픽스처의 Vite `/samples` HTTP 404
  서빙 제약이 있다. 렌더 불일치는 아니며, 루트에서 서빙 가능한 6개는 모두 통과했다.
- 기존 미추적 `scripts/frontend-metrics/`는 수정하거나 커밋하지 않았다.

## 최종 상태

#2185의 재현 원인, 최소 수정, 영구 회귀 핀과 반복 가능한 광역 검증이 모두 마련됐다.
현재 브랜치는 최종 승인 가능한 상태다. 원격 push와 PR 생성은 본 보고서 이후 게시
단계에서 수행하며, 브랜치 통합과 이슈 close는 별도 승인 전에는 수행하지 않는다.
