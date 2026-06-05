interface StatusBarProps {
  fileSize: number | null;
  isDirty: boolean;
  currentLine: number;
  currentCol: number;
  activeEncoding: string;
  detectedEncoding: string | null;
  replacementCount: number;
  hadErrors: boolean;
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
  hadErrors,
}: StatusBarProps) {
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
      <div className="status-center">
        <span className="status-item">
          当前: <strong>{activeEncoding}</strong>
        </span>
        {detectedEncoding && (
          <span className="status-item">
            推测: {detectedEncoding}
          </span>
        )}
        {hadErrors && replacementCount > 0 && (
          <span className="status-item warning">
            替换字符: {replacementCount}
          </span>
        )}
      </div>
    </div>
  );
}
