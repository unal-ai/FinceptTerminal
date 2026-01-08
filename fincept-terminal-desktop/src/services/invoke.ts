/**
 * Unified Invoke Service
 * 
 * This service provides a unified API for calling backend commands that works
 * in both Tauri (desktop) and web environments.
 * 
 * - In Tauri: Uses native IPC via @tauri-apps/api/core invoke
 * - In Web: Uses HTTP fetch to /api/rpc endpoint
 * 
 * Usage:
 *   import { invoke } from '@/services/invoke';
 *   const result = await invoke('get_market_quote', { symbol: 'AAPL' });
 */

// Environment detection
export const IS_TAURI = typeof window !== 'undefined' && '__TAURI__' in window;
export const IS_WEB = !IS_TAURI;

// API base URL for web mode
const API_BASE = import.meta.env.VITE_API_URL || '/api';

/**
 * Unified invoke function that works in both Tauri and web environments
 * 
 * @param cmd - Command name (e.g., 'get_market_quote')
 * @param args - Command arguments as an object
 * @returns Promise resolving to the command result
 */
export async function invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (IS_TAURI) {
    // Use native Tauri IPC
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return tauriInvoke<T>(cmd, args);
  } else {
    // Use HTTP RPC for web
    return invokeWeb<T>(cmd, args);
  }
}

/**
 * Web-mode invoke using HTTP fetch
 */
async function invokeWeb<T>(cmd: string, args: Record<string, unknown>): Promise<T> {
  const response = await fetch(`${API_BASE}/rpc`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ cmd, args }),
  });

  if (!response.ok) {
    throw new Error(`RPC call failed: ${response.status} ${response.statusText}`);
  }

  const result = await response.json();
  
  // Handle RPC response format - throw on explicit error or failed success flag
  if (result.error || result.success === false) {
    throw new Error(result.error || 'Unknown error');
  }

  // Return the data if it exists, otherwise return the whole result
  return result.data !== undefined ? result.data : result;
}

/**
 * Type-safe wrapper for commands with known return types
 */
