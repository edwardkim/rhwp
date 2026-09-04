# task_m100_3301 처리결과 보고서 — WMF/CFB 파서 안전 버그 2건

- **이슈**: [#3301](https://github.com/edwardkim/rhwp/issues/3301)
- **브랜치**: `pr/task-static-bugs-bundle` (upstream/devel 직분기)
- **범위**: `src/wmf/parser/objects/structure/poly_polygon.rs`,
  `src/parser/cfb_reader.rs`
- **분류**: 버그 수정 (메모리/자원 안전, 정적 분석 발견)

## 1. 배경

에이전트 팀을 동원해 이 저장소가 이미 겪은 결함 클래스(#3008 Region scan_count 미검증,
#3181 CFB DIFAT 순환 미탐지, WMF poly_line/polygon 미검증)와 같은 패턴을 코드베이스
전역에서 정적으로 훑고, 각 발견을 독립 에이전트가 반박 시도하는 적대적 검증을 거쳐
확정했다. 확정된 것 중 **서로 겹치지 않는 파일**·**정적 분석만으로 재현 가능**·
**빌드/테스트 부담이 작은** 2건을 이번 PR 로 묶었다.

## 2. 버그 1 — WMF PolyPolygon 점 개수 누적 오버플로

`src/wmf/parser/objects/structure/poly_polygon.rs`

`number_of_points`(타입 추론상 u16)가 `number_of_polygons`(u16, 최대 65535)개의
`a_points_per_polygon` 값(각각 u16, 최대 65535)을 `+=` 로 그대로 누적한다. 다각형
2개·각 65000점이면 합계 130000 > 65535 — debug 빌드는 오버플로 패닉, release 빌드는
wrap 되어 `aPointsPerPolygon` 이 요구하는 점 개수보다 적게 `aPoints` 를 읽어 이후
스트림 파싱이 어긋난다(desync).

**수정**: 누산 타입을 `u32` 로 올리고 `checked_add` 로 초과 시 `ParseError::NotSupported`
를 명시적으로 반환한다. u16×u16 의 이론적 최대 합(65535×65535 ≈ 42.9억)은 u32 범위
안이라 오버플로는 실질적으로 발생하지 않지만, 방어를 명시적으로 남겼다. `a_points` 의
`Vec::with_capacity` 는 사전 대량 할당을 피하기 위해 상한(1<<20)을 씌웠다 — 실제 벡터
크기는 이후 입력 스트림 소진 시 자연히 제한된다.

## 3. 버그 2 — CFB DIFAT→FAT sector id 중복 미검증 (DoS 증폭)

`src/parser/cfb_reader.rs` (`LenientCfbReader::open`)

DIFAT 파싱은 순환(같은 DIFAT 섹터로 되돌아옴)만 `visited_difat` 로 막을 뿐, 서로 다른
(또는 같은) DIFAT 엔트리에서 **같은 FAT 섹터 id 를 반복 기재**하는 것은 걸러내지
않는다. 유효한 CFB 파일에서는 각 FAT 섹터 id 가 DIFAT 에 한 번만 나타나는 것이 정상
불변식인데, 이를 검증 없이 신뢰하면 물리 FAT 섹터가 1개뿐이어도 그 id 를 DIFAT
섹터 하나에서 최대 `entries_per`(4096B 섹터 기준 1023)번 반복 기재해 FAT 벡터를
그만큼 부풀릴 수 있다. DIFAT 섹터를 늘리면(파일 크기는 그 섹터 수만큼만 증가)
FAT 벡터가 선형으로 폭증한다 — #3181 이 고친 "카운트 필드를 무검증으로 반복 사용"
하는 것과 같은 결함 클래스다.

**수정**: `visited_fat_sids: HashSet<u32>` 를 도입해 헤더 109슬롯·추가 DIFAT 체인
양쪽에서 동일 id 를 조용히 건너뛴다. 유효한 파일의 정상 동작(각 id 가 원래 한 번씩만
나타남)에는 영향이 없다.

## 4. 검증

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: **2921 passed, 0 failed,
  7 ignored**
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `CARGO_INCREMENTAL=0 cargo fmt --check`, `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 5. 메인터너 보정

원 PR의 음수 `width` 변경은 `BitmapInfoHeaderInfo::parse()`가 이미 `width <= 0`을
오류로 거부하는 기존 경로와 중복이므로 제거했다. 두 실제 수정에는 각각 총점 65,536의
PolyPolygon과 추가 DIFAT의 중복 FAT SID를 사용하는 회귀 테스트를 추가했다.

## 6. 남긴 것

이번 사냥에서 함께 확정됐으나 이번 PR 범위에서 제외한 것(모델+파서+직렬화기 다중 파일을
동시에 건드려야 하고 검증 샘플이 불확실해 "빌드 가볍고 성공 확률 높은" 기준에 맞지 않음):

- `SectionDef` 의 `secPr@memoShapeIDRef` HWPX 왕복 손실 (`src/model/document.rs:211`)
- `FootnoteShape.placement` 열거형 접힘으로 `END_OF_SECTION`/`BELOW_TEXT` 비가역
  (`src/model/footnote.rs:85`)
- `PageBorderFill` 위치 기반 합성으로 왕복 불일치 가능성 (`src/model/page.rs:70`,
  확신 낮음 — 파서 라인 미확인)

이들은 별도 이슈로 다뤄야 한다고 본다.
