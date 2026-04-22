import AppKit
import SwiftUI

@main
struct HwpQuickLookApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
                .frame(minWidth: 520, minHeight: 320)
        }
    }
}

private struct ContentView: View {
    @StateObject private var status = ExtensionStatusModel()

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            HStack(spacing: 12) {
                Image(systemName: "doc.richtext")
                    .font(.system(size: 34, weight: .semibold))
                    .foregroundStyle(Color.accentColor)

                VStack(alignment: .leading, spacing: 3) {
                    Text("HWP Quick Look")
                        .font(.title2.weight(.semibold))
                    Text("Finder preview and thumbnail extensions")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
            }

            Divider()

            VStack(alignment: .leading, spacing: 12) {
                ExtensionStatusRow(
                    title: "Quick Look Preview",
                    bundleIdentifier: ExtensionStatus.preview.bundleIdentifier,
                    state: status.preview
                )
                ExtensionStatusRow(
                    title: "Quick Look Thumbnail",
                    bundleIdentifier: ExtensionStatus.thumbnail.bundleIdentifier,
                    state: status.thumbnail
                )
                Label("Supported types: HWP and HWPX.", systemImage: "doc.text.magnifyingglass")
            }
            .font(.body)

            Spacer()

            HStack(spacing: 10) {
                Button("Open Sample") {
                    SampleFile.open()
                }
                Button("Refresh Status") {
                    status.refresh()
                }
                Spacer()
                Text(BuildInfo.displayVersion)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(28)
        .task {
            status.refresh()
        }
    }
}

private struct ExtensionStatusRow: View {
    let title: String
    let bundleIdentifier: String
    let state: ExtensionRegistrationState

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Image(systemName: state.symbolName)
                .foregroundStyle(state.color)
                .frame(width: 18)
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                Text("\(state.label) · \(bundleIdentifier)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
        }
    }
}

private enum ExtensionStatus: CaseIterable, Hashable {
    case preview
    case thumbnail

    var bundleIdentifier: String {
        switch self {
        case .preview:
            "com.postmelee.rhwpmac.QLExtension"
        case .thumbnail:
            "com.postmelee.rhwpmac.ThumbnailExtension"
        }
    }
}

private enum ExtensionRegistrationState: Equatable {
    case checking
    case registered
    case missing
    case unknown

    var label: String {
        switch self {
        case .checking:
            "Checking"
        case .registered:
            "Registered"
        case .missing:
            "Not registered"
        case .unknown:
            "Unable to check"
        }
    }

    var symbolName: String {
        switch self {
        case .checking:
            "clock"
        case .registered:
            "checkmark.circle.fill"
        case .missing:
            "exclamationmark.triangle.fill"
        case .unknown:
            "questionmark.circle.fill"
        }
    }

    var color: Color {
        switch self {
        case .checking:
            .secondary
        case .registered:
            .green
        case .missing:
            .orange
        case .unknown:
            .secondary
        }
    }
}

@MainActor
private final class ExtensionStatusModel: ObservableObject {
    @Published var preview: ExtensionRegistrationState = .checking
    @Published var thumbnail: ExtensionRegistrationState = .checking

    func refresh() {
        preview = .checking
        thumbnail = .checking

        Task.detached {
            let states = Dictionary(
                uniqueKeysWithValues: ExtensionStatus.allCases.map { status in
                    (status, Self.registrationState(for: status))
                }
            )

            await MainActor.run {
                self.preview = states[.preview, default: .unknown]
                self.thumbnail = states[.thumbnail, default: .unknown]
            }
        }
    }

    nonisolated private static func registrationState(for status: ExtensionStatus) -> ExtensionRegistrationState {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/pluginkit")
        process.arguments = ["-m"]

        let output = Pipe()
        process.standardOutput = output
        process.standardError = Pipe()

        do {
            try process.run()
            process.waitUntilExit()
            guard process.terminationStatus == 0 else {
                return .unknown
            }

            let data = output.fileHandleForReading.readDataToEndOfFile()
            guard let text = String(data: data, encoding: .utf8) else {
                return .unknown
            }

            return text.contains(status.bundleIdentifier) ? .registered : .missing
        } catch {
            return .unknown
        }
    }
}

private enum SampleFile {
    static func open() {
        guard let url = Bundle.main.url(forResource: "sample", withExtension: "hwpx") else {
            return
        }
        NSWorkspace.shared.open(url)
    }
}

private enum BuildInfo {
    static var displayVersion: String {
        let bundle = Bundle.main
        let version = bundle.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "0.1.0"
        let build = bundle.object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? "1"
        return "v\(version) (\(build))"
    }
}
