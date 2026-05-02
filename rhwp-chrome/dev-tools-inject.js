// rhwp DevTools — 페이지 컨텍스트에 주입되는 스크립트
// 브라우저 콘솔에서 rhwpDev.inspect() 등으로 사용
(function() {
  'use strict';
  if (window.rhwpDev) return;

  const VERSION = '0.2.2';

  window.rhwpDev = {
    inspect() {
      const links = document.querySelectorAll('a[href]');
      const hwpLinks = [];
      const issues = [];

      for (const a of links) {
        const href = a.href || '';
        const isMarked = a.getAttribute('data-hwp') === 'true';
        const isExt = /\.(hwp|hwpx)(\?.*)?$/i.test(href);

        if (!isMarked && !isExt) continue;

        const info = {
          href: a.href,
          text: a.textContent.trim().substring(0, 50),
          detected: isMarked ? 'data-hwp' : 'extension',
          attrs: {}
        };

        for (const attr of a.attributes) {
          if (attr.name.startsWith('data-hwp')) {
            info.attrs[attr.name] = attr.value;
          }
        }

        if (isExt && !isMarked) {
          issues.push({ link: a.href, issue: 'data-hwp="true" 누락 — 확장자 폴백으로 감지됨' });
        }
        if (info.attrs['data-hwp-size'] && isNaN(Number(info.attrs['data-hwp-size']))) {
          issues.push({ link: a.href, issue: 'data-hwp-size 값이 숫자가 아님' });
        }
        if (info.attrs['data-hwp-pages'] && isNaN(Number(info.attrs['data-hwp-pages']))) {
          issues.push({ link: a.href, issue: 'data-hwp-pages 값이 숫자가 아님' });
        }
        if (info.attrs['data-hwp-date'] && isNaN(Date.parse(info.attrs['data-hwp-date']))) {
          issues.push({ link: a.href, issue: 'data-hwp-date 값이 유효한 날짜가 아님 (ISO 8601 권장)' });
        }

        hwpLinks.push(info);
      }

      const metas = {};
      for (const meta of document.querySelectorAll('meta[name^="hwp:"]')) {
        metas[meta.name] = meta.content;
      }

      console.group('%c[rhwp DevTools]%c 페이지 검사 결과', 'color:#2563eb;font-weight:bold', 'color:inherit');
      console.log('rhwp 확장 버전:', VERSION);
      console.log('HWP 링크:', hwpLinks.length + '개');
      if (hwpLinks.length > 0) console.table(hwpLinks);

      if (Object.keys(metas).length > 0) {
        console.log('페이지 메타 태그:');
        console.table(metas);
      } else {
        console.log('%c페이지 메타 태그 없음%c — <meta name="hwp:integration" content="enabled"> 추가를 권장합니다', 'color:orange', 'color:inherit');
      }

      if (issues.length > 0) {
        console.warn('개선 사항:', issues.length + '건');
        console.table(issues);
      } else if (hwpLinks.length > 0) {
        console.log('%c✓ 개선 사항 없음', 'color:green');
      }

      console.groupEnd();
      return { links: hwpLinks, metas, issues };
    },

    inspectLink(anchor) {
      if (!anchor || anchor.tagName !== 'A') {
        console.error('[rhwp DevTools] <a> 태그를 전달해주세요');
        return;
      }

      const attrs = {};
      for (const attr of anchor.attributes) {
        if (attr.name.startsWith('data-hwp')) {
          attrs[attr.name] = attr.value;
        }
      }

      const recommended = ['data-hwp', 'data-hwp-title', 'data-hwp-pages', 'data-hwp-size'];
      const optional = ['data-hwp-author', 'data-hwp-date', 'data-hwp-thumbnail',
                         'data-hwp-category', 'data-hwp-format', 'data-hwp-description',
                         'data-hwp-form-fields', 'data-hwp-print-orientation', 'data-hwp-print-paper'];

      console.group('%c[rhwp DevTools]%c 링크 검사', 'color:#2563eb;font-weight:bold', 'color:inherit');
      console.log('URL:', anchor.href);
      console.log('텍스트:', anchor.textContent.trim());
      if (Object.keys(attrs).length > 0) console.table(attrs);

      const missing = recommended.filter(a => !attrs[a]);
      if (missing.length > 0) console.warn('권장 속성 누락:', missing.join(', '));

      const available = optional.filter(a => !attrs[a]);
      if (available.length > 0) console.log('선택 속성 (추가 가능):', available.join(', '));

      console.groupEnd();
      return { attrs, missing, available };
    },

    help() {
      console.log(`%c[rhwp DevTools]%c data-hwp-* 프로토콜 가이드

%c=== 최소 적용 (속성 1개) ===%c
<a href="file.hwp" data-hwp="true">문서.hwp</a>

%c=== 표준 적용 (권장 속성) ===%c
<a href="file.hwp"
   data-hwp="true"
   data-hwp-title="문서 제목"
   data-hwp-pages="12"
   data-hwp-size="245760">문서.hwp</a>

%c=== 고급 적용 (미리보기 + 양식) ===%c
<a href="file.hwp"
   data-hwp="true"
   data-hwp-title="전입신고서"
   data-hwp-pages="2"
   data-hwp-thumbnail="https://example.com/thumbs/preview.webp"
   data-hwp-category="민원서식"
   data-hwp-form-fields="true">전입신고서</a>

%c=== 페이지 메타 태그 ===%c
<meta name="hwp:integration" content="enabled">
<meta name="hwp:hover-preview" content="true">

%c=== 디버깅 명령어 ===%c
rhwpDev.inspect()           — 페이지 전체 검사
rhwpDev.inspectLink(elem)   — 특정 링크 상세 검사
rhwpDev.help()              — 이 도움말
rhwpDev.version             — 확장 버전`,
        'color:#2563eb;font-weight:bold', 'color:inherit',
        'color:#16a34a;font-weight:bold', 'color:inherit',
        'color:#16a34a;font-weight:bold', 'color:inherit',
        'color:#16a34a;font-weight:bold', 'color:inherit',
        'color:#16a34a;font-weight:bold', 'color:inherit',
        'color:#16a34a;font-weight:bold', 'color:inherit');
    },

    version: VERSION
  };

  console.log('%c[rhwp]%c DevTools 사용 가능 — rhwpDev.help() 로 시작하세요', 'color:#2563eb;font-weight:bold', 'color:inherit');
})();
