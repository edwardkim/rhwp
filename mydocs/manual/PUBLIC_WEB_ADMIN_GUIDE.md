---
kind: guide
status: active
canonical: mydocs/manual/PUBLIC_WEB_ADMIN_GUIDE.md
last_verified: 2026-07-17
---

# HWP 웹 통합 가이드 — 웹 관리자/개발자용

## 개요

이 문서는 공공기관, 학교, 기업 등의 **웹사이트 관리자와 개발자**를 대상으로 합니다.

웹사이트에 게시된 HWP 파일 링크에 간단한 HTML 속성을 추가하면, rhwp 브라우저 확장 프로그램이 해당 링크를 자동으로 인식하여 사용자에게 다음과 같은 향상된 경험을 제공합니다:

- HWP 링크 옆에 **열기 아이콘(배지)** 자동 표시
- 마우스 호버 시 **문서 정보 미리보기 카드** 표시
- 배지 클릭 시 **브라우저에서 바로 열기** (별도 프로그램 설치 불필요)
- 파일이 서버로 전송되지 않음 (사용자 브라우저 내 WASM 처리)

> **핵심 원칙**: 확장 프로그램이 설치되지 않은 사용자에게는 아무런 영향이 없습니다. 추가하는 `data-hwp-*` 속성은 HTML5 표준 `data-*` 속성이므로 브라우저가 무시합니다.

---

## 왜 필요한가?

### 현재 문제

공공기관 웹사이트에서 HWP 파일 다운로드 링크는 대부분 이런 형태입니다:

```
https://www.example.go.kr/common/download.do?fileId=145705507
```

이 URL에는 `.hwp` 확장자가 없어서 rhwp 확장 프로그램이 **자동으로 감지할 수 없습니다**. 확장자가 URL에 포함된 경우(`/files/공문.hwp`)에만 자동 감지가 가능합니다.

### 해결 방법

HTML 속성 하나(`data-hwp="true"`)만 추가하면 확장자가 없는 URL도 감지됩니다.

---

## 적용 방법

### 최소 적용 — 속성 1개 (5분)

기존 다운로드 링크에 `data-hwp="true"` 속성 하나만 추가합니다.

```html
<!-- 변경 전 -->
<a href="/common/download.do?fileId=145705507">공문_2026-001.hwp</a>

<!-- 변경 후 -->
<a href="/common/download.do?fileId=145705507" data-hwp="true">공문_2026-001.hwp</a>
```

**효과:**
- 링크 옆에 파란색 **H** 배지 표시
- 배지 클릭 시 rhwp 뷰어에서 바로 열기

### 표준 적용 — 문서 정보 추가 (권장)

문서의 제목, 페이지 수, 파일 크기를 추가하면 사용자에게 더 많은 정보를 제공합니다.

```html
<a href="/common/download.do?fileId=145705507"
   data-hwp="true"
   data-hwp-title="2026년 예산 집행 계획"
   data-hwp-pages="12"
   data-hwp-size="245760">
  예산 집행 계획.hwp (240KB)
</a>
```

**추가 효과:**
- 배지 툴팁에 "2026년 예산 집행 계획 (12쪽) 240KB" 표시
- 마우스 호버 시 **미리보기 카드** 팝업

### 고급 적용 — 풍부한 메타데이터

민원서식, 보도자료 등 자주 접근하는 문서에 상세 정보를 추가합니다.

