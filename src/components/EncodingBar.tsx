import type { EncodingId } from "../App";

interface EncodingBarProps {
  commonEncodings: { id: EncodingId; label: string }[];
  extraEncodings: { id: EncodingId; label: string }[];
  activeEncoding: string;
  loadingEncoding: string | null;
  onEncodeClick: (id: EncodingId) => void;
}

export default function EncodingBar({
  commonEncodings,
  extraEncodings,
  activeEncoding,
  loadingEncoding,
  onEncodeClick,
}: EncodingBarProps) {
  return (
    <div className="encoding-bar">
      {commonEncodings.map((enc) => (
        <button
          key={enc.id}
          className={`encoding-btn ${activeEncoding === enc.id ? "active" : ""} ${
            loadingEncoding === enc.id ? "loading" : ""
          }`}
          onClick={() => onEncodeClick(enc.id)}
          disabled={loadingEncoding !== null}
        >
          {loadingEncoding === enc.id ? "⏳" : ""} {enc.label}
        </button>
      ))}
      <div className="encoding-more-wrapper">
        <button className="encoding-btn more-btn" disabled={loadingEncoding !== null}>
          更多 ▾
        </button>
        <div className="encoding-more-menu">
          {extraEncodings.map((enc) => (
            <button
              key={enc.id}
              className={`encoding-menu-item ${activeEncoding === enc.id ? "active" : ""}`}
              onClick={() => onEncodeClick(enc.id)}
            >
              {enc.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
