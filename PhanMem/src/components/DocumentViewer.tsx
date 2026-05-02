import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { TransformWrapper, TransformComponent } from "react-zoom-pan-pinch";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";

import pdfWorker from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { PdfPageThumbnail } from "./PdfPageThumbnail";

// Configure PDF.js worker
pdfjs.GlobalWorkerOptions.workerSrc = pdfWorker;

export interface DocumentViewerProps {
  documentPath: string;
  documentType: string;
  initialPage?: number;
  highlightQuery?: string;
  ocrBlocks?: OcrOverlayBlock[];
  showOcrOverlay?: boolean;
  activeBlockId?: string | null;
  onPageChange?: (page: number) => void;
  onOcrBlockSelect?: (block: OcrOverlayBlock) => void;
  onAddSelectionToAi?: (text: string, pageNumber: number) => void;
  onCreateNoteFromSelection?: (text: string, pageNumber: number) => void;
}

export interface OcrOverlayBlock {
  id: string;
  block_type: string;
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
  page_width: number | null;
  page_height: number | null;
  confidence: number;
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function renderHighlightedText(value: string, query?: string) {
  const q = query?.trim();
  if (!q) return value;
  const escaped = escapeRegExp(q);
  if (!escaped) return value;
  return value.replace(new RegExp(`(${escaped})`, "gi"), "<mark>$1</mark>");
}

export function DocumentViewer({
  documentPath,
  documentType,
  initialPage = 1,
  highlightQuery,
  ocrBlocks = [],
  showOcrOverlay = false,
  activeBlockId,
  onPageChange,
  onOcrBlockSelect,
  onAddSelectionToAi,
  onCreateNoteFromSelection,
}: DocumentViewerProps) {
  const [assetUrl, setAssetUrl] = useState<string | null>(null);
  const [numPages, setNumPages] = useState<number>();
  const [pageNumber, setPageNumber] = useState<number>(1);
  const [pdfWidth, setPdfWidth] = useState<number>(800);
  const [error, setError] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; text: string } | null>(null);
  const thumbnailWindowStart = Math.max(1, pageNumber - 4);
  const thumbnailWindowEnd = Math.min(numPages ?? 1, pageNumber + 4);

  const typeLower = documentType.toLowerCase();
  const isPdf = typeLower.includes("pdf");
  const isImage = ["jpg", "jpeg", "png", "gif", "webp"].some((ext) =>
    typeLower.includes(ext)
  );

  useEffect(() => {
    if (!documentPath) return;
    try {
      // convertFileSrc converts a local path into a secure asset:// URL that Tauri can load
      const url = convertFileSrc(documentPath);
      console.info("[DocumentViewer] convertFileSrc ok", {
        documentPath,
        documentType,
        assetUrl: url,
      });
      setAssetUrl(url);
      setError(null);
      setPageNumber(Math.max(1, initialPage));
    } catch (e) {
      console.error("Error converting file src:", e);
      setError("Không thể tải tài liệu từ đường dẫn cục bộ.");
    }
  }, [documentPath, initialPage]);

  function onDocumentLoadSuccess({ numPages }: { numPages: number }) {
    setNumPages(numPages);
  }

  useEffect(() => {
    onPageChange?.(pageNumber);
  }, [onPageChange, pageNumber]);

  useEffect(() => {
    const onClick = () => setContextMenu(null);
    window.addEventListener("click", onClick);
    return () => window.removeEventListener("click", onClick);
  }, []);

  useEffect(() => {
    if (!numPages) return;
    setPageNumber(Math.min(Math.max(1, initialPage), numPages));
  }, [initialPage, numPages]);

