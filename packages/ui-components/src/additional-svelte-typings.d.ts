declare namespace svelteHTML {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface HTMLAttributes<T> {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    "on:complete"?: (event: any) => any;
  }
}
