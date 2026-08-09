export function libraryStoreMessage(failure: unknown): string {
  return failure instanceof Error ? failure.message : String(failure);
}

export async function runLibraryMutation(
  mutation: () => Promise<unknown>,
  onError: (message: string | null) => void,
  reload: () => Promise<void>,
): Promise<void> {
  try {
    await mutation();
    onError(null);
  } catch (failure) {
    onError(libraryStoreMessage(failure));
    return;
  }
  await reload();
}
