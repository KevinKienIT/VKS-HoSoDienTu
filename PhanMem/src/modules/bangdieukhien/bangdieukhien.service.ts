/**
 * bangdieukhien.service.ts — Service cho Bang dieu khien (Dashboard)
 *
 * Dashboard la man hinh tong hop, no can goi nhieu service khac nhau.
 * File nay re-export cac ham can thiet tu cac module lien quan
 * de giu dung quy chuan moi module co service rieng.
 */

// Re-export tu cac module lien quan
export { listCases, type CaseSummary } from "../hosovuan/hosovuan.service";
export {
  listDocuments,
  getDocumentGroups,
  type DocumentSummary,
  type DocumentGroup,
} from "../quantailieu/quantailieu.service";
export { checkAiStatus, type AiStatus } from "../phantichai/phantichai.service";
