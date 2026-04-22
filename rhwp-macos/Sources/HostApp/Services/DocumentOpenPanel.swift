import AppKit
import UniformTypeIdentifiers

enum DocumentOpenPanel {
    @MainActor
    static func chooseDocumentURL() -> URL? {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.resolvesAliases = true
        panel.title = "Open HWP Document"
        panel.message = "Choose an HWP or HWPX document."
        panel.allowedContentTypes = supportedContentTypes

        return panel.runModal() == .OK ? panel.url : nil
    }

    private static var supportedContentTypes: [UTType] {
        var types: [UTType] = [.data]
        [
            "com.postmelee.rhwpmac.hwp",
            "com.postmelee.rhwpmac.hwpx",
            "com.hancom.hwp",
            "com.hancom.hwpx",
            "com.haansoft.hancomofficeviewer.mac.hwp",
            "com.haansoft.hancomofficeviewer.mac.hwpx"
        ].forEach { identifier in
            if let type = UTType(identifier) {
                types.append(type)
            }
        }
        return types
    }
}
