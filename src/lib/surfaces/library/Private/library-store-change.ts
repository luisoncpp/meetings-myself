import { localizeError } from '../../../i18n';

export function libraryStoreMessage(failure: unknown): string {
  if (failure instanceof Error) return localizeError(failure.message);
  return localizeError(String(failure));
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
