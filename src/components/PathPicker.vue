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
  disabled = false,
  grow = false,
} = defineProps<{
  label?: string;
  placeholder?: string;
  mode?: "directory" | "file";
  filters?: PathDialogFilter[];
  browseLabel?: string;
  disabled?: boolean;
  grow?: boolean;
}>();

const { t } = useI18n();

async function browse() {
  if (disabled) return;
  try {
    const selected = await open({
      multiple: false,
      directory: mode === "directory",
      defaultPath: model.value || placeholder || undefined,
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
  <div class="path-picker" :class="{ grow, disabled }">
    <SettingsTextField
      v-model="model"
      :label="label"
      :placeholder="placeholder"
      :disabled="disabled"
      :grow="grow"
    />
    <GhostButton
      :label="browseLabel ?? t('common.browse')"
      :disabled="disabled"
      @click="browse"
    >
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
.path-picker.grow {
  flex: 1;
  width: 100%;
  min-width: 0;
}
.path-picker.disabled {
  opacity: 0.85;
}
</style>
