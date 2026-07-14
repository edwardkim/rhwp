import AppKit

/// [Task #2267] Quick Look 확장의 호스트 앱.
///
/// Quick Look 확장은 **앱 번들 안에만 존재할 수 있고**, 앱이 한 번 실행되어
/// Launch Services 에 등록되어야 Finder 가 확장을 집어든다. 이 앱 자체는
/// 설치 안내와 상태 확인만 한다.
@main
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var window: NSWindow?

    func applicationDidFinishLaunching(_ notification: Notification) {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 520, height: 260),
            styleMask: [.titled, .closable, .miniaturizable],
            backing: .buffered,
            defer: false
        )
        window.title = "rhwp Quick Look"
        window.center()

        let text = NSTextField(wrappingLabelWithString: """
        rhwp Quick Look 확장이 설치되었습니다.

        Finder 에서 .hwp / .hwpx / .hml 파일을 선택하고 스페이스바를 누르면
        미리보기가 표시됩니다.

        미리보기가 뜨지 않으면 터미널에서 확장 등록 상태를 확인하세요:
            pluginkit -m -p com.apple.quicklook.preview

        참고: 미리보기는 앞쪽 3페이지까지만 렌더합니다 (확장 메모리 한도).
        """)
        text.font = .systemFont(ofSize: 13)
        text.translatesAutoresizingMaskIntoConstraints = false

        let content = NSView()
        content.addSubview(text)
        NSLayoutConstraint.activate([
            text.leadingAnchor.constraint(equalTo: content.leadingAnchor, constant: 24),
            text.trailingAnchor.constraint(equalTo: content.trailingAnchor, constant: -24),
            text.topAnchor.constraint(equalTo: content.topAnchor, constant: 24),
        ])
        window.contentView = content
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)

        self.window = window
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }
}
