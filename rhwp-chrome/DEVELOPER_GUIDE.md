# HWP 웹 통합 프로토콜 — 개발자 가이드

공공 웹사이트에 `data-hwp-*` 속성을 추가하면 rhwp 확장 프로그램이 HWP 파일 링크를 자동으로 인식하여 풍부한 사용자 경험을 제공합니다.

## 빠른 시작

### 1단계: 속성 1개 추가 (최소 적용)

기존 HWP 다운로드 링크에 `data-hwp="true"` 속성만 추가하면 됩니다.

```html
<!-- 변경 전 -->
<a href="/download.do?fileId=12345">공문.hwp</a>

<!-- 변경 후 -->
<a href="/download.do?fileId=12345" data-hwp="true">공문.hwp</a>
```

이것만으로 rhwp 확장이:
- 링크 옆에 HWP 아이콘(H 배지) 표시
- 배지 클릭 시 뷰어에서 바로 열기

> **확장 미설치 사용자에게는?** `data-hwp` 속성은 HTML5 표준 `data-*` 속성이므로 브라우저가 무시합니다. 기존 동작에 영향이 없습니다.

### 2단계: 문서 정보 추가 (표준 적용)

문서 메타데이터를 추가하면 사용자에게 더 많은 정보를 제공합니다.

```html
<a href="/download.do?fileId=12345"
   data-hwp="true"
   data-hwp-title="2026년 예산 집행 계획"
   data-hwp-pages="12"
   data-hwp-size="245760">
  예산 집행 계획.hwp (240KB)
</a>
```

추가 효과:
- 배지 툴팁에 "2026년 예산 집행 계획 (12쪽) 240KB" 표시
- 마우스 호버 시 미리보기 카드 팝업

### 3단계: 풍부한 메타데이터 (고급 적용)

```html
<a href="/forms/전입신고서.hwp"
   data-hwp="true"
   data-hwp-title="전입신고서"
   data-hwp-pages="2"
   data-hwp-size="102400"
   data-hwp-author="행정안전부"
   data-hwp-date="2026-04-01"
   data-hwp-category="민원서식"
   data-hwp-description="주민등록법 제16조에 따른 전입신고서 양식"
   data-hwp-form-fields="true"
   data-hwp-thumbnail="/thumbs/전입신고서.webp">
  전입신고서.hwp
</a>
```

## 속성 사양

### 링크 속성 (`data-hwp-*`)

| 속성 | 필수 | 값 | 설명 |
|------|------|-----|------|
| `data-hwp` | ✅ | `"true"` | rhwp 확장이 이 링크를 HWP로 인식하는 마커 |
| `data-hwp-title` | 권장 | 문자열 | 문서 제목 |
| `data-hwp-pages` | 권장 | 숫자 | 총 페이지 수 |
| `data-hwp-size` | 권장 | 바이트(숫자) | 파일 크기 |
| `data-hwp-format` | 선택 | `"hwp"` / `"hwpx"` | 파일 형식 (기본: 확장자로 판단) |
| `data-hwp-author` | 선택 | 문자열 | 작성자/기관명 |
| `data-hwp-date` | 선택 | ISO 8601 | 작성일 (예: `2026-04-01`) |
| `data-hwp-category` | 선택 | 문자열 | 문서 분류 (민원서식, 공고, 보고서 등) |
| `data-hwp-description` | 선택 | 문자열 | 문서 설명 |
| `data-hwp-thumbnail` | 선택 | URL | 첫 페이지 미리보기 이미지 |
| `data-hwp-form-fields` | 선택 | `"true"` | 양식 입력 필드 포함 여부 |
| `data-hwp-print-paper` | 선택 | `"A4"` / `"B5"` 등 | 용지 크기 |
| `data-hwp-print-orientation` | 선택 | `"portrait"` / `"landscape"` | 인쇄 방향 |

### 페이지 메타 태그

페이지 전체에 적용되는 설정입니다.

```html
<head>
  <meta name="hwp:integration" content="enabled">
  <meta name="hwp:hover-preview" content="true">
  <meta name="hwp:batch-enabled" content="true">
</head>
```

