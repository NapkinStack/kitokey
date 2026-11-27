// swift-tools-version: 6.1
// kitokey-speech: Apple's SpeechAnalyzer for kitokey-transcription, on a Mac with macOS 26
// (D42, ADR-0012). No dependency: the system's Speech and AVFoundation alone.

import PackageDescription

let package = Package(
    name: "kitokey-speech",
    platforms: [.macOS("26")],
    targets: [
        .executableTarget(name: "kitokey-speech", path: "Sources/kitokey-speech")
    ]
)
