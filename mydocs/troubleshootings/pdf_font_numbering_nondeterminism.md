---
kind: reference
status: active
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# PDF 폰트 리소스 채번 비결정 (#2269) — 원인·해법

## 증상

`rhwp export-pdf <문서>` 를 **동일 바이너리로 2회** 실행하면 산출 PDF 가 바이트 diff 를
낸다(86712: 2.4MB, 75544: 2.57MB diff). **시각 출력·파일 크기·구조는 동일** — 회귀
diff 게이트(전/후 바이트 대조)만 방해받는다.

## 근본 원인 (업스트림 svg2pdf-0.13)

`svg2pdf-0.13.0/src/util/context.rs:81`:

```rust
for font in self.fonts.values_mut() {
    if let Some(font) = font { write_font(pdf, allocator, font)? }
}
```

`self.fonts` 는 `HashMap<ID, Option<Font>>`(context.rs:23). `.values_mut()` 반복 순서가
**프로세스별 랜덤 시드**(Rust 기본 `RandomState`)라 실행마다 다르다. `write_font` 이
반복 순서대로 객체 ref 를 할당하므로:

- 폰트 **객체 번호**(`/foN → M 0 R` 의 M)가 실행마다 달라진다.
- `/foN` **리소스 이름**도 다른 폰트에 배정된다(font.reference 가 위 채번 산물이므로).
- 나아가 객체 **방출 순서**(chunk `offsets` Vec)와 `/Font` 딕셔너리 **항목 순서**도 달라진다.

우리 측 `renderer/pdf.rs:366` 의 `chunk.renumber` 는 결정적(pdf-writer Chunk 는 Vec 저장)
이나, 입력 chunk 의 객체 순서 자체가 비결정이라 최종 산출이 비결정이 된다.

## 해법

### (권장, 근본) 업스트림 결정화
`context.rs:81` 을 폰트 ID 정렬 반복으로 바꾸면 결정화된다:
```rust
let mut ids: Vec<_> = self.fonts.keys().copied().collect();
ids.sort();
for id in ids { if let Some(font) = self.fonts.get_mut(&id).and_then(|f| f.as_mut()) { ... } }
```
svg2pdf 상위 기여 또는 `[patch.crates-io]` 벤더 패치가 필요하다(의존성 fork 결정은
메인테이너 판단 — 본 이슈에서는 미적용).

### (즉시, 자립) 정규화 비교 도구
`tools/pdf_normalize_compare.py` — 두 PDF 의 (1) 객체 방출 순서(내용 기준 정렬)
(2) 간접 객체 번호 (3) `/foN` 폰트 이름 (4) `/Font` 딕셔너리 항목 순서를 정준화한 뒤
바이트 비교한다. 채번 비결정만 제거하므로, **정규형이 동일하면 시각/구조 불변**으로
판정한다. 회귀 diff 게이트가 이 정규형을 쓰면 된다.

```
python tools/pdf_normalize_compare.py before.pdf after.pdf   # 0=불변, 1=진짜 회귀
python tools/pdf_normalize_compare.py --emit x.pdf > x.norm   # 게이트용 정규형 출력
```

검증: 86712(7폰트·65쪽)·75544 각각 2회 실행 raw diff 2.4~2.57MB → **정규형 바이트 동일**.
서로 다른 문서는 정규형 상이(회귀 검출) 확인.

## 근본 정정 채택 (2026-07-15, #2269 (a)안 승인)

업스트림 기여 시도 결과 **typst/svg2pdf 는 2026-07-10 아카이브**되어 PR 개설이 불가
("This project is not maintained anymore", 공식 대체: krilla/krilla-svg). master 에도
해당 비결정 코드는 미수정으로 남아 있어, 작업지시자 승인 하에 **벤더 패치**를 채택했다.

### 패치 브랜치

**planet6897/svg2pdf `determinism-0.13`** (커밋 2caeb0a, v0.13.0 태그 기반 2지점 수정):

| 지점 | 수정 | 결정화 대상 |
|---|---|---|
| `util/context.rs` `write_global_objects` | 폰트를 `Font.reference` 정렬 후 반복 | `write_font` 내부 부가 객체 4개/폰트(CID·descriptor·cmap·data)의 채번 + 방출순서 |
| `util/resources.rs` `finish()` | 딕셔너리 항목을 reference 정렬 후 방출 | `/Font` 등 리소스 딕셔너리 항목순서 |

규명 보완: `Font.reference` 와 `/foN` 이름은 문서 순회 순서(`fill_fonts`/glyph 순회)라
**원래 결정적**이다. 비결정의 실체는 (1) write 시점 부가 객체 채번·방출순서
(2) 리소스 딕셔너리 항목순서 2지점이며, 따라서 위 표의 reference 정렬(폰트 ID 정렬이
아니라)이 자연스러운 정준 순서다.

### rhwp 적용

`Cargo.toml` 에 `[patch.crates-io]` 로 포크 브랜치를 지정 (Cargo.lock 이 정확한 rev 고정):

```toml
[patch.crates-io]
svg2pdf = { git = "https://github.com/planet6897/svg2pdf", branch = "determinism-0.13" }
```

### 검증 (upstream/devel 640d04f7 + 패치)

- 86712(7폰트·65쪽)·75544: 각 2회 실행 **바이트 동일** (기존 raw diff 2.4~2.57MB → 0)
- 같은 소스 무패치↔패치 출력: `pdf_normalize_compare.py` **정규형 동일** (순서/채번만 변경, 시각·구조 불변)
- 별도 컴파일 2회의 산출물도 바이트 동일 (빌드 재현성)

### 유지보수 노트

- `pdf_normalize_compare.py` 게이트는 이중 안전망으로 유지한다.
- **krilla-svg 마이그레이션 시 이 패치는 제거**한다 (svg2pdf 계열 EOL — 장기 축은 별도 이슈).
