/**
 * VKS ECMS — Module Registry
 *
 * Central registry of all available modules.
 * Each module has an id, metadata, version list, and dynamic loaders.
 */

export interface ModuleDefinition {
  id: string;
  name: string;
  description: string;
  icon: string; // emoji or icon name
  category: "viewer" | "processor" | "ai" | "organizer";
  versions: string[];
  defaultVersion: string;
  requires?: string[];
  tauriCommands?: string[];
}

export interface ModuleConfig {
  module_id: string;
  enabled: boolean;
  selected_version: string;
  settings: string; // JSON
  updated_at: string;
}

// All available modules — IDs theo quy chuan tieng Viet khong dau (07_module_map.md)
export const MODULE_DEFINITIONS: ModuleDefinition[] = [
  {
    id: "quettailieu",
    name: "Quét Tài Liệu",
    description: "OCR quét và nhận dạng văn bản từ ảnh/PDF",
    icon: "📄",
    category: "processor",
    versions: ["v1", "v2"],
    defaultVersion: "v1",
  },
  {
    id: "hosovuan",
    name: "Hồ Sơ Vụ Án",
    description: "Phân nhóm, tổ chức hồ sơ theo tiêu chí linh hoạt",
    icon: "📁",
    category: "organizer",
    versions: ["v1"],
    defaultVersion: "v1",
  },
  {
    id: "phantichtailieu",
    name: "Phân Tích Tài Liệu",
    description: "Phóng to ảnh giữ độ nét cao, phân tích layout",
    icon: "🔍",
    category: "processor",
    versions: ["v1"],
    defaultVersion: "v1",
  },
  {
    id: "phantichai",
    name: "Phân Tích AI",
    description: "Tóm tắt, hỏi đáp, lên slide từ hồ sơ (offline LLM)",
    icon: "🤖",
    category: "ai",
    versions: ["v1"],
    defaultVersion: "v1",
    requires: [],
    tauriCommands: ["ai_check_status", "ai_summarize_case", "ai_ask_case"],
  },
  {
    id: "quantailieu",
    name: "Quản Tài Liệu",
    description: "Side-by-side, annotation, citation highlight",
    icon: "👁",
    category: "viewer",
    versions: ["v1"],
    defaultVersion: "v1",
  },
  {
    id: "duyethotro",
    name: "Dòng Thời Gian",
    description: "Timeline tương tác với sự kiện, nhân vật, vật chứng",
    icon: "📅",
    category: "organizer",
    versions: ["v1"],
    defaultVersion: "v1",
  },
];

export function getModuleDefinition(
  moduleId: string
): ModuleDefinition | undefined {
  return MODULE_DEFINITIONS.find((m) => m.id === moduleId);
}

export function getDefaultConfigs(): ModuleConfig[] {
  return MODULE_DEFINITIONS.map((m) => ({
    module_id: m.id,
    enabled: false,
    selected_version: m.defaultVersion,
    settings: "{}",
    updated_at: new Date().toISOString(),
  }));
}
