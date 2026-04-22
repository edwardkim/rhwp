import Foundation
import SwiftUI

enum ExtensionStatus: CaseIterable, Hashable {
    case preview
    case thumbnail

    var title: String {
        switch self {
        case .preview:
            "Quick Look Preview"
        case .thumbnail:
            "Quick Look Thumbnail"
        }
    }

    var bundleIdentifier: String {
        switch self {
        case .preview:
            "com.postmelee.rhwpmac.QLExtension"
        case .thumbnail:
            "com.postmelee.rhwpmac.ThumbnailExtension"
        }
    }
}

enum ExtensionRegistrationState: Equatable {
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
final class ExtensionStatusModel: ObservableObject {
    @Published var preview: ExtensionRegistrationState = .checking
    @Published var thumbnail: ExtensionRegistrationState = .checking

    func state(for status: ExtensionStatus) -> ExtensionRegistrationState {
        switch status {
        case .preview:
            preview
        case .thumbnail:
            thumbnail
        }
    }

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
