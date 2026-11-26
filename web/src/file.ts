/** Lets the browser save `data` as a file called `filename`. */
export function download(data: BlobPart, filename: string, type = 'application/octet-stream') {
  const url = URL.createObjectURL(new Blob([data], { type }));
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  // The download has started once `click()` returns; the URL is not needed afterwards.
  setTimeout(() => URL.revokeObjectURL(url));
}

/** The text of the first file chosen in a file input, or `undefined` if none is chosen. */
export async function readChosenFile(input: HTMLInputElement): Promise<string | undefined> {
  return input.files?.[0]?.text();
}
