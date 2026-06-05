interface ToolbarProps {
  filePath: string | null;
  isDirty: boolean;
  onOpen: () => void;
  onSave: () => void;
  onSaveAs: () => void;
}

export default function Toolbar({
  filePath,
  isDirty,
  onOpen,
  onSave,
  onSaveAs,
}: ToolbarProps) {
  const displayPath = filePath
    ? filePath.length > 60
      ? "..." + filePath.slice(-57)
      : filePath
    : null;

  return (
    <div className="toolbar">
      <div className="toolbar-left">
        <span className="app-title">DecodeNote</span>
        <button className="toolbar-btn" onClick={onOpen} title="打开文件">
          📂 打开
        </button>
        <button className="toolbar-btn" onClick={onSave} title="保存">
          💾 保存
        </button>
        <button className="toolbar-btn" onClick={onSaveAs} title="另存为">
          📄 另存为
        </button>
      </div>
      <div className="toolbar-right">
        <span className="file-path" title={filePath ?? ""}>
          {displayPath ?? "未打开文件"}
          {isDirty && displayPath ? " *" : ""}
        </span>
      </div>
    </div>
  );
}
