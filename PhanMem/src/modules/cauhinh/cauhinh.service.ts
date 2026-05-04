
import { invoke } from "@tauri-apps/api/core";

export interface AppSettingItem {
    key: string;
    value: string;
}

async function safeInvoke<T>(
    command: string,
    args?: Record<string, unknown>
): Promise<T | null> {
    try {
        return await invoke<T>(command, args);
    } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        console.warn(`[appSettingService] Tauri invoke failed: ${command} — ${msg}`);
        return null;
    }
}

export async function getAppSettings(): Promise<AppSettingItem[]> {
    const result = await safeInvoke<AppSettingItem[]>("get_app_settings");
    return result ?? [];
}

export async function getAppSetting(key: string): Promise<string | null> {
    const items = await getAppSettings();
    const found = items.find((item) => item.key === key);
    return found?.value ?? null;
}

export async function setAppSetting(key: string, value: string): Promise<boolean> {
    const result = await safeInvoke<void>("set_app_setting", { key, value });
    return result !== null;
}