  useEffect(() => {
    if (!isPdf) {
      return;
    }

    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.key === "ArrowLeft") {
        setPageNumber((p) => Math.max(1, p - 1));
      } else if (e.key === "ArrowRight") {
        setPageNumber((p) => Math.min(numPages ?? p, p + 1));
      } else if (e.key === "+") {
        setPdfWidth((w) => Math.min(1600, w + 100));
      } else if (e.key === "-") {
        setPdfWidth((w) => Math.max(400, w - 100));
      }
    };

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [isPdf, numPages]);

  if (error) {
    return <div className="viewer-empty-state">⚠️ {error}</div>;
  }

  if (!assetUrl) {
    return <div className="viewer-empty-state">Đang tải...</div>;
  }

  if (isImage) {
    return (
      <div className="viewer-container image-viewer">
        <TransformWrapper
          initialScale={1}
          minScale={0.5}
          maxScale={5}
          centerOnInit
        >
          <TransformComponent wrapperStyle={{ width: "100%", height: "100%" }}>
            <img
              src={assetUrl}
              alt="Document"
              style={{ maxWidth: "100%", maxHeight: "100%", objectFit: "contain" }}
            />
          </TransformComponent>
        </TransformWrapper>
      </div>
    );
  }

  if (isPdf) {
    const overlayScale = ocrBlocks[0]?.page_width ? pdfWidth / (ocrBlocks[0].page_width || pdfWidth) : 1;
    return (
      <div className="viewer-container pdf-viewer">
        <div className="toolbar" style={{ justifyContent: "center", marginBottom: 0 }}>
          <button className="btn btn-icon" onClick={() => setPdfWidth((w) => Math.max(400, w - 100))} title="Zoom out">
            −
          </button>
          <button className="btn btn-sm" onClick={() => setPdfWidth(800)}>
            Fit
          </button>
          <button className="btn btn-icon" onClick={() => setPdfWidth((w) => Math.min(1600, w + 100))} title="Zoom in">
            +
          </button>
          <div className="toolbar-separator" />
          <button
            className="btn btn-icon"
            disabled={pageNumber <= 1}
            onClick={() => setPageNumber((p) => p - 1)}
            title="Trang trước"
          >
            ◀
          </button>
          <span className="text-xs">Trang</span>
          <input
            type="number"
            min={1}
            max={numPages || 1}
            value={pageNumber}
            onChange={(e) => {
              const v = parseInt(e.target.value, 10);
              if (!isNaN(v) && v >= 1 && v <= (numPages || 1)) {
                setPageNumber(v);
              }
            }}
            className="form-input text-mono text-xs"
            style={{ width: 58, textAlign: "center" }}
          />
          <span className="text-mono text-xs">/ {numPages || "--"}</span>
          <button
            className="btn btn-icon"
            disabled={numPages === undefined || pageNumber >= numPages}
            onClick={() => setPageNumber((p) => p + 1)}
            title="Trang tiếp"
          >
            ▶
          </button>
        </div>
        {highlightQuery?.trim() ? (
          <div className="viewer-highlight-banner">
            Đang mở kết quả tìm kiếm: <mark>{highlightQuery.trim()}</mark>
          </div>
        ) : null}
        <div className="pdf-reader-layout">
          <aside className="pdf-thumbnail-panel">
            {Array.from({ length: numPages ?? 0 }, (_, i) => i + 1).map((page) => (
              <button
                key={page}
                className={`pdf-thumbnail ${page === pageNumber ? "active" : ""}`}
                onClick={() => setPageNumber(page)}
                title={`Trang ${page}`}
              >
                {page >= thumbnailWindowStart && page <= thumbnailWindowEnd ? (
                  <PdfPageThumbnail
                    filePath={documentPath}
                    fileType={documentType}
                    pageNumber={page}
                    width={54}
                    label={`Trang ${page}`}
                  />
                ) : (
                  <div className="doc-preview-sheet" style={{ width: 54, height: 76 }}>
                    <span style={{ fontSize: 14 }}>PDF</span>
                  </div>
                )}
                <span>Trang {page}</span>
              </button>
            ))}
          </aside>
          <div
            className="pdf-content"
            onContextMenu={(e) => {
              const selectedText = window.getSelection()?.toString().trim();
              if (!selectedText) return;
              e.preventDefault();
              setContextMenu({ x: e.clientX, y: e.clientY, text: selectedText });
            }}
          >
            <Document
              file={assetUrl}
              onLoadSuccess={onDocumentLoadSuccess}
              onLoadError={(err) => {
                console.error("[DocumentViewer] PDF load error", {
                  message: err.message,
                  documentPath,
                  assetUrl,
                });
                setError(`Không thể mở PDF (${err.message}). Kiểm tra file tồn tại và Tauri asset protocol.`);
              }}
              loading={<div className="viewer-empty-state">⏳ Đang tải PDF...</div>}
            >
              <div className="pdf-page-overlay-stage" style={{ width: pdfWidth }}>
                <Page
                  pageNumber={pageNumber}
                  renderTextLayer={true}
                  renderAnnotationLayer={true}
                  width={pdfWidth}
                  customTextRenderer={(textItem) => renderHighlightedText(textItem.str, highlightQuery)}
                />
                {showOcrOverlay ? (
                  <div className="ocr-overlay-layer" aria-label="OCR overlay">
                    {ocrBlocks.map((block) => {
                      const cls =
                        block.confidence < 0.62
                          ? "error"
                          : block.block_type === "possible_handwriting" || block.block_type === "low_confidence"
                            ? "warning"
                            : block.block_type === "stamp" || block.block_type === "signature"
                              ? "mark"
                              : "ok";
                      return (
                        <button
                          key={block.id}
                          className={`ocr-overlay-block ${cls} ${activeBlockId === block.id ? "active" : ""}`}
                          style={{
                            left: block.x * overlayScale,
                            top: block.y * overlayScale,
                            width: Math.max(8, block.width * overlayScale),
                            height: Math.max(8, block.height * overlayScale),
                          }}
                          title={`${block.block_type} · ${Math.round(block.confidence * 100)}% · ${block.text}`}
                          onClick={(e) => {
                            e.stopPropagation();
                            onOcrBlockSelect?.(block);
                          }}
                        />
                      );
                    })}
                  </div>
                ) : null}
              </div>
            </Document>
            {contextMenu ? (
              <div
                className="viewer-selection-menu"
                style={{ left: contextMenu.x, top: contextMenu.y }}
                onClick={(e) => e.stopPropagation()}
              >
                <button
                  onClick={() => {
                    onAddSelectionToAi?.(contextMenu.text, pageNumber);
                    setContextMenu(null);
                  }}
                >
                  Thêm vào AI
                </button>
                <button
                  onClick={() => {
                    onCreateNoteFromSelection?.(contextMenu.text, pageNumber);
                    setContextMenu(null);
                  }}
                >
                  Tạo ghi chú
                </button>
              </div>
            ) : null}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="viewer-empty-state">
      <div style={{ fontSize: "48px", marginBottom: "16px" }}>📄</div>
      <div>Không thể hiển thị loại file này trực tiếp ({documentType}).</div>
      <div style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginTop: "8px" }}>
        Đường dẫn: {documentPath}
      </div>
    </div>
  );
}
