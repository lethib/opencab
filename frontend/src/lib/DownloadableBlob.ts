export type MIMEType =
  | "application/json"
  | "application/xml"
  | "application/x-www-form-urlencoded"
  | "application/javascript"
  | "application/pdf"
  | "application/zip"
  | "application/vnd.ms-excel"
  | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
  | "application/msword"
  | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
  | "application/vnd.ms-powerpoint"
  | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
  | "application/octet-stream"
  | "application/graphql"
  | "text/html"
  | "text/plain"
  | "text/css"
  | "text/javascript"
  | "text/csv"
  | "image/png"
  | "image/jpeg"
  | "image/gif"
  | "image/svg+xml"
  | "image/webp"
  | "audio/mpeg"
  | "audio/ogg"
  | "audio/wav"
  | "audio/webm"
  | "video/mp4"
  | "video/webm"
  | "video/ogg"
  | "font/woff"
  | "font/woff2"
  | "font/ttf"
  | "font/otf"
  | "multipart/form-data";

export class DownloadableBlob extends Blob {
  filename: string;

  constructor(
    parts: BlobPart[],
    options?: BlobPropertyBag & { filename?: string },
  ) {
    super(parts, options);
    this.filename = options?.filename ?? "download";
  }

  static fromBase64(
    data: string,
    type: MIMEType,
    filename?: string,
  ): DownloadableBlob {
    const binaryString = atob(data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return new DownloadableBlob([bytes], { type, filename });
  }

  download = () => {
    const url = URL.createObjectURL(this);
    const link = document.createElement("a");
    link.href = url;
    link.download = this.filename;
    link.style.display = "none"; // évite tout flash
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };
}
