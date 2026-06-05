import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save, message, ask } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { EditorView } from "@codemirror/view";
import CodeMirrorEditor from "./editor/CodeMirrorEditor";
import Toolbar from "./components/Toolbar";
import StatusBar from "./components/StatusBar";
import EncodingBar from "./components/EncodingBar";

export type EncodingId = string;

interface DecodeResult {
  text: string;
  encoding: EncodingId;
  detected_encoding: EncodingId | null;
  file_size: number;
  had_errors: boolean;
  replacement_count: number;
  bom: string | null;
  revision: number;
}

interface SaveResult {
  path: string;
  file_size: number;
  revision: number;
}

interface EncodingOption {
  id: EncodingId;
  label: string;
  category: string;
}

interface ToastState {
  message: string;
  onConfirm: () => void;
  confirmLabel: string;
}

export default function App() {
  const editorRef = useRef<EditorView | null>(null);
  const [filePath, setFilePath] = useState<string | null>(null);
  const [encoding, setEncoding] = useState<EncodingId>("UTF-8");
  const [detectedEncoding, setDetectedEncoding] = useState<EncodingId | null>(null);
  const [fileSize, setFileSize] = useState<number | null>(null);
  const [isDirty, setIsDirty] = useState(false);
  const [hadErrors, setHadErrors] = useState(false);
  const [replacementCount, setReplacementCount] = useState(0);
  const [currentLine, setCurrentLine] = useState(1);
  const [currentCol, setCurrentCol] = useState(1);
  const [loadingEncoding, setLoadingEncoding] = useState<string | null>(null);
  const [encodingOptions, setEncodingOptions] = useState<EncodingOption[]>([]);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const seqRef = useRef(0);
  const dirtyRef = useRef(false);

  dirtyRef.current = isDirty;

  useEffect(() => {
    invoke<EncodingOption[]>("get_supported_encodings")
      .then(setEncodingOptions)
      .catch(console.error);
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWindow()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "over") {
          setDragOver(true);
        } else if (payload.type === "leave") {
          setDragOver(false);
        } else if (payload.type === "drop") {
          setDragOver(false);
          const paths = payload.paths;
          if (paths.length > 0) {
            const targetPath = paths[0];
            if (dirtyRef.current) {
              setToast({
                message: "当前内容有未保存修改。继续操作会丢弃这些修改，是否继续？",
                confirmLabel: "继续",
                onConfirm: () => {
                  setToast(null);
                  handleOpenPath(targetPath);
                },
              });
            } else {
              handleOpenPath(targetPath);
            }
          }
        }
      })
      .then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  const handleOpenPath = useCallback(async (path: string) => {
    try {
      setLoadingEncoding(path);
      const result = await invoke<DecodeResult>("open_file", { path });
      applyDecodeResult(result);
      setFilePath(path);
    } catch (err) {
      await message(String(err), { title: "错误", kind: "error" });
    } finally {
      setLoadingEncoding(null);
    }
  }, []);

  const applyDecodeResult = useCallback((result: DecodeResult) => {
    const view = editorRef.current;
    if (view) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: result.text,
        },
      });
    }
    setEncoding(result.encoding);
    setDetectedEncoding(result.detected_encoding);
    setFileSize(result.file_size);
    setHadErrors(result.had_errors);
    setReplacementCount(result.replacement_count);
    setIsDirty(false);
    setCurrentLine(1);
    setCurrentCol(1);
  }, []);

  const handleOpen = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "文本文件", extensions: ["txt", "*"] }],
    });
    if (selected) {
      const targetPath = typeof selected === "string" ? selected : selected;
      if (dirtyRef.current) {
        setToast({
          message: "当前内容有未保存修改。继续操作会丢弃这些修改，是否继续？",
          confirmLabel: "继续",
          onConfirm: () => {
            setToast(null);
            handleOpenPath(targetPath);
          },
        });
      } else {
        handleOpenPath(targetPath);
      }
    }
  }, [handleOpenPath]);

  const handleSave = useCallback(async () => {
    if (!filePath) {
      await message("请先打开一个文件", { title: "提示", kind: "info" });
      return;
    }
    const view = editorRef.current;
    if (!view) return;
    const text = view.state.doc.toString();
    try {
      await invoke<SaveResult>("save_current", {
        text,
        encoding,
      });
      setIsDirty(false);
    } catch (err) {
      await message(`保存失败: ${err}`, { title: "错误", kind: "error" });
    }
  }, [filePath, encoding]);

  const handleSaveAs = useCallback(async () => {
    const view = editorRef.current;
    if (!view) return;
    const selected = await save({
      filters: [{ name: "文本文件", extensions: ["txt", "*"] }],
    });
    if (!selected) return;
    const targetPath = typeof selected === "string" ? selected : selected;
    const text = view.state.doc.toString();
    try {
      const result = await invoke<SaveResult>("save_as", {
        path: targetPath,
        text,
        encoding,
      });
      setFilePath(result.path);
      setFileSize(result.file_size);
      setIsDirty(false);
    } catch (err) {
      await message(`另存为失败: ${err}`, { title: "错误", kind: "error" });
    }
  }, [encoding]);

  const handleEncodeClick = useCallback(
    async (encId: EncodingId) => {
      if (encId === encoding && !isDirty) return;

      if (isDirty) {
        const confirmed = await ask(
          "当前内容有未保存修改。继续操作会丢弃这些修改，是否继续？",
          { title: "确认", kind: "warning" }
        );
        if (!confirmed) return;
      }

      const seq = ++seqRef.current;
      setLoadingEncoding(encId);
      try {
        const result = await invoke<DecodeResult>("decode_current_as", {
          encoding: encId,
        });
        if (seq !== seqRef.current) return;
        applyDecodeResult(result);
      } catch (err) {
        if (seq !== seqRef.current) return;
        await message(String(err), { title: "解码错误", kind: "error" });
      } finally {
        if (seq === seqRef.current) {
          setLoadingEncoding(null);
        }
      }
    },
    [encoding, isDirty, applyDecodeResult]
  );

  const handleContentChange = useCallback(() => {
    setIsDirty(true);
  }, []);

  const handleCursorChange = useCallback((line: number, col: number) => {
    setCurrentLine(line);
    setCurrentCol(col);
  }, []);

  const commonEncodings = encodingOptions
    .filter((e) => e.category === "common")
    .map((e) => ({ id: e.id, label: e.label }));
  const extraEncodings = encodingOptions
    .filter((e) => e.category === "extra")
    .map((e) => ({ id: e.id, label: e.label }));

  return (
    <>
      <Toolbar
        filePath={filePath}
        isDirty={isDirty}
        onOpen={handleOpen}
        onSave={handleSave}
        onSaveAs={handleSaveAs}
      />
      <CodeMirrorEditor
        editorRef={editorRef}
        onContentChange={handleContentChange}
        onCursorChange={handleCursorChange}
      />
      <StatusBar
        fileSize={fileSize}
        isDirty={isDirty}
        currentLine={currentLine}
        currentCol={currentCol}
        activeEncoding={encoding}
        detectedEncoding={detectedEncoding}
        replacementCount={replacementCount}
        hadErrors={hadErrors}
      />
      <EncodingBar
        commonEncodings={commonEncodings}
        extraEncodings={extraEncodings}
        activeEncoding={encoding}
        loadingEncoding={loadingEncoding}
        onEncodeClick={handleEncodeClick}
      />

      {dragOver && (
        <div className="drag-overlay">
          <span className="drag-overlay-text">释放以打开文本文件</span>
        </div>
      )}

      {toast && (
        <div className="toast">
          <div className="toast-message">{toast.message}</div>
          <div className="toast-buttons">
            <button
              className="toast-btn"
              onClick={() => setToast(null)}
            >
              取消
            </button>
            <button
              className="toast-btn primary"
              onClick={toast.onConfirm}
            >
              {toast.confirmLabel}
            </button>
          </div>
        </div>
      )}
    </>
  );
}
