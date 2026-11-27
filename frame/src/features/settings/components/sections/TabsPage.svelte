<script lang="ts">
  import * as m from "$shared/i18n/messages";
  import { preferences } from "$domain/preferences";
  import SegmentedControl from "$shared/ui/SegmentedControl";
  import SettingsGroup from "$shared/ui/SettingsGroup";
  import SettingsRow from "$shared/ui/SettingsRow";
  import PreferenceSelect from "../PreferenceSelect.svelte";
  import PreferenceSwitch from "../PreferenceSwitch.svelte";

  let tabLayout = $derived(preferences.value("sidebar.mode") === "top" ? "top" : "sidebar");
</script>

<SettingsGroup title="Sekme Düzeni">
  <SettingsRow
    settingId="tabs.layout"
    title="Sekme konumu"
    description="Sekmeleri mevcut sol kenar çubuğunda (yanda) veya klasik tarayıcı düzeninde (yukarıda) kullan."
  >
    <SegmentedControl
      label="Sekme konumu"
      size="compact"
      value={tabLayout}
      disabled={preferences.saving()}
      options={[
        { value: "sidebar", label: "Yanda (Kenar Çubuğu)" },
        { value: "top", label: "Yukarıda (Klasik)" },
      ]}
      onchange={(v) => void preferences.set("sidebar.mode", v === "top" ? "top" : "default")}
    />
  </SettingsRow>
</SettingsGroup>

<SettingsGroup title={m.tabs_group_opening()}
  ><PreferenceSelect id="general.startup" preference="tabs.startup" /><PreferenceSelect
    id="tabs.new-position"
    preference="tabs.new-position"
  /><PreferenceSelect
    id="tabs.external"
    preference="tabs.external-position"
  /><PreferenceSwitch id="tabs.duplicates" preference="tabs.switch-to-open" /><PreferenceSwitch
    id="tabs.automatic-pip"
    preference="tabs.automatic-pip"
  /></SettingsGroup
>
<SettingsGroup title={m.tabs_group_closing()}
  ><PreferenceSelect id="tabs.close" preference="tabs.after-close" /><PreferenceSelect
    id="tabs.close-button"
    preference="tabs.close-button"
  /></SettingsGroup
>
<SettingsGroup title={m.tabs_group_archive()}
  ><PreferenceSelect id="tabs.archive" preference="tabs.archive" /></SettingsGroup
>