```html
<a href="/forms/transfer_in.hwp"
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

**추가 효과:**
- 미리보기 카드에 작성자, 날짜, 카테고리, 설명, 썸네일 표시
- 양식 문서 표시

---

## 속성 사양

### 링크 속성 (`data-hwp-*`)

| 속성 | 필수 | 값 예시 | 설명 |
|------|------|---------|------|
| `data-hwp` | ✅ | `"true"` | HWP 파일 링크 마커 |
| `data-hwp-title` | 권장 | `"전입신고서"` | 문서 제목 |
| `data-hwp-pages` | 권장 | `"12"` | 총 페이지 수 |
| `data-hwp-size` | 권장 | `"245760"` | 파일 크기 (바이트) |
| `data-hwp-format` | 선택 | `"hwp"` / `"hwpx"` | 파일 형식 |
| `data-hwp-author` | 선택 | `"행정안전부"` | 작성자/기관명 |
| `data-hwp-date` | 선택 | `"2026-04-01"` | 작성일 (ISO 8601) |
| `data-hwp-category` | 선택 | `"민원서식"` | 문서 분류 |
| `data-hwp-description` | 선택 | `"전입신고서 양식"` | 문서 설명 |
| `data-hwp-thumbnail` | 선택 | `"/thumbs/doc.webp"` | 미리보기 이미지 URL |
| `data-hwp-form-fields` | 선택 | `"true"` | 양식 입력 필드 포함 여부 |
| `data-hwp-print-paper` | 선택 | `"A4"` | 인쇄 용지 크기 |
| `data-hwp-print-orientation` | 선택 | `"portrait"` / `"landscape"` | 인쇄 방향 |

### 페이지 수준 메타 태그

페이지의 `<head>`에 추가하여 페이지 전체 설정을 선언합니다.

```html
<head>
  <!-- 이 페이지가 HWP 통합을 지원함을 선언 -->
  <meta name="hwp:integration" content="enabled">
  
  <!-- 호버 미리보기 활성화 -->
  <meta name="hwp:hover-preview" content="true">
</head>
```

---

## CMS별 적용 가이드

### 그누보드 (gnuboard)

게시판 스킨의 첨부파일 템플릿(`view_file_default.skin.php` 등)에서 다운로드 링크를 수정합니다.

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

게시판 스킨의 첨부파일 출력 부분을 수정합니다.

```html
<a href="{$file->download_url}"
   <!--@if(preg_match('/\.(hwp|hwpx)$/i', $file->source_filename))-->
   data-hwp="true"
   data-hwp-title="{$file->source_filename}"
   data-hwp-size="{$file->file_size}"
   <!--@end-->>
  {$file->source_filename}
</a>
```

### WordPress

첨부파일 출력 필터를 추가합니다.

```php
// functions.php
add_filter('wp_get_attachment_link', function($link, $id) {
    $file = get_attached_file($id);
    if (preg_match('/\.(hwp|hwpx)$/i', $file)) {
        $title = get_the_title($id);
        $size = filesize($file);
        $link = str_replace('<a ', "<a data-hwp=\"true\" data-hwp-title=\"{$title}\" data-hwp-size=\"{$size}\" ", $link);
    }
    return $link;
}, 10, 2);
```

### 직접 HTML 관리

정적 HTML 페이지에서는 각 HWP 링크에 직접 속성을 추가합니다.

```html
<table class="board">
  <tr>
    <td>2026-127</td>
    <td>
      2026년도 제2차 정기이사회 회의록
      <div class="attach">
        첨부:
        <a href="/files/meeting_2026_02.hwp"
           data-hwp="true"
           data-hwp-title="2026년도 제2차 정기이사회 회의록"
           data-hwp-pages="15"
           data-hwp-size="358400"
           data-hwp-category="회의록">
          회의록.hwp (350KB)
        </a>
      </div>
    </td>
  </tr>
