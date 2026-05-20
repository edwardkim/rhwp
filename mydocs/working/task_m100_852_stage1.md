# Task M100-852 Stage 1 분석 보고서

## 1. 결정적 발견 (이슈 본문 root cause 정정)

**이슈 #852 본문 가설** = Form 컨트롤 직렬화 결함. Stage 40 가 Form 컨트롤 본질 직렬화 (`CTRL_HEADER(form)` + `HWPTAG_FORM_OBJECT`) 추가하여 rhwp 재로드 OK 이나 한컴 손상 미해소.

**Stage 1 진단 결과 = Form 무관, HWPX parser 의 contract 스트림 일반 누락**. HWP→HWP roundtrip 은 정상 보존되나 HWPX→HWP 변환만 누락.

## 2. 정답지 / 변환본 CFB 스트림 비교

### form-01.hwp / form-02.hwp (정답지)

```text
9 streams: HwpSummaryInformation / BodyText/Section0 / DocInfo /
           DocOptions/_LinkDoc / FileHeader / PrvImage / PrvText /
           Scripts/DefaultJScript / Scripts/JScriptVersion
```

### rhwp convert HWPX → HWP 결과

```text
3 streams: BodyText/Section0 / DocInfo / FileHeader
```

**누락 6 스트림**:
- HwpSummaryInformation (461) — 문서 메타 (title, creator, subject 등)
- DocOptions/_LinkDoc (524) — UTF-16 LE 임시 파일 경로 메타
- Scripts/DefaultJScript (406) — Form/매크로 스크립트 (zlib)
- Scripts/JScriptVersion (13) — 스크립트 버전
- PrvImage (1415) — 미리보기 PNG
- PrvText (60) — 미리보기 텍스트

추가: BodyText/Section0 자체도 1620 → 418 (1202 누락, 74%) — Form 컨트롤 BodyText 영역 잔존 누락 (Stage 40 보강 후도 부족 또는 HWPX→IR 변환 누락).

## 3. Form 무관 입증 — 일반 HWP 정답지에도 동일 5 스트림 보유

| 정답지 | DocOptions | Scripts | Summary |
|--------|------------|---------|---------|
| form-01.hwp (Form) | ✅ | ✅ | ✅ |
| form-02.hwp (Form) | ✅ | ✅ | ✅ |
| hy-001.hwp (Form 없음) | ✅ | ✅ | ✅ |
| sample16-hwp5 (Form 없음) | ✅ | ✅ | ✅ |

**결론**: 누락 5 스트림은 Form 특화가 아닌 **모든 한컴 HWP 정답지의 contract**. troubleshootings/hwpx2hwp-rule.md 5.A (Container/Stream Contract) 위반.

## 4. HWP roundtrip vs HWPX 변환 비교 — root cause 확정

`rhwp convert samples/hwpx/hancom-hwp/hy-001.hwp output/.../hy-001-roundtrip.hwp` 결과:

| 스트림 | HY-001 ORIGINAL | HY-001 ROUNDTRIP | HWPX→HWP 변환 |
|--------|-----------------|------------------|---------------|
| HwpSummaryInformation | 473 | **473** ✅ | 0 ❌ |
| DocOptions/_LinkDoc | 524 | **524** ✅ | 0 ❌ |
| Scripts/DefaultJScript | 16 | **16** ✅ | 0 ❌ |
| Scripts/JScriptVersion | 13 | **13** ✅ | 0 ❌ |
| PrvImage / PrvText | 49429 / 2046 | **49429 / 2046** ✅ | 0 / 0 ❌ |

**HWP→HWP roundtrip 은 정상 (rhwp serializer 가 OLE 스트림 passthrough 보존)**, **HWPX→HWP 변환만 누락**.

## 5. HWPX 컨테이너에 동등 데이터 존재 확인

`samples/hwpx/form-01.hwpx` ZIP 내부 13 파일:

```text
mimetype, version.xml, settings.xml,
Contents/header.xml, Contents/section0.xml, Contents/content.hpf,
Preview/PrvText.txt (40), Preview/PrvImage.png (6509),
Scripts/headerScripts (860), Scripts/sourceScripts (700),
META-INF/container.xml, META-INF/container.rdf, META-INF/manifest.xml
```

| HWP 스트림 | HWPX 동등 파일 |
|------------|----------------|
| HwpSummaryInformation | Contents/content.hpf opf:metadata (title/creator/subject/date 등) |
| DocOptions/_LinkDoc | settings.xml |
| Scripts/DefaultJScript | Scripts/sourceScripts |
| Scripts/JScriptVersion | Scripts/headerScripts 메타 |
| Preview/PrvText | Preview/PrvText.txt |
| Preview/PrvImage | Preview/PrvImage.png |

