// A minimal implementation of parsing and viewing MJPEG-over-TCP

import SwiftUI
import UIKit

struct MJPEGView: UIViewRepresentable {
    private let url: URL
    private var imageView = UIImageView()

    init(url: URL) {
        self.url = url
    }

    func makeUIView(context: Context) -> UIImageView {
        let delegate = MySessionDelegate(imageView: imageView)
        let session = URLSession(configuration: .default, delegate: delegate, delegateQueue: nil)
        let task = session.dataTask(with: url)
        task.resume()

        imageView.contentMode = .scaleAspectFit // Scale the image to fit and center it
        return imageView
    }

    func updateUIView(_ uiView: UIImageView, context: Context) {}

    class MySessionDelegate: NSObject, URLSessionDataDelegate {
        private weak var imageView: UIImageView?
        private var buffer = Data()

        init(imageView: UIImageView) {
            self.imageView = imageView
        }

        func urlSession(_ session: URLSession, dataTask: URLSessionDataTask, didReceive data: Data) {
            buffer.append(data)

            while let range = buffer.range(of: Data([0xFF, 0xD8]), options: [], in: buffer.startIndex..<buffer.endIndex),
                  let endRange = buffer.range(of: Data([0xFF, 0xD9]), options: [], in: range.upperBound..<buffer.endIndex) {
                let imageData = buffer[range.lowerBound..<endRange.upperBound]
                if let image = UIImage(data: imageData) {
                    DispatchQueue.main.async {
                        self.imageView?.image = image
                    }
                }
                buffer.removeSubrange(buffer.startIndex..<endRange.upperBound)
            }
        }
    }
}