export const commands = {
  // Market Data Commands
  getMarketQuote: (symbol: string) => 
    invoke<MarketQuoteResponse>('get_market_quote', { symbol }),
  
  getMarketQuotes: (symbols: string[]) => 
    invoke<MarketQuotesResponse>('get_market_quotes', { symbols }),
  
  getHistoricalData: (symbol: string, startDate: string, endDate: string) =>
    invoke<HistoricalResponse>('get_historical_data', { symbol, startDate, endDate }),
  
  getStockInfo: (symbol: string) =>
    invoke<StockInfoResponse>('get_stock_info', { symbol }),
  
  getFinancials: (symbol: string) =>
    invoke<FinancialsResponse>('get_financials', { symbol }),
  
  getPeriodReturns: (symbol: string) =>
    invoke<PeriodReturnsResponse>('get_period_returns', { symbol }),
  
  checkMarketDataHealth: () =>
    invoke<boolean>('check_market_data_health', {}),
  
  // Database Commands
  dbCheckHealth: () =>
    invoke<HealthCheckResponse>('db_check_health', {}),
  
  dbGetAllSettings: () =>
    invoke<Setting[]>('db_get_all_settings', {}),
  
  dbGetSetting: (key: string) =>
    invoke<string | null>('db_get_setting', { key }),
  
  dbSaveSetting: (key: string, value: string, category?: string) =>
    invoke<{ saved: boolean }>('db_save_setting', { key, value, category }),
  
  // Watchlist Commands
  dbGetWatchlists: () =>
    invoke<Watchlist[]>('db_get_watchlists', {}),
  
  dbCreateWatchlist: (name: string, description?: string, color?: string) =>
    invoke<Watchlist>('db_create_watchlist', { name, description, color }),
  
  dbGetWatchlistStocks: (watchlistId: string) =>
    invoke<WatchlistStock[]>('db_get_watchlist_stocks', { watchlistId }),
  
  dbAddWatchlistStock: (watchlistId: string, symbol: string, notes?: string) =>
    invoke<WatchlistStock>('db_add_watchlist_stock', { watchlistId, symbol, notes }),
  
  dbRemoveWatchlistStock: (watchlistId: string, symbol: string) =>
    invoke<{ removed: boolean }>('db_remove_watchlist_stock', { watchlistId, symbol }),
  
  dbDeleteWatchlist: (watchlistId: string) =>
    invoke<{ deleted: boolean }>('db_delete_watchlist', { watchlistId }),
  
  // Credential Commands
  dbGetCredentials: () =>
    invoke<Credential[]>('db_get_credentials', {}),
  
  dbSaveCredential: (credential: Credential) =>
    invoke<{ success: boolean; message: string }>('db_save_credential', credential),
  
  dbGetCredentialByService: (serviceName: string) =>
    invoke<Credential | null>('db_get_credential_by_service', { serviceName }),
  
  dbDeleteCredential: (id: number) =>
    invoke<{ success: boolean; message: string }>('db_delete_credential', { id }),
  
  // LLM Config Commands
  dbGetLlmConfigs: () =>
    invoke<LlmConfig[]>('db_get_llm_configs', {}),
  
  dbSaveLlmConfig: (config: LlmConfig) =>
    invoke<{ saved: boolean }>('db_save_llm_config', config),
  
  dbGetLlmGlobalSettings: () =>
    invoke<LlmGlobalSettings>('db_get_llm_global_settings', {}),
  
  dbSaveLlmGlobalSettings: (settings: LlmGlobalSettings) =>
    invoke<{ saved: boolean }>('db_save_llm_global_settings', settings),
  
  // Chat Session Commands
  dbCreateChatSession: (title: string) =>
    invoke<ChatSession>('db_create_chat_session', { title }),
  
  dbGetChatSessions: (limit?: number) =>
    invoke<ChatSession[]>('db_get_chat_sessions', { limit }),
  
  dbAddChatMessage: (message: ChatMessage) =>
    invoke<ChatMessage>('db_add_chat_message', message),
  
  dbGetChatMessages: (sessionUuid: string) =>
    invoke<ChatMessage[]>('db_get_chat_messages', { sessionUuid }),
  
  dbDeleteChatSession: (sessionUuid: string) =>
    invoke<{ deleted: boolean }>('db_delete_chat_session', { sessionUuid }),
  
  // Data Source Commands
  dbGetAllDataSources: () =>
    invoke<DataSource[]>('db_get_all_data_sources', {}),
  
  dbSaveDataSource: (source: DataSource) =>
    invoke<{ success: boolean; message: string; id?: string }>('db_save_data_source', source),
  
  dbDeleteDataSource: (id: string) =>
    invoke<{ success: boolean; message: string }>('db_delete_data_source', { id }),
  
  // Portfolio Commands
  dbListPortfolios: () =>
    invoke<Portfolio[]>('db_list_portfolios', {}),
  
  dbGetPortfolio: (portfolioId: string) =>
    invoke<Portfolio | null>('db_get_portfolio', { portfolioId }),
  
  dbCreatePortfolio: (name: string, currency?: string, description?: string) =>
    invoke<{ id: string; created: boolean }>('db_create_portfolio', { name, currency, description }),
  
  dbDeletePortfolio: (portfolioId: string) =>
    invoke<{ deleted: boolean }>('db_delete_portfolio', { portfolioId }),
  
  // Setup Commands
  checkSetupStatus: () =>
    invoke<SetupStatus>('check_setup_status', {}),
  
  // Utility Commands
  greet: (name: string) =>
    invoke<string>('greet', { name }),
  
  sha256Hash: (input: string) =>
    invoke<string>('sha256_hash', { input }),
};

// Type definitions for API responses
export interface MarketQuoteResponse {
  success: boolean;
  data?: {
    symbol: string;
    price: number;
    change: number;
    change_percent: number;
    volume?: number;
    high?: number;
    low?: number;
    open?: number;
    previous_close?: number;
    timestamp: number;
  };
  error?: string;
}

export interface MarketQuotesResponse {
  success: boolean;
  data: Array<{
    symbol: string;
    price: number;
    change: number;
    change_percent: number;
    volume?: number;
    timestamp: number;
  }>;
  error?: string;
}

export interface HistoricalResponse {
  success: boolean;
  data: Array<{
    symbol: string;
    timestamp: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
    adj_close: number;
  }>;
  error?: string;
}

