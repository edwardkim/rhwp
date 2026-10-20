---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-10-01
---

# 명시적 조판 폰트 환경

같은 HWP/HWPX라도 기준 PDF를 만든 컴퓨터에 원래 글꼴이 있었는지에 따라 줄바꿈이 달라질 수 있다.
문서의 폰트 이름만으로 PDF 생성 환경을 추정하지 않는다. `--font-environment`는 대체된 최종 폰트를
호출자가 알고 있을 때 **측정과 출력에 함께 적용하는 세션 설정**이다.

```json
{
  "id": "hancom-kopub-unavailable",
  "substitutions": {
    "KoPub돋움체 Light": "함초롬바탕",
    "KoPub돋움체 Medium": "함초롬바탕",
    "KoPub돋움체 Bold": "함초롬바탕"
  }
}
```

위 예시는 #6389의 KoPub돋움체→바탕 환경 선언이다. 전체 시스템의 폰트 미설치를 자동 재현하는
프로필이 아니며, 실제 기준 PDF와 대상 글꼴을 확인한 뒤 매핑을 선택한다. 테스트용 설정은
[`kopub-unavailable.json`](../../tests/fixtures/issue_6389/kopub-unavailable.json)에 있다.

## CLI와 Visual Sweep

```bash
rhwp export-svg input.hwp --font-environment environment.json -o output/svg --font-style
rhwp export-render-tree input.hwp --font-environment environment.json -o output/tree
rhwp export-pdf input.hwp --font-environment environment.json -o output/result.pdf

venv/bin/python scripts/visual_sweep.py \
  --file-target review input.hwp reference.pdf \
  --rhwp-bin target/pr-review/release-test/rhwp \
  --font-environment environment.json --pages 1 --out output/font-review
```

Visual Sweep은 Native와 `--wasm-pkg` 양쪽에 동일한 환경을 적용하고 JSON 파일 해시를 provenance에
기록한다. 설정이 달라지면 이전 `--resume` 결과를 재사용하지 않는다. 문서와 PDF의 대응 페이지는
내용으로 확인한다. #6389 편람의 KoPub PDF p68과 no-ttf PDF p69는 같은 내용이므로 단순히 같은
페이지 번호끼리 비교해서는 안 된다.

옵션을 생략하면 종전 동작과 출력 계약을 유지한다. SVG JSON manifest에는 설정한
`fontEnvironment`가 추가된다. CLI 지원 명령은 위 세 가지이며 다른 명령의 전역 옵션은 아니다.

## Rust와 WASM

```rust,ignore
let environment = rhwp::renderer::font_environment::FontEnvironment::from_json(json)?;
core.set_font_environment(Some(environment))?;
core.set_font_environment(None)?; // 기본 조판 복원
```

```javascript
// HwpDocument의 세션 설정. Studio의 자동 폰트 감지나 메뉴 설정이 아니다.
document.setFontEnvironment(JSON.stringify(environment));
document.setFontEnvironment(undefined); // 기본 조판 복원
```

- 매핑 key는 문서의 정확한 font face 이름, value는 최종 단일 face 이름이다. 재귀 치환하지 않는다.
- 환경 ID/이름의 빈 문자열, 앞뒤 공백, 제어 문자, CSS family chain 등을 거부한다.
- 내장 폰트는 미설치로 처리하지 않고 기존 이름을 유지한다. 매핑하지 않은 폰트도 기존 규칙을 따른다.
- 환경을 바꾸면 스타일·문단 구성·측정·페이지 캐시를 재구성한다. 이전 Canvas 측정 세션은 폐기된다.
- 같은 환경의 반복 적용은 no-op이다. 편집 batch 중 변경은 거부한다.
- 원본 FontFace와 저장 LineSeg를 수정하거나 HWP/HWPX에 환경 설정을 삽입하지 않는다.
- `getFontDecisionTrace`는 원본 face, 최종 layout/paint face, `fontEnvironment` 결정 단계를 구분한다.
  `oracle.status=declared`와 `profileId`는 호출자의 선언이며 실제 폰트 설치나 한컴 PDF 일치 인증이 아니다.

이 설정은 빠진 글꼴 파일을 제공하지 않는다. 출력 backend가 목표 폰트를 사용할 수 있어야 하며,
최종 시각 판정은 [Visual Sweep](verification/visual_sweep_guide.md)의 직접 비교 절차를 따른다.

## 같은 이름의 TrueType·HFT 프로그램 구분

