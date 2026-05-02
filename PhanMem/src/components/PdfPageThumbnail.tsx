import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Document, Page, pdfjs } from "react-pdf";

import pdfWorker from "pdfjs-dist/build/pdf.worker.min.mjs?url";

pdfjs.GlobalWorkerOptions.workerSrc = pdfWorker;

interface PdfPageThumbnailProps {
  filePath: string;
  fileType?: string;
  pageNumber?: number;
  width?: number;
  label?: string;
}

export function PdfPageThumbnail({
  filePath,
  fileType,
  pageNumber = 1,
  width = 92,
  label,
}: PdfPageThumbnailProps) {
  const [assetUrl, setAssetUrl] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  const typeLower = (fileType || filePath.split(".").pop() || "").toLowerCase();
  const isPdf = typeLower.includes("pdf");
  const isImage = ["jpg", "jpeg", "png", "gif", "webp", "tif", "tiff"].some((ext) =>
    typeLower.includes(ext)
  );

  useEffect(() => {
    if (!filePath) return;
    try {
      const url = convertFileSrc(filePath);
      console.info("[PdfPageThumbnail] convertFileSrc ok", { filePath, assetUrl: url, pageNumber });
      setAssetUrl(url);
      setFailed(false);
    } catch (e) {
      console.error("[PdfPageThumbnail] convertFileSrc failed", { filePath, pageNumber, error: e });
      setFailed(true);
    }
  }, [filePath, pageNumber]);

  if (!assetUrl || failed) {
    return (
      <div className="doc-preview-sheet">
        <span>PDF</span>
        <span>{label ?? `Trang ${pageNumber}`}</span>
      </div>
    );
  }

  if (isImage) {
    return (
      <div className="pdf-thumb-real" style={{ width }}>
        <img src={assetUrl} alt={label ?? `Trang ${pageNumber}`} />
      </div>
    );
  }

  if (!isPdf) {
    return (
      <div className="doc-preview-sheet">
        <span>FILE</span>
        <span>{label ?? (typeLower || "Tệp")}</span>
      </div>
    );
  }

  return (
    <div className="pdf-thumb-real" style={{ width }}>
      <Document
        file={assetUrl}
        loading={<div className="pdf-thumb-skeleton" />}
        onLoadError={() => setFailed(true)}
        error={
          <div className="doc-preview-sheet">
            <span>PDF</span>
            <span>{label ?? `Trang ${pageNumber}`}</span>
          </div>
        }
      >
        <Page
          pageNumber={pageNumber}
          width={width}
          renderTextLayer={false}
          renderAnnotationLayer={false}
        />
      </Document>
    </div>
  );
}
