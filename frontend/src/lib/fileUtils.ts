export class DownloadableBlob extends Blob {
  filename: string;

  constructor(
    parts: BlobPart[],
    options?: BlobPropertyBag & { filename?: string },
  ) {
    super(parts, options);
    this.filename = options?.filename ?? "download";
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

export const base64ToBlob = (data: string) => {
  const binaryString = atob(data);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return new Blob([bytes], { type: "application/pdf" });
};
