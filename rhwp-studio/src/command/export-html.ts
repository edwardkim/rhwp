// 문서 전체 HTML 조립과 HTML / Word(.doc) 내보내기 페이로드 생성.
//
// 엔진의 `exportSelectionHtml` 은 섹션 단위 선택 범위를 클립보드용
// 래퍼(`<html><body><!--StartFragment-->…`)로 반환한다. 여기서는 각 섹션의
// 전체 범위(첫 문단 0 오프셋 ~ 마지막 문단 끝)를 이어붙여 문서 HTML 을
// 만들고, 저장 가능한 완전한 HTML 문서 또는 Word 호환 문서로 감싼다.
//
// `.doc` 는 OOXML(.docx)이 아니라 Word 가 표준적으로 여는 "HTML 기반 Word
// 문서"다. 충실도는 HTML 수준(레이아웃/고급 서식 일부 손실)이며, 엔진에
// docx 직렬화가 없는 상황에서 브라우저만으로 가능한 현실적인 출력이다.

/** 내보내기에 필요한 엔진 표면 (WasmBridge 의 부분집합) */
export interface DocumentHtmlEngine {
  getSectionCount(): number;
  getParagraphCount(sectionIdx: number): number;
  getParagraphLength(sectionIdx: number, paraIdx: number): number;
  exportSelectionHtml(
    sectionIdx: number,
    startPara: number,
    startOffset: number,
    endPara: number,
    endOffset: number,
  ): string;
}

export type HtmlExportFormat = 'html' | 'doc';

export interface HtmlExportFile {
  fileName: string;
  content: string;
  mimeType: string;
}

export const HTML_EXPORT_DETAILS: Record<
  HtmlExportFormat,
  { extension: string; mimeType: string; label: string }
> = {
  html: { extension: 'html', mimeType: 'text/html;charset=utf-8', label: 'HTML' },
  doc: { extension: 'doc', mimeType: 'application/msword', label: 'Word 문서(.doc)' },
};

/**
 * 엔진이 반환하는 조각의 클립보드용 래퍼(`<html><body><!--StartFragment-->…`)를
 * 벗겨 body 내부만 남긴다. 래퍼가 없으면 원문을 그대로 돌려준다.
 */
export function unwrapEngineHtmlFragment(fragment: string): string {
  const bodyMatch = fragment.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  const inner = bodyMatch ? bodyMatch[1] : fragment;
  return inner
    .replace(/<!--StartFragment-->/gi, '')
    .replace(/<!--EndFragment-->/gi, '')
    .trim();
}

/** 문서 전체 HTML 을 섹션별 selection HTML 로 조립한다. */
export function collectDocumentHtml(engine: DocumentHtmlEngine): string {
  const sectionCount = Math.max(0, engine.getSectionCount());
  const parts: string[] = [];
  for (let section = 0; section < sectionCount; section += 1) {
    const paragraphCount = engine.getParagraphCount(section);
    if (paragraphCount <= 0) continue;
    const lastPara = paragraphCount - 1;
    try {
      const endOffset = engine.getParagraphLength(section, lastPara);
      const fragment = engine.exportSelectionHtml(section, 0, 0, lastPara, endOffset) ?? '';
      const inner = unwrapEngineHtmlFragment(fragment);
      if (inner) parts.push(inner);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`구역 ${section + 1} HTML 변환 실패: ${message}`);
    }
  }
  return parts.join('\n');
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

function wrapHtmlDocument(innerHtml: string, title: string): string {
  return `<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="utf-8" />
<title>${escapeHtml(title)}</title>
</head>
<body>
${innerHtml}
</body>
</html>`;
}

/** Word 가 HTML 기반 문서로 여는 래퍼 (mso 네임스페이스 포함). */
function wrapWordDocument(innerHtml: string, title: string): string {
  return `<html xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:w="urn:schemas-microsoft-com:office:word" xmlns="http://www.w3.org/TR/REC-html40">
<head>
<meta charset="utf-8" />
<title>${escapeHtml(title)}</title>
</head>
<body>
${innerHtml}
</body>
</html>`;
}

/** "문서.hwp" → "문서". 이름이 비면 기본값을 쓴다. */
export function htmlExportBaseName(fileName: string | undefined): string {
  const name = (fileName ?? '').trim() || '문서';
  return name.replace(/\.(hwpx?|hml|html?|doc)$/i, '');
}

/** 현재 문서를 지정 형식의 저장 가능한 파일 페이로드로 만든다. */
export function buildHtmlExportFile(
  engine: DocumentHtmlEngine,
  format: HtmlExportFormat,
  sourceFileName: string | undefined,
): HtmlExportFile {
  const details = HTML_EXPORT_DETAILS[format];
  const baseName = htmlExportBaseName(sourceFileName);
  const innerHtml = collectDocumentHtml(engine);
  const content = format === 'doc'
    ? wrapWordDocument(innerHtml, baseName)
    : wrapHtmlDocument(innerHtml, baseName);
  return {
    fileName: `${baseName}.${details.extension}`,
    content,
    mimeType: details.mimeType,
  };
}
