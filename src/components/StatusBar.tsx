interface StatusBarProps {
  fileSize: number | null;
  modifiedChars: number;
  currentLine: number;
  currentCol: number;
  activeEncoding: string;
  detectedEncoding: string | null;
  garbledChars: number;
  totalChars: number;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function StatusBar({
  fileSize,
  modifiedChars,
  currentLine,
  currentCol,
  activeEncoding,
  detectedEncoding,
  garbledChars,
  totalChars,
}: StatusBarProps) {
  const saved = modifiedChars === 0;
  const modified = modifiedChars > 0;
  const garbled = garbledChars > 0;
  const garbledRate =
    totalChars > 0 ? ((garbledChars / totalChars) * 100).toFixed(2) : "0.00";

  return (
    <div className="status-bar">
      <div className="status-left">
        {fileSize !== null && (
          <span className="status-item">{formatFileSize(fileSize)}</span>
        )}
        <span className="status-item">
          行 {currentLine}, 列 {currentCol}
        </span>
      </div>
      <div className="status-right">
        <span className={`status-item ${saved ? "green" : "red"}`}>
          {saved ? "已保存" : "未保存"}
        </span>
        <span className={`status-item ${modified ? "red" : "green"}`}>
          {modified ? `已改: ${modifiedChars} 字符` : "未改"}
        </span>
        <span className={`status-item ${garbled ? "red" : "green"}`}>
          乱码: {garbledChars}/{totalChars} ({garbledRate}%)
        </span>
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
