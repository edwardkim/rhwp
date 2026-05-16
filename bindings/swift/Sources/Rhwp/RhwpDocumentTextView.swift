import Foundation
import SwiftUI

@available(iOS 13.0, macOS 10.15, *)
public struct RhwpDocumentTextView: View {
    private let inputFile: URL
    private let page: RhwpPage

    @State private var document: RhwpDocumentText?
    @State private var errorMessage: String?
    @State private var isLoading = false

    public init(inputFile: URL, page: RhwpPage = .all) {
        self.inputFile = inputFile
        self.page = page
    }

    public var body: some View {
        Group {
            if let document {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 16) {
                        ForEach(document.pages ?? []) { page in
                            RhwpTextPageView(page: page)
                        }
                    }
                    .padding()
                }
            } else if let errorMessage {
                Text(errorMessage)
                    .font(.body)
                    .foregroundColor(.secondary)
                    .padding()
            } else if isLoading {
                Text("문서를 여는 중입니다.")
                    .font(.body)
                    .foregroundColor(.secondary)
                    .padding()
            } else {
                Color.clear
                    .onAppear(perform: load)
            }
        }
        .onAppear(perform: load)
    }

    private func load() {
        guard !isLoading, document == nil, errorMessage == nil else {
            return
        }

        isLoading = true
        do {
            document = try Rhwp.readText(inputFile: inputFile, page: page)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}

@available(iOS 13.0, macOS 10.15, *)
public struct RhwpTextPageView: View {
    private let page: RhwpTextPage

    public init(page: RhwpTextPage) {
        self.page = page
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("페이지 \(page.index + 1)")
                .font(.caption)
                .foregroundColor(.secondary)
            Text(page.text)
                .font(.body)
                .textSelectionIfAvailable()
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}

private extension Text {
    @ViewBuilder
    func textSelectionIfAvailable() -> some View {
        if #available(iOS 15.0, macOS 12.0, *) {
            self.textSelection(.enabled)
        } else {
            self
        }
    }
}
