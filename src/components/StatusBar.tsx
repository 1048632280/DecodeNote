interface StatusBarProps {
  fileSize: number | null;
  isDirty: boolean;
  currentLine: number;
  currentCol: number;
  activeEncoding: string;
  detectedEncoding: string | null;
  replacementCount: number;
  totalChars: number;
  hadErrors: boolean;
  modifiedChars: number;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function StatusBar({
  fileSize,
  isDirty,
  currentLine,
  currentCol,
  activeEncoding,
  detectedEncoding,
  replacementCount,
  totalChars,
  hadErrors,
  modifiedChars,
}: StatusBarProps) {
  const garbledRate =
    totalChars > 0 ? ((replacementCount / totalChars) * 100).toFixed(2) : "0.00";

  return (
    <div className="status-bar">
      <div className="status-left">
        {fileSize !== null && (
          <span className="status-item">{formatFileSize(fileSize)}</span>
        )}
        {isDirty && <span className="status-item dirty">已修改</span>}
        <span className="status-item">
          行 {currentLine}, 列 {currentCol}
        </span>
      </div>
      <div className="status-right">
        {fileSize !== null && modifiedChars > 0 && (
          <span className="status-item warning">
            已改: {modifiedChars} 字符
          </span>
        )}
        {fileSize !== null && hadErrors && replacementCount > 0 && (
          <span className="status-item warning">
            乱码: {replacementCount}/{totalChars} ({garbledRate}%)
          </span>
        )}
        <span className="status-item">
          <strong>{activeEncoding}</strong>
        </span>
        {detectedEncoding && detectedEncoding !== activeEncoding && (
          <span className="status-item" style={{ color: "var(--text-muted)" }}>
            推测:{detectedEncoding}
          </span>
        )}
      </div>
    </div>
  );
}