</table>
```

---

## 디버깅 도구

rhwp 확장이 설치된 브라우저에서 **F12 → 콘솔**을 열면 디버깅 명령어를 사용할 수 있습니다.

### 페이지 전체 검사

```javascript
rhwpDev.inspect()
```

출력 내용:
- 감지된 HWP 링크 목록 (감지 방법: data-hwp / 확장자)
- 페이지 메타 태그 존재 여부
- **개선 사항** (누락된 속성, 잘못된 값)

### 특정 링크 검사

```javascript
rhwpDev.inspectLink(document.querySelector('a[data-hwp]'))
```

출력 내용:
- 현재 설정된 속성 목록
- 권장 속성 중 누락된 것
- 추가 가능한 선택 속성

### 도움말

```javascript
rhwpDev.help()
```

프로토콜 사양, 적용 예제, 명령어 전체 목록을 출력합니다.

### 자동 검출 항목

`rhwpDev.inspect()` 실행 시 다음을 자동 검출합니다:

| 항목 | 설명 |
|------|------|
| `data-hwp="true"` 누락 | 확장자로 감지되었지만 마커 없음 |
| `data-hwp-size` 값 오류 | 숫자가 아닌 값 (예: `"big"`) |
| `data-hwp-pages` 값 오류 | 숫자가 아닌 값 (예: `"많음"`) |
| `data-hwp-date` 값 오류 | 유효하지 않은 날짜 (ISO 8601 권장) |
| 메타 태그 부재 | `hwp:integration` 메타 태그 없음 |

---

## 확장 프로그램 감지

웹사이트에서 rhwp 확장 설치 여부를 감지하여 조건부 UI를 제공할 수 있습니다.

### 방법 1: data 속성 확인

```javascript
if (document.documentElement.dataset.hwpExtension === 'rhwp') {
  // 확장 설치됨 — HWP 링크에 "브라우저에서 열기" 안내 표시
  document.querySelectorAll('a[data-hwp]').forEach(a => {
    a.title = '클릭하면 브라우저에서 바로 열 수 있습니다';
  });
}
```

### 방법 2: 이벤트 수신

```javascript
window.addEventListener('hwp-extension-ready', (e) => {
  console.log('rhwp 확장 감지:', e.detail);
  // { name: 'rhwp', version: '0.1.0', capabilities: ['preview', 'edit', 'print'] }

  // 확장이 있으면 "HWP 뷰어 설치 안내" 배너 숨기기
  document.getElementById('hwp-install-banner')?.remove();
});
```

### 활용 예시

```html
<!-- 확장 미설치 시 표시되는 안내 배너 -->
<div id="hwp-install-banner" style="background:#fef3c7; padding:8px 16px; border-radius:4px; font-size:13px;">
  💡 <a href="https://chromewebstore.google.com/...">rhwp 확장 프로그램</a>을 설치하면
  HWP 파일을 브라우저에서 바로 열 수 있습니다.
</div>

<script>
  // 확장이 설치되어 있으면 배너 숨기기
  if (document.documentElement.dataset.hwpExtension === 'rhwp') {
    document.getElementById('hwp-install-banner')?.remove();
  }
</script>
```

---

## 자주 묻는 질문

**Q: 속성을 추가하면 확장이 없는 사용자에게 영향이 있나요?**

A: 없습니다. `data-hwp-*` 속성은 HTML5 표준 `data-*` 속성이므로 모든 브라우저가 무시합니다. 기존 다운로드 링크가 그대로 동작합니다.

**Q: 확장자가 URL에 없는 경우에도 감지되나요?**

A: `data-hwp="true"` 속성을 추가하면 감지됩니다. `/download.do?fileId=123` 같은 URL은 확장자가 없으므로 반드시 이 속성이 필요합니다.

**Q: 사용자의 파일이 서버로 전송되나요?**

A: 아닙니다. rhwp 확장은 모든 파일 처리를 사용자 브라우저 내에서 WebAssembly(WASM)로 수행합니다. 외부 서버와 통신하지 않습니다.

**Q: 어떤 브라우저에서 동작하나요?**

A: Chrome, Edge, Whale, Arc 등 Chromium 기반 브라우저에서 동작합니다.

**Q: 기업 환경에서 무료인가요?**

A: 네. MIT 라이선스로 개인/기업 모두 무료입니다.

**Q: 기존 CMS를 수정하지 않고 적용할 수 있나요?**

A: URL에 `.hwp`/`.hwpx` 확장자가 포함된 경우에는 수정 없이 자동 감지됩니다. 확장자가 없는 다운로드 URL은 `data-hwp="true"` 속성 추가가 필요합니다.

**Q: HWPX 파일도 지원하나요?**

A: 네. HWP와 HWPX 모두 열기를 지원합니다. 저장은 현재 HWP 형식이며, HWPX 저장은 추후 지원 예정입니다.

---

## 관련 링크

- [rhwp GitHub](https://github.com/edwardkim/rhwp)
- [개인정보 처리방침](https://github.com/edwardkim/rhwp/blob/main/rhwp-chrome/PRIVACY.md)
- [Chrome Web Store](https://chromewebstore.google.com/detail/pgakpjflombjmehnebnbpnalhegaanag) (게시 완료)
- [Microsoft Edge Add-ons](#) (심사 중)
