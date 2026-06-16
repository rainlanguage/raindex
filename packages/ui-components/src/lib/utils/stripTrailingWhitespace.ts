/**
 * Removes trailing whitespace from rainlang/dotrain editor text, mirroring the
 * "delete trailing whitespace on save" behaviour of common code editors.
 *
 * - Trailing spaces and tabs are removed from the end of every line.
 * - Line breaks between content are preserved (`\r\n` is normalised to `\n`).
 * - Internal blank lines are preserved (only their trailing whitespace is
 *   stripped) so deliberate spacing in the source is kept.
 * - Trailing blank lines and a trailing newline at the very end of the text are
 *   removed, so the saved text never ends in dangling whitespace.
 * - Leading whitespace (indentation) is never touched.
 *
 * @param text - The editor contents to normalise.
 * @returns The text with trailing whitespace stripped.
 */
export function stripTrailingWhitespace(text: string): string {
  return text
    .split(/\r\n|\r|\n/)
    .map((line) => line.replace(/[ \t]+$/, ""))
    .join("\n")
    .replace(/\n+$/, "");
}
