---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-09-16
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
  --font-environment environment.json --pages 1 --out /tmp/font-review
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