독립 PDF와 실제 글꼴 파일이 TrueType 프로그램을 사용하는 것으로 확인되었을 때만
`trueTypeFaces`에 **최종 face 이름**을 지정한다. 생략하면 기존 HFT 호환 폭을 유지한다.
원본의 TTF 선언·파일 확장자·저장 제품 버전만으로 값을 채우지 않는다. 현재 별도 폭이 검증된
휴먼명조의 가운뎃점과 이중 따옴표 `“”`는 HMKMM.TTF의 실제 hmtx 전진폭을 사용한다.
독립 PDF의 동일 outline과 다음 글자 원점이 전각 전진을 확인한 범위다. 단일 따옴표와
다른 face는 별도 근거 없이 이 계약으로 바꾸지 않는다.

```json
{
  "id": "hancom2024-human-truetype",
  "substitutions": {"휴먼명조": "HumanMyeongJo"},
  "trueTypeFaces": ["HumanMyeongJo"]
}
```

`HumanMyeongJo`는 내장 메트릭의 휴먼명조 별칭이며 SVG 임베더도 HMKMM.TTF를 찾는다.
이 선택은 실제 PDF의 TrueType 글꼴·설치 파일 해시를 확인한 세션에 한정한다.
`--embed-fonts full --font-path <실제 글꼴 디렉터리>`로 원 프로그램의 outline을 제공하고
임베드 검사·Native/fresh WASM PNG를 확인한다. EBDT/EBLC 제거는 임베드 사본에만 적용한다.
프로필이나 메트릭 지원 자체는 글꼴 설치·출력 일치 인증이 아니며, HFT 기준 PDF에는 적용하지 않는다.
검증 기록에는 JSON과 글꼴 SHA-256, PDF의 실제 글꼴 형식, 임베드 결과를 남긴다.

## 결측 글리프의 명시적 폴백

원 face는 그대로 쓰되 일부 글자만 다른 프로그램에서 그려야 하면 `glyphFallbacks`로
공급을 확인한 face 순서를 선언한다. 키는 치환 후의 최종 face 이름이다. 일반 OS 폴백보다
먼저 조회하고, 원본의 `substFont`는 그 뒤에 보존한다. 기본값은 비어 있으며 자동으로 설치
글꼴을 찾아 채우지 않는다.

```json
{
  "id": "verified-primary-and-glyph-provider",
  "substitutions": {},
  "trueTypeFaces": ["휴먼명조"],
  "glyphFallbacks": {"휴먼명조": ["Haansoft Batang"]}
}
```

위 예시는 두 프로그램의 실제 공급과 문자 지원을 확인한 세션에만 적용한다. 메트릭 결측과
실제 glyph 결측을 구분한다. 기본 TrueType 프로그램의 검증한 cmap이 glyph 부재를 입증하고,
다음 선언 프로그램의 cmap과 전진폭도 확인된 범위에서만 다음 메트릭을 찾는다. 현재 검증한
기술 기호 구간은 regular 휴먼명조·한컴바탕 프로그램이다. 수집하지 않은 구간이나 다른
weight의 지원 여부는 추정하지 않고 기존 backend 보충 측정으로 남긴다. primary hit·공백·
문서의 HFT 폭 규칙은 유지하며, 뒤 face에 앞 face의 HFT 속성을 복제하지 않는다.
SVG는 체인 구성 face를 각각 임베드하고 Canvas와 Native Skia도 같은 선언 순서를 사용한다.
미지의 OS 선택을 메트릭 DB의 이름만으로 추정하지 않는다. DB에 없는 글자의 기존 보충 측정
계약도 유지한다.

확인한 SFNT의 `unitsPerEm`과 historical DB의 `em_size`가 다를 수 있다. 추출 시 실제 DB face와
em을 확인하고 `scripts/extract_verified_symbol_metrics.py --metric-em`으로 DB 단위를 지정한다.
원 프로그램의 폭과 em을 증거로 보존하고, 메트릭 소비에는 같은 비율로 환산한 폭을 쓴다.
cmap의 글자 유무는 단위 변환과 독립이다. 이름이나 숫자가 같다는 이유만으로 프로그램의
동일성을 주장하지 않으며, 실제 파일 해시·독립 Print의 glyph·backend 선택을 함께 확인한다.

최대256개 key, key마다1~16개의 단일 face 이름을 허용한다. 빈 목록·중복·자기 참조·CSS 삽입
문자열은 거절한다. 이 설정만으로 설치·선택·PDF 일치를 입증할 수 없다. 실제 출력 프로그램,
glyph cmap/전진폭, 공급 파일의 SHA-256과 backend 선택을 확인하고 동일 설정의
Native/fresh WASM 전체 페이지와 확대 판독으로 검증한다. 설정은 원본 문서에 저장하지 않는다.
