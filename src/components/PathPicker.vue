<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import type { PathDialogFilter } from "../settings/catalog";
import GhostButton from "./GhostButton.vue";
import SettingsTextField from "./SettingsTextField.vue";

const model = defineModel<string>({ default: "" });

const {
  label,
  placeholder,
  mode = "directory",
  filters,
  browseLabel,
} = defineProps<{
  label?: string;
  placeholder?: string;
  mode?: "directory" | "file";
  filters?: PathDialogFilter[];
  browseLabel?: string;
}>();

const { t } = useI18n();

async function browse() {
  try {
    const selected = await open({
      multiple: false,
      directory: mode === "directory",
      defaultPath: model.value || undefined,
      filters: mode === "file" ? filters : undefined,
      title: label,
    });
    if (typeof selected === "string" && selected) {
      model.value = selected;
    }
  } catch {
    /* user cancelled or dialog unavailable */
  }
}
</script>

<template>
  <div class="path-picker">
    <SettingsTextField
      v-model="model"
      :label="label"
      :placeholder="placeholder"
    />
    <GhostButton :label="browseLabel ?? t('common.browse')" @click="browse">
      {{ browseLabel ?? t("common.browse") }}
    </GhostButton>
  </div>
</template>

<style scoped>
.path-picker {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: none;
}
</style>
