/** 메인 문서만 연결한다. 비교용 bridge는 브라우저 제목을 변경하지 않는다. */
export function installDocumentTitle(
  bridge: {
    readonly fileName: string;
    hasLoadedDocument(): boolean;
    onFileNameChanged?: (fileName: string) => void;
  },
): void {
  // Chromium 설치형 창은 앱 이름을 직접 붙이므로 페이지 제목에는 파일명만 둔다.
  // 일반 브라우저의 전체 화면은 설치형 창으로 분류하지 않는다.
  const appModes = window.matchMedia(
    '(display-mode: standalone), (display-mode: minimal-ui), (display-mode: window-controls-overlay)',
  );
  const update = () => {
    document.title = bridge.hasLoadedDocument()
      ? (appModes.matches ? bridge.fileName : `${bridge.fileName} - rhwp-studio`)
      : 'rhwp-studio';
  };
  bridge.onFileNameChanged = update;
  appModes.addEventListener('change', update);
  update();
}