| 메타 태그 | 값 | 설명 |
|----------|-----|------|
| `hwp:integration` | `"enabled"` | 이 페이지가 HWP 통합을 지원함을 선언 |
| `hwp:hover-preview` | `"true"` | 호버 미리보기 활성화 |
| `hwp:batch-enabled` | `"true"` | 일괄 다운로드 UI 활성화 (추후 지원) |

## CMS 적용 가이드

### 그누보드 (gnuboard)

게시판 스킨의 첨부파일 템플릿에서 `<a>` 태그에 속성을 추가합니다.

```php
<!-- 변경 전 -->
<a href="<?=$file['href']?>"><?=$file['source']?></a>

<!-- 변경 후 -->
<a href="<?=$file['href']?>"
   <?php if (preg_match('/\.(hwp|hwpx)$/i', $file['source'])): ?>
   data-hwp="true"
   data-hwp-title="<?=htmlspecialchars($file['source'])?>"
   data-hwp-size="<?=$file['filesize']?>"
   <?php endif; ?>>
  <?=$file['source']?>
</a>
```

### XpressEngine (XE)

스킨 파일에서 첨부파일 링크를 수정합니다.

```html
<a href="{$file->download_url}"
   <![CDATA[
   <!--@if($file->isHwp())-->
   data-hwp="true"
   data-hwp-title="{$file->source_filename}"
   data-hwp-size="{$file->file_size}"
   <!--@end-->
   ]]>>
  {$file->source_filename}
</a>
```

## 디버깅

Chrome/Edge에서는 페이지의 `<head>`에 `<meta name="hwp:integration" content="enabled">`를 추가하면 개발자 도구(F12) → 콘솔에서 디버깅 명령어를 사용할 수 있습니다. 일반 HWP 링크의 배지와 호버 미리보기에는 이 태그가 필요하지 않습니다.

```javascript
// 페이지 전체 검사 — HWP 링크 목록, 메타 태그, 개선 사항 출력
rhwpDev.inspect()

// 특정 링크 상세 검사
rhwpDev.inspectLink(document.querySelector('a[data-hwp]'))

// 프로토콜 사양 + 예제 도움말
rhwpDev.help()

// 확장 버전 확인
rhwpDev.version
```

### 자동 검출 항목

`rhwpDev.inspect()` 실행 시 다음을 자동 검출합니다:

- `data-hwp="true"` 누락 (확장자 폴백으로 감지된 링크)
- `data-hwp-size` 값이 숫자가 아닌 경우
- `data-hwp-pages` 값이 숫자가 아닌 경우
- `data-hwp-date` 값이 유효한 날짜가 아닌 경우
- 페이지 수준 `hwp:integration` 메타 태그 부재

## 확장 프로그램 감지

Chrome/Edge에서는 `hwp:integration` 메타 태그로 통합을 활성화한 페이지에서 아래 속성과 이벤트로 rhwp 설치 여부를 감지할 수 있습니다.

```javascript
// 방법 1: data 속성 확인
if (document.documentElement.dataset.hwpExtension === 'rhwp') {
  console.log('rhwp 확장 설치됨');
}

// 방법 2: 이벤트 수신
window.addEventListener('hwp-extension-ready', (e) => {
  console.log('rhwp 확장:', e.detail);
  // { name: 'rhwp', version: '0.1.0', capabilities: ['preview', 'edit', 'print'] }
});
```

## FAQ

**Q: 확장이 없는 사용자에게 영향이 있나요?**
A: 없습니다. `data-hwp-*` 속성은 HTML5 표준 `data-*` 속성이므로 브라우저가 무시합니다.

**Q: 확장자가 URL에 없는 경우는?**
A: `/download.do?fileId=123` 같은 URL은 확장자로 감지할 수 없습니다. `data-hwp="true"` 속성을 추가해야 합니다.

**Q: 파일이 서버로 전송되나요?**
A: 아닙니다. 모든 처리는 사용자 브라우저 내에서 WASM으로 수행됩니다.

**Q: 기업 환경에서 무료인가요?**
A: 네. MIT 라이선스로 개인/기업 무료입니다.