**모든 데이터가 HWPX 컨테이너에 존재**. HWPX parser 가 읽지 않을 뿐.

## 6. rhwp 코드 진단

### Document IR 의 `extra_streams` 필드 보유

`src/model/document.rs`:
```rust
/// 파서가 모델링하지 않는 추가 CFB 스트림 (라운드트립 보존용)
pub extra_streams: Vec<(String, Vec<u8>)>,
```

### cfb_writer 가 `extra_streams` 를 그대로 작성

`src/serializer/cfb_writer.rs:155-158`:
```rust
// 6. 추가 스트림 (Scripts, DocOptions 등 — 라운드트립 보존)
for (path, data) in extra_streams {
    streams.push((path.clone(), data.clone()));
}
```

### HWP parser 가 OLE 스트림을 `extra_streams` 에 보존 (HWP roundtrip 성공의 원리)

`src/parser/cfb_reader.rs` 가 모든 OLE 스트림 읽어 `extra_streams` 에 저장 (확인 완료).

### HWPX parser 가 `extra_streams: Vec::new()` 으로 초기화

`src/parser/hwpx/mod.rs:185`:
```rust
extra_streams: Vec::new(),
```

→ HWPX 컨테이너 파일 (Scripts/settings/Preview/content.hpf) 미처리. **단일 누락 위치**.

## 7. 정적 템플릿 자산 (잠재적 해결책)

`src/document_core/commands/document.rs:515`:
```rust
const BLANK_TEMPLATE: &[u8] = include_bytes!("../../../saved/blank2010.hwp");
```

본 프로젝트가 이미 **"한컴 호환 최소 HWP 템플릿"** (`saved/blank2010.hwp`) 보유. 이 템플릿에서 contract 스트림 (HwpSummary / DocOptions / Scripts 등) 을 추출하여 HWPX→HWP 변환 시 기본값으로 사용 가능.

## 8. 해결책 후보 (Stage 2 구현 계획서에서 확정)

### A. HWPX 컨테이너 파일 → HWP OLE 스트림 변환 (정공법)

- HWPX parser 가 ZIP 내 settings.xml / Scripts/* / Preview/* / content.hpf 메타 읽음
- 각각을 HWP OLE 스트림 형식 (HwpSummary 구조, DocOptions 구조, Scripts zlib 등) 으로 변환
- `extra_streams` 에 추가

**장점**: HWPX 원본 데이터 보존 (스크립트/메타/preview 정확)
**단점**: 각 형식 변환 로직 필요 (Scripts 압축, HwpSummary 구조 등). 5+ 형식 구현

### B. 정적 템플릿 기반 fallback 생성 (실용 해법)

- `saved/blank2010.hwp` 에서 contract 5 스트림 추출 + 정적 자산화
- HWPX→HWP 변환 시 이 정적 스트림을 `extra_streams` 에 주입
- HWPX 컨테이너의 동등 정보가 있으면 일부 overwrite (예: Preview/PrvText)

**장점**: 구현 단순, 즉시 한컴 호환 확보 가능성
**단점**: 메타 정보 (title, creator) 가 blank 템플릿 값 (rhwp 자동 보정 필요)

### C. 하이브리드 (권고)

- A 의 정공법으로 시작 가능한 영역 (Preview/Scripts) 은 HWPX → HWP 변환
- B 의 fallback (DocOptions/_LinkDoc, HwpSummary 기본값) 은 정적 자산 활용
- 한컴 손상 미판정이 목적이므로 **모든 스트림 존재 + 최소 정합** 충족 우선

### 추가: BodyText/Section0 1202 bytes 잔존 누락

Form 컨트롤 자체의 직렬화 (Stage 40 이후 잔존) — Stage 2 에서 별도 분석 + 보강. 본 task 핵심은 5 스트림 누락이나 BodyText 도 보완 필요.

## 9. Stage 2 구현 계획서로 진행

권고 — **옵션 C 하이브리드**:
1. HwpSummary / DocOptions/_LinkDoc / Scripts (× 2) / Preview (× 2) 5 contract 스트림 작성 보장
2. 가능한 영역 (Preview / Scripts) 은 HWPX 컨테이너 데이터 활용
3. 나머지 (HwpSummary / DocOptions) 는 정적 fallback (`saved/blank2010.hwp` 추출) 활용
4. BodyText Form 컨트롤 잔존 누락은 Stage 40 진단 + 보강 (Stage 2.2)

검증 기준 — **한컴 에디터에서 form-01.hwp / form-02.hwp 변환 결과가 손상 없이 열림** (작업지시자 시각 판정 게이트).