export interface StockInfoResponse {
  success: boolean;
  data?: Record<string, unknown>;
  error?: string;
}

export interface FinancialsResponse {
  success: boolean;
  data?: Record<string, unknown>;
  error?: string;
}

export interface PeriodReturnsResponse {
  symbol: string;
  seven_day: number;
  thirty_day: number;
}

export interface HealthCheckResponse {
  status: string;
  message: string;
}

export interface Setting {
  setting_key: string;
  setting_value: string;
  category?: string;
  updated_at: string;
}

export interface Watchlist {
  id: string;
  name: string;
  description?: string;
  color: string;
  created_at: string;
  updated_at: string;
}

export interface WatchlistStock {
  id: string;
  watchlist_id: string;
  symbol: string;
  notes?: string;
  added_at: string;
}

export interface Credential {
  id?: number;
  service_name: string;
  username?: string;
  password?: string;
  api_key?: string;
  api_secret?: string;
  additional_data?: string;
  created_at?: string;
  updated_at?: string;
}

export interface LlmConfig {
  provider: string;
  api_key?: string;
  base_url?: string;
  model?: string;
  is_active: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface LlmGlobalSettings {
  temperature?: number;
  max_tokens?: number;
  system_prompt?: string;
}

export interface ChatSession {
  session_uuid: string;
  title: string;
  message_count: number;
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  id: string;
  session_uuid: string;
  role: string;
  content: string;
  timestamp?: string;
  provider?: string;
  model?: string;
  tokens_used?: number;
}

export interface DataSource {
  id: string;
  alias: string;
  display_name: string;
  description?: string;
  ds_type: string;
  provider: string;
  category: string;
  config?: string;
  enabled: boolean;
  tags?: string;
  created_at?: string;
  updated_at?: string;
}

export interface Portfolio {
  id: string;
  name: string;
  owner: string;
  currency: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface SetupStatus {
  needs_setup: boolean;
  python_installed: boolean;
  database_ready: boolean;
}

// ============================================================================
// EVENT LISTENER (Tauri events / Web fallback)
// ============================================================================

/**
 * Unlisten function type - call to unsubscribe from an event
 */
export type UnlistenFn = () => void;

/**
 * Event payload wrapper type
 */
export interface Event<T> {
  payload: T;
}

/**
 * Listen to Tauri events in desktop mode, no-op in web mode
 * 
 * In web mode, events should be handled via WebSocket or Server-Sent Events
 * This function provides a compatible API that won't break web builds
 * 
 * @param event - Event name to listen to
 * @param handler - Callback function to handle the event
 * @returns Promise resolving to an unlisten function
 */
export async function listen<T>(
  event: string,
  handler: (event: Event<T>) => void
): Promise<UnlistenFn> {
  if (IS_TAURI) {
    // Use native Tauri event listener
    const { listen: tauriListen } = await import('@tauri-apps/api/event');
    return tauriListen<T>(event, handler);
  } else {
    // In web mode, return no-op (events should be handled via WebSocket/SSE)
    console.warn(`[Web Mode] Event listener for '${event}' is not available. Use WebSocket for real-time updates.`);
    return () => {}; // No-op unlisten function
  }
}

// ============================================================================
// PATH UTILITIES (Tauri path API / Web fallback)
// ============================================================================

/**
 * Get the application data directory
 * In web mode, returns a sensible default path
 */
export async function appDataDir(): Promise<string> {
  if (IS_TAURI) {
    const { appDataDir: tauriAppDataDir } = await import('@tauri-apps/api/path');
    return tauriAppDataDir();
  } else {
    // In web mode, return a relative path (actual storage would be server-side)
    return '/app/data';
  }
}

/**
 * Join path segments
 * In web mode, uses simple string concatenation with forward slashes
 */
export async function joinPath(...paths: string[]): Promise<string> {
  if (IS_TAURI) {
    const { join: tauriJoin } = await import('@tauri-apps/api/path');
    return tauriJoin(...paths);
  } else {
    // In web mode, use forward slash path joining while preserving leading slashes
    // Edge case handling: empty array or all empty strings returns empty string
    if (paths.length === 0) {
      return '';
    }
    const result = paths
      .map((p, i) => i === 0 ? p.replace(/\/+$/g, '') : p.replace(/^\/+|\/+$/g, ''))
      .filter(p => p.length > 0)
      .join('/');
    return result;
  }
}

// ============================================================================
// HTTP FETCH (Tauri plugin-http / Web native fetch)
// ============================================================================

/**
 * Unified fetch function that uses Tauri's HTTP plugin in desktop mode
 * and native fetch in web mode
 */
export async function tauriFetch(
  url: string | URL | Request,
  options?: RequestInit
): Promise<Response> {
  if (IS_TAURI) {
    const { fetch: pluginFetch } = await import('@tauri-apps/plugin-http');
    return pluginFetch(url, options);
  } else {
    // In web mode, use native fetch
    return fetch(url, options);
  }
}

// ============================================================================
// SHELL UTILITIES (Tauri plugin-shell / Web fallback)
// ============================================================================

/**
 * Open a URL or path in the default application
 */
export async function shellOpen(path: string): Promise<void> {
  if (IS_TAURI) {
    const { open } = await import('@tauri-apps/plugin-shell');
    return open(path);
  } else {
    // In web mode, open in new tab with security attributes to prevent reverse tabnabbing
    window.open(path, '_blank', 'noopener,noreferrer');
  }
}

// ============================================================================
// URL OPENER (Tauri plugin-opener / Web fallback)
// ============================================================================

/**
 * Open a URL in the default browser
 */
export async function openUrl(url: string): Promise<void> {
  if (IS_TAURI) {
    const { openUrl: tauriOpenUrl } = await import('@tauri-apps/plugin-opener');
    return tauriOpenUrl(url);
  } else {
    // In web mode, open in new tab with security attributes to prevent reverse tabnabbing
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}

// ============================================================================
// EVENT EMIT (Tauri event API / Web fallback)
// ============================================================================

/**
 * Emit an event (for Tauri internal communication)
 */
export async function emit(event: string, payload?: unknown): Promise<void> {
  if (IS_TAURI) {
    const { emit: tauriEmit } = await import('@tauri-apps/api/event');
    return tauriEmit(event, payload);
  } else {
    // In web mode, no-op (use WebSocket for real-time communication)
    console.warn(`[Web Mode] Event emit for '${event}' is not available.`);
  }
}

// ============================================================================
// FILE DIALOG (Tauri plugin-dialog / Web fallback)
// ============================================================================

export interface DialogFilter {
  name: string;
  extensions: string[];
}

export interface OpenDialogOptions {
  multiple?: boolean;
  directory?: boolean;
  filters?: DialogFilter[];
  defaultPath?: string;
  title?: string;
}

export interface SaveDialogOptions {
  filters?: DialogFilter[];
  defaultPath?: string;
  title?: string;
}

/**
 * Open a file dialog to select files
 */
export async function openDialog(options?: OpenDialogOptions): Promise<string | string[] | null> {
  if (IS_TAURI) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    return open(options);
  } else {
    // In web mode, use HTML file input
    // Note: Returns file names (not full paths) as web browsers don't expose full paths for security
    // Note: The input element is not appended to DOM; browser GC handles cleanup
    return new Promise((resolve) => {
      let resolved = false;
      const input = document.createElement('input');
      input.type = 'file';
      input.multiple = options?.multiple ?? false;
      if (options?.filters?.length) {
        input.accept = options.filters.flatMap(f => f.extensions.map(e => `.${e}`)).join(',');
      }
      
      // Handle cancel - browsers fire focus event when file dialog is cancelled
      // Use a 300ms delay to allow onchange to fire first if a file was selected
      // (This delay accounts for the browser's internal event processing)
      const handleCancel = () => {
        setTimeout(() => {
          if (!resolved && !input.files?.length) {
            resolved = true;
            resolve(null);
          }
        }, 300);
      };
      window.addEventListener('focus', handleCancel, { once: true });
      
      input.onchange = () => {
        resolved = true;
        if (input.files?.length) {
          const files = Array.from(input.files).map(f => f.name);
          resolve(options?.multiple ? files : files[0]);
        } else {
          resolve(null);
        }
      };
      
      input.click();
    });
  }
}

/**
 * Open a save file dialog
 * In web mode, returns null as file system dialogs are not available
 */
export async function saveDialog(options?: SaveDialogOptions): Promise<string | null> {
  if (IS_TAURI) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    return save(options);
  } else {
    // In web mode, save dialogs are not available - use download API instead
    console.warn('[Web Mode] Save dialog not available. Use browser download API for file saving.');
    return null;
  }
}

// ============================================================================
// FILE SYSTEM (Tauri plugin-fs / Web fallback)
// ============================================================================

export enum BaseDirectory {
  AppCache = 16,
  AppConfig = 13,
  AppData = 14,
  AppLocalData = 15,
  AppLog = 17,
  Audio = 1,
  Cache = 2,
  Config = 3,
  Data = 4,
  Desktop = 6,
  Document = 7,
  Download = 8,
  Executable = 23,
  Font = 9,
  Home = 10,
  LocalData = 5,
  Log = 24,
  Picture = 11,
  Public = 12,
  Resource = 18,
  Runtime = 22,
  Temp = 19,
  Template = 20,
  Video = 21,
}

export interface FsOptions {
  baseDir?: BaseDirectory;
}

/**
 * Read a text file
 */
export async function readTextFile(path: string, options?: FsOptions): Promise<string> {
  if (IS_TAURI) {
    const { readTextFile: tauriReadTextFile } = await import('@tauri-apps/plugin-fs');
    return tauriReadTextFile(path, options);
  } else {
    // In web mode, throw error (file system not available)
    throw new Error('[Web Mode] File system operations are not available. Use server-side APIs.');
  }
}

/**
 * Write a text file
 */
export async function writeTextFile(path: string, contents: string, options?: FsOptions): Promise<void> {
  if (IS_TAURI) {
    const { writeTextFile: tauriWriteTextFile } = await import('@tauri-apps/plugin-fs');
    return tauriWriteTextFile(path, contents, options);
  } else {
    // In web mode, throw error (file system not available)
    throw new Error('[Web Mode] File system operations are not available. Use server-side APIs.');
  }
}

/**
 * Read a binary file
 */
export async function readFile(path: string, options?: FsOptions): Promise<Uint8Array> {
  if (IS_TAURI) {
    const { readFile: tauriReadFile } = await import('@tauri-apps/plugin-fs');
    return tauriReadFile(path, options);
  } else {
    throw new Error('[Web Mode] File system operations are not available. Use server-side APIs.');
  }
}

/**
 * Write a binary file
 */
export async function writeFile(path: string, contents: Uint8Array, options?: FsOptions): Promise<void> {
  if (IS_TAURI) {
    const { writeFile: tauriWriteFile } = await import('@tauri-apps/plugin-fs');
    return tauriWriteFile(path, contents, options);
  } else {
    throw new Error('[Web Mode] File system operations are not available. Use server-side APIs.');
  }
}

/**
 * Create a directory
 */
export async function mkdir(path: string, options?: FsOptions & { recursive?: boolean }): Promise<void> {
  if (IS_TAURI) {
    const { mkdir: tauriMkdir } = await import('@tauri-apps/plugin-fs');
    return tauriMkdir(path, options);
  } else {
    throw new Error('[Web Mode] File system operations are not available. Use server-side APIs.');
  }
}

// ============================================================================
// AUTO UPDATER (Tauri plugin-updater / Web fallback)
// ============================================================================

export interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

/**
 * Check for updates
 */
export async function checkForUpdates(): Promise<UpdateInfo | null> {
  if (IS_TAURI) {
    const { check } = await import('@tauri-apps/plugin-updater');
    const update = await check();
    if (update) {
      return {
        version: update.version,
        date: update.date,
        body: update.body,
      };
    }
    return null;
  } else {
    // In web mode, no auto-update needed
    console.warn('[Web Mode] Auto-updater not available in web mode.');
    return null;
  }
}

/**
 * Relaunch the application
 */
export async function relaunch(): Promise<void> {
  if (IS_TAURI) {
    const { relaunch: tauriRelaunch } = await import('@tauri-apps/plugin-process');
    return tauriRelaunch();
  } else {
    // In web mode, reload the page
    window.location.reload();
  }
}

// Export for backward compatibility
export default invoke;
