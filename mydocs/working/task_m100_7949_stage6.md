# Task #7949·#7950 — 도형 채움과 보이는 paint 보존

- 작성일: 2026-10-16 (Asia/Seoul)
- 제품 검증 source: `d37cc9257be276d9ea2684b4e48c09194132e9ba`
- 상태: 사용자 직접 시각 판정 통과. 집중 회귀 통과, 최종 제출 검증 진행 중
- 앞 단계: [19쪽 전체 열기와 반복 해석](task_m100_7949_stage5.md)

> 후속 정정: MOF의 이 단계 점수는 배경색 아래 가려진 표 글자를 검출하지 못했다.
> 19쪽 문서에 대한 사용자 판정과 별개로 이 회귀를 실제 PNG에서 확인해 수정했다.
> 최종 표 승격 경계·수정 전 실패/수정 후 통과와 devel 통합 결과는
> [stage7](task_m100_7949_stage7.md)을 따른다. 아래 값은 당시 source의 관측 기록이다.

## 원인과 적용 규칙

마스크를 합성한 그림과 별개로, 기존 lowering은 PDF의 채운 경로를 버리고 일부 직선만
보존했다. 추출된 투명 텍스트도 일반 글상자로 다시 만들면서 그림 위에 중복 표시했다.
PDF의 fill/stroke 색·`ca`/`CA`·`Tr`·winding·clip·paint 순서를 해당 소비 경로까지 전달했다.
근거는 원본 PDF와 PDF Reference 1.7의 graphics state·path painting·transparency 계약이다.
검색용 추출과 실제 그리는 텍스트를 구분하며, 특정 문서나 색상에 대한 분기는 추가하지 않았다.

| 생산 → 실제 소비 | 보정과 경계 |
| --- | --- |
| `paths.rs`의 path/paint → `lowering.rs` → `path_paint.rs` → 기존 도형/그림 paint | 사각형·단일 다각형은 편집 가능한 채움을 유지한다. 복합 곡선·구멍·비사각형 clip은 개체 범위의 투명 PNG에 보존한다. 전체 페이지를 이미지로 바꾸지 않는다 |
| `text.rs`의 `Tr`, ExtGState → `extract_painted_spans` → 글상자 | 보이지 않는 OCR/접근성 문자를 다시 그리지 않는다. 기존 일반 텍스트 추출 API는 숨김 문자를 계속 보존한다 |
| 원본 path/image operator 순서 → control의 `z_order` → 기존 출력 backend | 그림 뒤의 배경과 그림 앞의 도형을 구분한다. Form 호출 경로도 순서 키에 포함한다 |
| 동일 path bbox·clip → `common`/`lower_picture` → 저장 frame·paint | 기존 페이지 좌표와 실제 보이는 영역을 사용한다. 별도 표 높이·분할·재조판 규칙은 바꾸지 않았다 |

경로 raster의 크기·버퍼와 clip scratch는 예약 전에 기존 요청 예산에 계상한다.
동일 요청의 종료된 zlib scratch 한 블록만 재사용하며, 출력 버퍼와 동시 생존 codec의 비용은
계속 계상한다. 256MiB 누적·32MiB 개별 한도를 늘리거나 sticky 실패를 해제하지 않았다.
전체 파서 할당 증명은 여전히 `parserAllocationBoundsProven=false`다.

## 실제 실행 결과

모든 경로는 진단용 HWPX가 아닌 원본 PDF를 입력했다. Native는 기본 feature에
`native-skia`만 추가했고, WASM은 표준 Docker `wasm` 서비스의 기본 feature 빌드다.
ignored `output/pr-review/issue7949/`의 `type002/paint-final-source.json`에 제품 파일·실행 파일·
WASM/JS SHA를, `paint-final-results.json`에 전쪽 TSV SHA·쪽수·최저값을 고정했다.

| 입력·검증 | Native | fresh Docker WASM |
| --- | --- | --- |
| type002 19쪽, 최저 실루엣 일치율 | 99.40149%, 11쪽 | 99.40886%, 11쪽 |
| MOF 1쪽 | 97.39936% | 97.47529% |
| 공개 대조 PDF 12쪽, 최저 | 90.14609%, 11쪽 | 90.16832%, 11쪽 |
| 합성 fill/숨김 문자 | 99.99778% | 100% |
| 합성 compound winding/Bezier | 100% | 100% |
| 합성 path/image/path 순서 | 100% | 100% |

전체 32쪽의 Native/WASM 각각에 미달·누락·쪽수 차이는 없다. 첫 MOF의 표 배경·흰 글자도
공통 보정으로 개선됐다. 기존 시각 예외를 새 type002 문서에 사용하지 않았다.
2·11·18쪽에서 원본과 양쪽 PNG를 직접 대조해 채움·흰 글자·구멍·그림 위 중복 글자를 확인했다.
기본 폰트 메트릭·작은 bullet 모양 등 차이는 완전 일치라고 보고하지 않는다.

사용자가 Studio의 2쪽 사각형 채움과 18쪽 표 배경을 확인하고 **시각 판정 통과**를 밝혔다.
실제 Canvas Studio의 PDF 형식·19쪽·오류 없음과 fetch WASM SHA를
`type002/studio-paint-result.json`에 보존했다. 루트 pkg와 Studio public의 WASM/JS도 같다.

- Studio: `http://localhost:7798/?review=issue7949-type002-paint-d37cc9257&lang=ko`
- Windows 전쪽 비교: `E:\tmp\issue7949-type002-review-20261016\index.html`
- Linux 전쪽 비교: `/mnt/e/tmp/issue7949-type002-review-20261016/index.html`
- 대표 2·11·18쪽 비교: `type002/paint-review/p02-review.png`, `p11-review.png`, `p18-review.png`
- 전쪽 TSV: `type002-paint-final-native/type002/silhouette.tsv`,
  `type002-paint-final-wasm/type002/silhouette.tsv`

## 회귀와 제출 경계

선행 시각 조건과 직접 판독 뒤 `tests/cases/issue_7950_pdf_paint.rs`에 3건을 추가했다.
같은 테스트 실행 파일이 CLI를 실행하고 저장된 HWPX의 실제 내용을 검사한다.
수정 전 producer `1c426c5dc`는 숨김 문자 재표시·채움 누락·복합 경로 누락으로 3/3 실패했고,
수정 producer는 3/3 통과했다. px 위치나 전체 SVG 해시를 고정하지 않았다.
실행 로그는 `paint-contract-before.log`, `paint-contract-after.log`, `paint-contract-final.log`다.

자원 검사 113건 중 Flate 검사 1건이 이전의 매번 새 scratch 할당을 기대해 실패했다.
실제 새 예약량 `scratch + 출력 바이트 누적`으로 정정하되 cap−1/정확한 cap/cap+1,
개별 scratch cap과 sticky 실패 검사는 유지했다. 정정 뒤 **113/113 통과**했으며
`paint-resource-final.log`와 `paint-resource-corrected.log`에 전후 결과를 보존했다.
실제 WASM/Worker 계약은 85 assertion·5 Worker, alpha·Matte PNG 대조는 9/9 통과했다.

제출 전 Clippy에서 불필요한 unwrap과 모든 필드를 이미 지정한 struct update를 검출했다.
같은 조건의 값 바인딩과 중복 기본값 제거로 정리했다. 이 기계적 정리 후 최종 제품 재검증,
전체 회귀와 남은 lint는 별도 최종 기록에 연결한다. 앞선 시각 통과를 다른 source의 새 실행으로
표시하지 않는다. 원격 push·PR 생성·이슈 종료·기본 Studio 7700 변경은 수행하지 않았다.
