export interface AddedProfile {
  id: number;
  name: string;
}

export interface LoadStep {
  /** 1-based position of the profile being loaded */
  index: number;
  total: number;
  name: string;
}

export interface LoadFailure {
  profile: AddedProfile;
  message: string;
}

/**
 * Profiles created by an account import have no channels yet: load their lists one after the
 * other (the active profile first) so favorites, series progress and presets can attach to them.
 * A failing list does not stop the others.
 */
export async function loadAddedProfiles(
  profiles: AddedProfile[],
  activeProfileId: number | null,
  refresh: (playlistId: number) => Promise<unknown>,
  onStep: (step: LoadStep) => void,
  describe: (err: unknown) => string
): Promise<LoadFailure[]> {
  const ordered = [...profiles].sort(
    (a, b) => Number(b.id === activeProfileId) - Number(a.id === activeProfileId)
  );
  const failures: LoadFailure[] = [];
  for (const [i, profile] of ordered.entries()) {
    onStep({ index: i + 1, total: ordered.length, name: profile.name });
    try {
      await refresh(profile.id);
    } catch (err) {
      failures.push({ profile, message: describe(err) });
    }
  }
  return failures;
}
