# PR #7671·#7673 통합 호환 보정 — ZIP 9 엔트리 이름 API

## 원인과 변경

원 PR의 회귀 입력 생성 함수는 `ZipFile::name()`을 문자열로 사용했다.
현재 기준 `2efefc6a1f9812a188e32c1cb283e9ef7962ea3e`의 ZIP 9에서는
반환형이 `Result<Cow<str>, ZipError>`이므로 두 소스의 비교와 `start_file` 호출,
총 네 곳에서 E0369/E0277 컴파일 오류가 발생했다.

- `tests/cases/auto_number_after_control.rs`
- `tests/cases/new_page_column_definition.rs`

네 호출에서 엔트리 이름을 `expect("HWPX ZIP 엔트리 이름")`으로 확인한다.
입력 XML 변경, 자동 번호 위치·내용, 단 너비·방향·구분선 assertion은 유지했다.

## 확인한 증거와 남은 검증

동일 생성 함수를 복사해 실행하는 로컬 진단 helper는 변경 전 네 컴파일 오류로 실패했고,
변경 후 19개 입력을 생성하며 종료 코드 0으로 완료했다.
이 결과는 정식 regression suite 실행을 대신하지 않는다. 정식 focused 실행과 lint,
한컴 Print 및 Native/fresh WASM 시각 검증은 이어서 수행한다.

로그는 ignored `output/pr-review/semanticist21-20261015/logs/`의
`visual-input-generation-20261015.log`,
`visual-input-generation-after-zip-compat-20261015.log`에 보존했다.
최종 판정은 개별 PR review에 연결한다.

## 본문 PAGE/TotalPage 실제 출력 보정

원 PR의 serializer 위치 보존과 실제 본문 paint는 별개였다. 독립 Print는 각주·미주·그림 뒤
PAGE1을 표시하지만 통합 후보는 본문 번호를 공백으로 그렸다. 구역 첫 문단, 같은 쪽 두 필드,
다음 쪽 값, TotalPage와 PAGE 혼재, 한 문단의 세 쪽 분할, 두 자리 번호와 좁은 줄을 반례로 삼았다.

### 공통 결과와 실제 소비

1. `document_core/queries/body_page_fields.rs`가 원본 자동 번호와 자리표 모델 위치를 수집한다.
   FullParagraph는 명시적 전체 문단 소유, PartialParagraph는 실제 최종 run의 모델 범위로
   논리 쪽 소속을 정한다. 머리말/꼬리말·마스터·표·글상자·각주는 별도 흐름으로 제외한다.
2. `Paragraph.layout_field_values`는 원문/UTF-16/컨트롤 위치를 바꾸지 않는 비직렬화 파생값이다.
   composer `resolved_fields::apply`와 줄 토큰이 같은 모델 위치의 값을 읽는다.
3. 독립 Print의36/38.5px 두 줄·39.52px 한 줄, 크기20pt의77px 두 줄·79.04px 한 줄,
   PAGE12의39.52px 한 줄 관측에 따라 줄 수용은 한 em을 예약하고 paint는 숫자의 자연 전진을 쓴다.
   숫자 자릿수로 수용 폭을 늘리는 초기 가설은 독립 Print가 반증해 폐기했다.
4. frame·재조판 토큰화에서 공백으로 건너뛰지 않는 필드 원자 토큰을 쓰고, 실제 run 분할은 공통
   `display_for_model_range`를 소비한다. 각주 앞뒤와 TAC 앞뒤 네 최종 분할에서도 표시값을 보존한다.
5. `LeadingSpacePlan::with_body_fields`가 원본 소유 필드를 한 원자 간격으로 취급한다.
   필드 내부와 뒤에 양쪽 정렬의 탄성 간격을 붙이지 않으며, 일반 글자 뒤의 유효 간격은 유지한다.
   PAGE1·PAGE2·PAGE12의 선행/중간 필드 및 줄 끝 글자 위치를 독립 Print로 대조했다.
6. 실제 쪽수 의존 값이 줄 소속을 바꾸면 같은 composer/pagination을 다시 사용해 안정 상태를 정한다.
   필드 없는 문서는 기존 단일 pass를 유지한다. 순환/32회 예산 초과는 출력 오류로 표시한다.
   잘못된 좌표 clamp나 특정 문서 분기는 없다.
   본문 동적 필드 문단의 scalar 커닝 투영은 현재 지원 범위에 맞춰 K0 측정 경로를 사용한다.

그림 뒤에서 이미 원문 자리표를 소유한 후속 run의 `sub_char_offset`을 다시 증가시키던 경로를
보정했다. 실제 정식 owner 검사 수정 전1PASS/1FAIL(번호 scalar2, 기대1), 수정 후 통과했다.

### 입력과 독립 기대값

원 번호8개 입력과 추가7개 반례, 합계15문서22쪽은 Native/fresh WASM 전쪽100%·Print 쪽수 일치와
직접 확대 판독을 마쳤다. 공개 HWP 저장은 Studio와 같은 `HwpDocument.exportHwp()` adapter다.
추가7개는 명시적 XML 합성 입력이며 수동 편집으로 무효인 줄 캐시를 제거했다. 정상 한컴 생성본으로
보고하지 않는다. 서식·원문·컨트롤의 소유 및 직렬화 불변식과 실제 표시를 함께 검사한다.

[전쪽 값·원본/PDF SHA·PNG·페이지 선정](../assets/semanticist21_20261015/README.md),
[`body_page_auto_number_display.rs`](../../../tests/cases/body_page_auto_number_display.rs),
[`semanticist21_qualified_print_positions.rs`](../../../tests/cases/semanticist21_qualified_print_positions.rs)에
동적 값·같은/다음 쪽·TotalPage·부분 문단의 내용 보존 및 독립±2px 좌표를 연결한다.
실제 Studio Canvas2D/CanvasKit/SVG도 대표 입력의 소속·줄바꿈·그림과 후속 텍스트를 대조했다.

최종 통합 code 후보는 `20d93fe1d4226acab640b66875964d09c7d6d59b`이다.
위 ZIP 호환 보정 시점의 남은 gate 기록은 중간 이력이며, 최종 검증·원격 상태는 개별 review에 연결한다.
