<script lang="ts">
  import { AppShell, currentSurface } from './lib/shell';

  const surface = currentSurface(window.location.search);
</script>

<AppShell {surface} />
